#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

use std::{convert::Infallible, net::SocketAddr};
use hyper::{Body, Request, Response, Server, Client, StatusCode};
use hyper::body::HttpBody as _;
use hyper::service::{make_service_fn, service_fn};
use tokio::io::{stdout, AsyncWriteExt as _};
use tokio::net::TcpStream;
use hyper_tls::HttpsConnector;

//
// https://hyper.rs/guides/server/hello-world/
// https://hyper.rs/guides/client/advanced/
// https://tokio.rs/tokio/tutorial/async
// https://github.com/hyperium/hyper/blob/master/examples/upgrades.rs
//

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() { 
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle))
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn handle(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let result = send_request(request).await;
    let mut response = Response::default();
    let mut response_consumer = Response::default();
    let mut response_provider = Response::default();

    match result {
        Ok(t) => {
            response = *t;
            let clone_result = clone_response(response).await;
            match clone_result {
                Ok(t) => { (response_consumer, response_provider) = t; },
                Err(e) => println!("Error:\n{}", e),
            }
        },
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

async fn send_request(request: Request<Body>) -> Result<Box<Response<Body>>, Error> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    println!("Request:");
    println!("Method: {}", request.method());
    println!("URI: {}", request.uri());
    println!("Headers: {:#?}", request.headers());

    let mut result = client.request(request).await;
    let mut response = Response::default();
    match result {
        Ok(t) => { response = t; },
        Err(e) => { println!("Err! {}", e); },
    }

    println!("Response:");
    println!("Method: {}", response.status());
    println!("Headers: {:#?}", response.headers());

    let b = Box::new(response);
    Ok(b)
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

