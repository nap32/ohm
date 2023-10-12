use crate::data::Datastore;
use crate::model::auth::AuthInfo;
use crate::model::traffic::Traffic;
use crate::service::ca::CA;
use crate::DATASTORE_CLIENT;
use crate::FILTER_CHAIN;

use std::convert::Infallible;
use std::sync::Arc;

use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Client, Method, Request, Response, StatusCode, Uri};

use tokio::io::{AsyncRead, AsyncWrite};

use http::uri::{Authority, Scheme};
use hyper_tls::HttpsConnector;
use tokio_rustls::TlsAcceptor;

use std::collections::HashMap;
use flate2::read::{DeflateDecoder, GzDecoder};
use std::io::{Read, Write};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn handle_request(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response: Response<Body>;
    let result: Result<Response<Body>, Error> = if request.method() == Method::CONNECT {
        handle_connect(request).await
    } else {
        send_request(request).await
    };
    match result {
        Ok(t) => response = t,
        Err(e) => {
            response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(e.to_string()))
                .unwrap();
        }
    }
    Ok(response)
}

pub async fn handle_connect(mut request: Request<Body>) -> Result<Response<Body>, Error> {
    if let Some(_addr) = request.uri().authority().map(|auth| auth.to_string()) {
        tokio::task::spawn(async move {
            match hyper::upgrade::on(&mut request).await {
                Ok(upgraded) => {
                    let mut ca = CA::new().await;
                    let proxy_config = ca
                        .get_proxy_config(request)
                        .await
                        .expect("Couldn't get proxy certificate.");
                    let stream = match TlsAcceptor::from(Arc::new(proxy_config))
                        .accept(upgraded)
                        .await
                    {
                        Ok(stream) => stream,
                        Err(_e) => return,
                    };
                    if let Err(e) = serve_stream(stream).await {
                        if !e.to_string().starts_with("error shutting down connection") {
                            println!("[ERROR] [src/service/proxy.rs] [handle_connect]: (serve_stream error!) {:?}", e);
                        }
                    }
                }
                Err(e) => eprintln!("Upgrade error: {}", e),
            }
        });
        Ok(Response::new(Body::empty()))
    } else {
        eprintln!(
            "[ERROR] [src/service/proxy.rs] [handle_connect]: (CONNECT is not a socket addr): {:?}",
            request.uri()
        );
        let mut response = Response::new(Body::from("CONNECT must be to a socket address."));
        *response.status_mut() = StatusCode::BAD_REQUEST;
        Ok(response)
    }
}

// This function needs refactored - borrowed hudsucker's handling to get a proof-of-concept.
// For proxying, must rewrite URI into absolute format - {SCHEME}://{AUTHORITY}/{URI}
pub async fn serve_stream<I>(stream: I) -> Result<(), Error>
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let service = service_fn(|mut req| {
        if req.version() == hyper::Version::HTTP_10 || req.version() == hyper::Version::HTTP_11 {
            // Abstract this into another function - you need to do absolute URI rewriting.
            let (mut parts, body) = req.into_parts();
            let authority = parts
                .headers
                .get(hyper::header::HOST)
                .expect("Host is a required header")
                .as_bytes();
            parts.uri = {
                let mut parts = parts.uri.into_parts();
                parts.scheme = Some(Scheme::HTTPS);
                parts.authority =
                    Some(Authority::try_from(authority).expect("Failed to parse authority"));
                Uri::from_parts(parts).expect("Failed to build URI")
            };
            req = Request::from_parts(parts, body);
        };

        send_request(req)
    });

    let result = Http::new()
        .serve_connection(stream, service)
        .with_upgrades()
        .await;
    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

pub async fn send_request(request: Request<Body>) -> Result<Response<Body>, Error> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let (request_browser, request_traffic) = clone_request(request).await.unwrap();

    let result = client.request(request_browser).await;
    let mut response = Response::default();
    match result {
        Ok(t) => {
            response = t;
        }
        Err(e) => {
            println!("[ERROR] [src/service/proxy.rs] [send_request]: {}", e);
        }
    }

    let (response_browser, response_traffic) = clone_response(response).await.unwrap();

    let mut traffic = Traffic::new(request_traffic, response_traffic).await;
    tokio::task::spawn(async move {
        process_traffic(&mut traffic).await;
    });
    Ok(response_browser)
}

pub async fn process_traffic(traffic: &mut Traffic) {
    let filter_chain = FILTER_CHAIN
        .get()
        .expect("Traffic filtering chain not intialized.");
    println!("{:?}", traffic);
    if (filter_chain.filter(traffic).await).is_ok() {
        store_traffic(traffic).await
    }
}

pub async fn store_traffic(traffic: &Traffic) {
    let datastore = DATASTORE_CLIENT.get().expect("Datastore not initialized.");
    let result = datastore.add_traffic(traffic).await;
    match result {
        Ok(()) => {}
        Err(e) => {
            println!("[ERROR] [src/service/proxy.rs] [store_traffic]: {:?}", e);
        }
    }
}

pub async fn store_auth(auth: &AuthInfo) {
    let datastore = DATASTORE_CLIENT.get().expect("Datastore not initialized.");
    let result = datastore.add_authinfo(auth).await;
    match result {
        Ok(()) => {}
        Err(e) => {
            println!("[ERROR] [src/service/proxy.rs] [store_auth]: {:?}", e);
        }
    }
}

