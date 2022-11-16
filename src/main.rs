#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::{Body, Request, Response, Server, Client, Method, StatusCode};
use hyper::header::{HeaderMap, HeaderName, UPGRADE};
use hyper::body::HttpBody as _;
use hyper::service::{make_service_fn, service_fn};
use hyper::upgrade::Upgraded;

use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::io::stdout;
use tokio::net::{TcpStream, TcpListener};
use hyper_tls::HttpsConnector;

//
// https://hyper.rs/guides/server/hello-world/
// https://hyper.rs/guides/client/advanced/
// https://tokio.rs/tokio/tutorial/async
// https://github.com/hyperium/hyper/blob/master/examples/upgrades.rs
// https://github.com/hyperium/hyper/issues/1884
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
    
    print_request_metadata(&request);

    let mut response = Response::default();
    if request.method() == Method::CONNECT {
        println!("CONNECT received!");
        let result = handle_connect(request).await;
        match result {
            Ok(t) => response = *t,
            Err(e) => println!("Error:\n{}", e),
        }
    }else{
        println!("NON-CONNECT received!");
        let result = send_request(request).await;
        match result {
            Ok(t) => response = *t,
            Err(e) => println!("Error:\n{}", e),
        }
    }
    
    print_response_metadata(&response);

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
    return Ok(response)
}

async fn handle_connect(mut request: Request<Body>) -> Result<Box<Response<Body>>, Error> {
    
    if let Some(addr) = host_addr(request.uri()) {
        tokio::task::spawn(async move {
            match hyper::upgrade::on(request).await {
                Ok(upgraded) => {
                    if let Err(e) = tunnel(upgraded, addr).await{
                    }
                },
                Err(e) => eprintln!("Upgrade error: {}", e),
            }
        });
        Ok(Box::new(Response::new(Body::empty())))
    }else{
        eprintln!("CONNECT host is not a socket addr: {:?}", request.uri());
        let mut response = Response::new(Body::from("CONNECT must be to a socket address."));
        *response.status_mut() = StatusCode::BAD_REQUEST;
        Ok(Box::new(response))
    }

}


async fn send_request(request: Request<Body>) -> Result<Box<Response<Body>>, Error> {

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let mut result = client.request(request).await;
    let mut response = Response::default();
    match result {
        Ok(t) => { response = t; },
        Err(e) => { println!("Err! {}", e); },
    }

    let b = Box::new(response);
    Ok(b)
}

async fn tunnel(mut upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    let mut server = TcpStream::connect(addr).await?;
    let (from_client, from_server) = tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;
    println!("Client wrote {} bytes and received {} bytes.", from_client, from_server);
    Ok(())
}

fn host_addr(uri: &hyper::Uri) -> Option<String> {
    uri.authority().and_then(|auth| Some(auth.to_string()))
}

async fn clone_response(response: Response<Body>) -> Result<(Response<Body>, Response<Body>), Error> {
    let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
    let mut response_provider = Response::builder()
        .body(Body::from(body_bytes.clone()))?;
    let mut response_consumer = Response::builder()
        .body(Body::from(body_bytes.clone()))?;
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

async fn print_response(mut response: Response<Body>) -> Result<(), Error> {
    while let Some(chunk) = response.body_mut().data().await {
        stdout().write_all(&chunk?).await?;
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

