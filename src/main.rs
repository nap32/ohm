#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};
use std::sync::Arc;
use std::fs;
use std::io::Read;

use hyper::{Body, Request, Response, Server, Client, Method, StatusCode, Uri};
use hyper::header::{HeaderMap, HeaderName, UPGRADE};
use hyper::body::HttpBody as _;
use hyper::service::{make_service_fn, service_fn};
use hyper::upgrade::Upgraded;
use hyper::server::conn::Http;

use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use tokio::io::stdout;
use tokio::net::{TcpStream, TcpListener};

use hyper_tls::HttpsConnector;
use tokio_rustls::{TlsAcceptor, TlsStream};
use tokio_rustls::rustls;
use tokio_rustls::rustls::{ServerConfig, ConfigBuilder, PrivateKey};
use http::uri::{Authority, Scheme};

use openssl::asn1::{Asn1Integer, Asn1Time};
use openssl::bn::BigNum;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private};
use openssl::rand;
use openssl::x509::extension::SubjectAlternativeName;
use openssl::x509::{X509, X509Builder, X509NameBuilder};

use flate2::read::GzDecoder;

//
// https://hyper.rs/guides/server/hello-world/
// https://hyper.rs/guides/client/advanced/
// https://tokio.rs/tokio/tutorial/async
// https://github.com/hyperium/hyper/blob/master/examples/upgrades.rs
// https://github.com/hyperium/hyper/issues/1884
// https://github.com/omjadas/hudsucker/blob/main/src/certificate_authority/openssl_authority.rs
// https://docs.rs/crate/openssl/latest/source/examples/mk_certs.rs
//

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() { 

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });

    let server = Server::bind(&addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn handle_request(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    

    let mut response = Response::default();
    if request.method() == Method::CONNECT {
        let result = handle_connect(request).await;
        match result {
            Ok(t) => response = t,
            Err(e) => println!("Error:\n{}", e),
        }
    }else{
        let result = send_request(request).await;
        match result {
            Ok(t) => response = t,
            Err(e) => println!("Error:\n{}", e),
        }
    }
    
    return Ok(response)
}

async fn handle_connect(mut request: Request<Body>) -> Result<Response<Body>, Error> {
    
    if let Some(addr) = host_addr(request.uri()) {
        tokio::task::spawn(async move {
            match hyper::upgrade::on(&mut request).await {
                Ok(upgraded) => {
                    let proxy_config = get_proxy_config(request).await.expect("Couldn't get proxy certificate.");
                    let stream = match TlsAcceptor::from(Arc::new(proxy_config)).accept(upgraded).await {
                            Ok(stream) => stream,
                            Err(e) => { return },
                    };
                    if let Err(e) = serve_stream(stream, Scheme::HTTPS).await {
                        if !e.to_string().starts_with("error shutting down connection") {
                            println!("Handle Connect's serve_stream error! {}", e);
                        }
                    }
                    //if let Err(e) = tunnel(upgraded, addr).await{
                    //    eprintln!("Upgrade error: {}", e)
                    //}
                },
                Err(e) => eprintln!("Upgrade error: {}", e),
            }
        });
        Ok(Response::new(Body::empty()))
    }else{
        eprintln!("CONNECT host is not a socket addr: {:?}", request.uri());
        let mut response = Response::new(Body::from("CONNECT must be to a socket address."));
        *response.status_mut() = StatusCode::BAD_REQUEST;
        Ok(response)
    }

}


async fn send_request(request: Request<Body>) -> Result<Response<Body>, Error> {

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    //print_request_metadata(&request);

    let mut result = client.request(request).await;
    let mut response = Response::default();
    match result {
        Ok(t) => { response = t; },
        Err(e) => { println!("Err! {}", e); },
    }

    //print_response_metadata(&response);

    if response.headers().contains_key("content-encoding") && response.headers()["content-encoding"] == "gzip" {
    let mut response_consumer = Response::default();
    let mut response_provider = Response::default();
    let clone_result = clone_response(response).await;
    match clone_result {
        Ok(t) => { (response_consumer, response_provider) = t; },
        Err(e) => println!("Error:\n{}", e),
    }
    let result_print = print_response(response_consumer).await;
    match result_print {
        Ok(t) => {},
        Err(e) => println!("Error:\n{}", e),
    }
    response = response_provider;
    }
    Ok(response)
}

// This function is stolen from hudsucker -- I think it's a bit complex for my level of Rustacean,
// I need to rewrite it -- but it's making sure it replaces the URI with https://authority or
// something.
async fn serve_stream<I>(stream: I, scheme: Scheme) -> Result<(), Error>
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let service = service_fn(|mut req| {
        if req.version() == hyper::Version::HTTP_10 || req.version() == hyper::Version::HTTP_11
        {
            // Abstract this into another function - you need to do absolute URI rewriting.
            let (mut parts, body) = req.into_parts();
            let authority = parts
                .headers
                .get(hyper::header::HOST)
                .expect("Host is a required header")
                .as_bytes();
            parts.uri = {
                let mut parts = parts.uri.into_parts();
                parts.scheme = Some(scheme.clone());
                parts.authority = Some(Authority::try_from(authority).expect("Failed to parse authority"));
                Uri::from_parts(parts).expect("Failed to build URI")
           };
            req = Request::from_parts(parts, body);
        };

        send_request(req)
    });

    let result = Http::new().serve_connection(stream, service).with_upgrades().await;
    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

async fn get_proxy_config(mut request: Request<Body>) -> Result<ServerConfig, Error>{
    
    let authority = request
        .uri()
        .authority()
        .expect("URI does not contain authority");
    
    create_server_config(authority).await
}

async fn create_server_config(authority: &Authority) -> Result<ServerConfig, Error>{
    
    //  This needs refactored.
    let private_key_bytes: &[u8] = include_bytes!("../ca/ohm.key");
    let pkey = PKey::private_key_from_pem(private_key_bytes).expect("Failed to parse private key");
    let private_key = rustls::PrivateKey(
        pkey.private_key_to_der()
            .expect("Failed to encode private key"),
    );
    //////////////////////////

    let result = create_proxy_certificate(authority).await;
    let cert : rustls::Certificate;
    match result {
        Ok(t) => { cert = t; },
        Err(e) => { return Err(e) },
    }

    let mut server_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec!(cert), private_key.clone())
        .expect("Failed to build ServerConfig.");

    server_config.alpn_protocols = vec![
        #[cfg(feature = "http2")]
        b"h2".to_vec(),
        b"http/1.1".to_vec(),
    ];

    Ok(server_config)
}