pub async fn clone_request(
    request: Request<Body>,
) -> Result<(hyper::Request<Body>, crate::model::traffic::Request), Error> {
    let (parts, body) = request.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await?;

    let mut hyper_request = Request::builder()
        .uri(parts.uri.clone())
        .method(parts.method.clone())
        .version(parts.version);
    {
        let headers = hyper_request.headers_mut().unwrap();
        headers.extend(parts.headers.clone());
    }
    let hyper_request = hyper_request.body(Body::from(body_bytes.clone()))?;

    let body = parse_body(body_bytes, parts.headers.get("Content-Encoding")).await;
    let traffic_request = crate::model::traffic::Request::new(
        parts.method.to_string(),
        parts.uri.scheme_str().unwrap_or_else(|| "").to_string(),
        parts.uri.host().unwrap_or_else(|| "").to_string(),
        parts.uri.path().to_string(),
        parse_query(parts.uri.query().unwrap_or_else(|| "")).await,
        parse_headers(parts.headers).await,
        body,
        parse_version(parts.version).await,
    )
    .await;

    Ok((hyper_request, traffic_request))
}

pub async fn clone_response(
    response: Response<Body>,
) -> Result<(hyper::Response<Body>, crate::model::traffic::Response), Error> {
    let (parts, body) = response.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await?;

    let mut hyper_response = Response::builder()
        .status(parts.status)
        .version(parts.version);
    {
        let headers = hyper_response.headers_mut().unwrap();
        headers.extend(parts.headers.clone());
    }
    let hyper_response = hyper_response.body(Body::from(body_bytes.clone()))?;

    let body = parse_body(body_bytes, parts.headers.get("Content-Encoding")).await;
    let traffic_response = crate::model::traffic::Response {
        status: parts.status.into(),
        headers: parse_headers(parts.headers).await,
        body: body,
        version: parse_version(parts.version).await,
    };

    Ok((hyper_response, traffic_response))
}

pub async fn parse_query(queries: &str) -> HashMap<String, String> {
    let mut q = HashMap::<String, String>::new();
    for pair in queries.split("&") {
        let query: Vec<_> = pair.split("=").collect();
        if query.len() == 2 {
            q.insert(query[0].to_string(), query[1].to_string());
        }
    }
    q
}

pub async fn parse_headers(headers: hyper::HeaderMap) -> HashMap<String, String> {
    let mut h = HashMap::<String, String>::new();
    for (key, value) in headers.iter() {
        if let Ok(vs) = value.to_str() {
            h.insert(key.to_string(), vs.to_string());
        }
    }
    h
}

pub async fn parse_body(body_bytes: hyper::body::Bytes, encoding: Option::<&hyper::header::HeaderValue>) -> String {
    let body_string = match std::str::from_utf8(&body_bytes) {
        Ok(b) => b,
        Err(_) => ""
    };
    match encoding {
        Some(v) => {
            let vs = match v.to_str() {
                Ok(str) => str,
                Err(_) => "",
            };
            let result = match vs {
                "deflate" => decompress_deflate(body_string).await,
                "gzip" => decompress_gzip(body_string).await,
                "br" => decompress_gzip(body_string).await,
                _ => body_string.to_string(),
            };
            result
        },
        None => {
            body_string.to_string()
        }
    }
}

pub async fn decompress_gzip(encoded: &str) -> String {
    let encoded_bytes = &encoded.as_bytes().to_vec();
    let mut decoded_buffer = Vec::new();
    let mut gz = GzDecoder::new(&encoded_bytes[..]);
    gz.read_to_end(&mut decoded_buffer).expect("Decompressing gz failed.");
    let result = match String::from_utf8(decoded_buffer) {
        Ok(s) => {
            s
        },
        Err(_) => {
            encoded.to_string()
        }
    };
    result
}

pub async fn decompress_deflate(encoded: &str) -> String {
    let encoded_bytes = &encoded.as_bytes().to_vec();
    let mut decoded_buffer = Vec::new();
    let mut deflate = DeflateDecoder::new(&encoded_bytes[..]);
    deflate.read_to_end(&mut decoded_buffer).expect("Decompressing deflate failed.");
    let result = match String::from_utf8(decoded_buffer) {
        Ok(s) => {
            s
        },
        Err(_) => {
            "".to_owned()
        }
    };
    result
}

pub async fn decompress_br(encoded: &str) -> String {
    let encoded_bytes = &encoded.as_bytes().to_vec();
    let mut decoded_buffer = Vec::new();
    let mut brotli = brotli::DecompressorWriter::new(&mut decoded_buffer[..], 4096);
    brotli.write_all(&encoded_bytes[..]).unwrap();
    brotli.into_inner().unwrap();
    let result = match String::from_utf8(decoded_buffer) {
        Ok(s) => {
            s
        },
        Err(_) => {
            "".to_owned()
        }
    };
    result
}

pub async fn parse_version(version: http::version::Version) -> String {
    return match version {
        http::Version::HTTP_09 => "HTTP/0.9".to_owned(),
        http::Version::HTTP_10 => "HTTP/1.0".to_owned(),
        http::Version::HTTP_11 => "HTTP/1.1".to_owned(),
        http::Version::HTTP_2 => "HTTP/2.0".to_owned(),
        http::Version::HTTP_3 => "HTTP/3.0".to_owned(),
        _ => "HTTP/1.0".to_owned(),
    };
}
