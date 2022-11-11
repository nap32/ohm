#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use std::{convert::Infallible, net::SocketAddr};
use hyper::{Body, Request, Response, Server, Client, StatusCode};
use hyper::body::HttpBody as _;
use hyper::service::{make_service_fn, service_fn};
use tokio::io::{stdout, AsyncWriteExt as _};
use tokio::net::TcpStream;
use hyper_tls::HttpsConnector;
use std::error::Error;

//
// https://hyper.rs/guides/server/hello-world/
// https://hyper.rs/guides/client/advanced/
// https://tokio.rs/tokio/tutorial/async
//

async fn handle(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response = send_request(request).await;
    match response {
        Ok(t) => {
            let response = *t;
            return Ok(response);
        },
        Err(e) => {
            let body : Body = Body::empty();
            let response = Response::builder()
                .status(500)
                .body(body)
                .unwrap();
            return Ok(response)
        },
    }
}

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

async fn send_request(request: Request<Body>) -> Result<Box<Response<Body>>, Box<dyn Error + Send + Sync>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    //let uri = "https://www.chess.com".parse()?;
    //let mut resp = client.get(uri).await?;
    println!("Request:\nURI: {}", request.uri());
    println!("Headers: {:#?}", request.headers());

    let mut result = client.request(request).await;
    let mut response = Response::default();
    match result {
        Ok(t) => {
            response = t;
        },
        Err(e) => {
            println!("Err! {}", e);
        }
    }

    let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
    let mut response_foo = Response::builder()
        .body(Body::from(body_bytes.clone()))?;
    let mut response_bar = Response::builder()
        .body(Body::from(body_bytes.clone()))?;

    while let Some(chunk) = response_foo.body_mut().data().await {
        stdout().write_all(&chunk?).await?;
    }
    
    response = response_bar;
    println!("Response: {}", response.status());

    let b = Box::new(response);

    Ok(b)
}