async fn create_proxy_certificate(authority: &Authority) -> Result<rustls::Certificate, Error> {
    
    // This needs refactored.
    let private_key_bytes: &[u8] = include_bytes!("../ca/ohm.key");
    let pkey = PKey::private_key_from_pem(private_key_bytes).expect("Failed to parse private key");
    let ca_cert_bytes: &[u8] = include_bytes!("../ca/ohm.pem");
    let ca_cert = X509::from_pem(ca_cert_bytes).expect("Failed to parse CA certificate pem.");
    //////////

    let mut name_builder = X509NameBuilder::new()?;
    name_builder.append_entry_by_text("C", "US").unwrap();
    name_builder.append_entry_by_text("ST", "CA").unwrap();
    name_builder.append_entry_by_text("O", "OHM").unwrap();
    name_builder.append_entry_by_text("CN", authority.host()).unwrap();
    let name = name_builder.build();

    let mut x509_builder = X509Builder::new().unwrap();
    x509_builder.set_subject_name(&name)?;
    x509_builder.set_version(2)?;

    let not_before = Asn1Time::days_from_now(0)?;
    x509_builder.set_not_before(&not_before)?;
    let not_after = Asn1Time::days_from_now(365)?;
    x509_builder.set_not_after(&not_after)?;

    x509_builder.set_pubkey(&pkey)?; 
    x509_builder.set_issuer_name(ca_cert.subject_name())?;

    let alternative_name = SubjectAlternativeName::new()
        .dns(authority.host())
        .build(&x509_builder.x509v3_context(Some(&ca_cert), None))?;
    x509_builder.append_extension(alternative_name)?;

    let mut serial_number = [0; 16];
    rand::rand_bytes(&mut serial_number)?;
    let serial_number = BigNum::from_slice(&serial_number)?;
    let serial_number = Asn1Integer::from_bn(&serial_number)?;
    x509_builder.set_serial_number(&serial_number)?;

    x509_builder.sign(&pkey, MessageDigest::sha256())?;
    let x509 = x509_builder.build();

    Ok(rustls::Certificate(x509.to_der()?))
}

// This works - plug it into the handle_connect function w/o the TlsAcceptor line.
async fn tunnel(mut upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    let mut server = TcpStream::connect(addr).await?;
    let (from_client, from_server) = tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;
    
    println!("Client wrote {} bytes and received {} bytes.", from_client, from_server);
    Ok(())
}

fn host_addr(uri: &hyper::Uri) -> Option<String> {
    uri.authority().and_then(|auth| Some(auth.to_string()))
}

async fn clone_response(mut response: Response<Body>) -> Result<(Response<Body>, Response<Body>), Error> {
    let (parts, body) = response.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await?;
    let mut response_provider = Response::builder()
        .status(parts.status.clone())
        //.extensions(parts.extensions)
        .version(parts.version.clone());
    {
        let headers = response_provider.headers_mut().unwrap();
        headers.extend(parts.headers.clone());
    }
    let response_provider = response_provider.body(Body::from(body_bytes.clone()))?;
    let mut response_consumer = Response::builder()
        .status(parts.status.clone())
        //.extension(parts.extensions)
        .version(parts.version.clone());
    {
        let headers = response_consumer.headers_mut().unwrap();
        headers.extend(parts.headers.clone());
    }
    let response_consumer = response_consumer.body(Body::from(body_bytes.clone()))?;
    return Ok((response_consumer, response_provider))
}

async fn clone_request(request: Request<Body>) -> Result<(Request<Body>, Request<Body>), Error> {
    let body_bytes = hyper::body::to_bytes(request.into_body()).await?;
    let mut request_provider = Request::builder()
        .body(Body::from(body_bytes.clone()))?;
    let mut request_consumer = Request::builder()
        .body(Body::from(body_bytes.clone()))?;
    return Ok((request_consumer, request_provider))
}

async fn print_response_body(mut response: Response<Body>) -> Result<(), Error> {
    println!("Response Body:");
    let (parts, body) = response.into_parts();
    if parts.headers.contains_key("content-encoding") && parts.headers["content-encoding"] == "gzip"{
        let body_bytes = hyper::body::to_bytes(body).await?;
        let mut gunzipped = String::new();
        let mut d = GzDecoder::new(&*body_bytes);
        d.read_to_string(&mut gunzipped).unwrap();
        println!("{:?}", gunzipped);
    }else{
        let body_bytes = hyper::body::to_bytes(body).await?;
        println!("Body:\n{:?}", body_bytes);
    }
    return Ok(())
}

fn print_request_metadata(request: &Request<Body>) {
    println!("Request:");
    println!("Method: {}", request.method());
    println!("URI: {}", request.uri());
    println!("Headers: {:#?}", request.headers());
}

fn print_response_metadata(response: &Response<Body>) {
    println!("Response:");
    println!("Method: {}", response.status());
    println!("Headers: {:#?}", response.headers());
}

async fn print_response(response: Response<Body>) -> Result<(), Error>{
    print_response_metadata(&response);
    print_response_body(response).await
}
