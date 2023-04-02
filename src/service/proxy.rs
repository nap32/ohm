use crate::DATASTORE_CLIENT;
use crate::FILTER_CHAIN;
use crate::service::ca::CA;
use crate::model::traffic::Traffic;
use crate::model::auth::AuthInfo;
use crate::data::Datastore;

use std::convert::Infallible;
use std::sync::Arc;

use hyper::{Body, Request, Response, Client, Method, StatusCode, Uri};
use hyper::service::service_fn;
use hyper::server::conn::Http;

use tokio::io::{AsyncRead, AsyncWrite};

use hyper_tls::HttpsConnector;
use tokio_rustls::TlsAcceptor;
use http::uri::{Authority, Scheme};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn handle_request(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response : Response<Body>;
    let result : Result<Response<Body>, Error>; 
    if request.method() == Method::CONNECT {
        result = handle_connect(request).await;
    }else{
        result = send_request(request).await;
    }
    match result {
        Ok(t) => response = t,
        Err(e) => {
            response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(e.to_string()))
                .unwrap();
        },
    }
    return Ok(response)
}

pub async fn handle_connect(mut request: Request<Body>) -> Result<Response<Body>, Error> {
    
    if let Some(_addr) = request.uri().authority().and_then(|auth| Some(auth.to_string())) {
        tokio::task::spawn(async move {
            match hyper::upgrade::on(&mut request).await {
                Ok(upgraded) => {
                    let mut ca = CA::new().await;
                    let proxy_config = ca.get_proxy_config(request).await.expect("Couldn't get proxy certificate.");
                    let stream = match TlsAcceptor::from(Arc::new(proxy_config)).accept(upgraded).await {
                            Ok(stream) => stream,
                            Err(_e) => { return },
                    };
                    if let Err(e) = serve_stream(stream).await {
                        if !e.to_string().starts_with("error shutting down connection") {
                            println!("[ERROR] [src/service/proxy.rs] [handle_connect]: (serve_stream error!) {:?}", e);
                        }
                    }
                },
                Err(e) => eprintln!("Upgrade error: {}", e),
            }
        });
        Ok(Response::new(Body::empty()))
    }else{
        eprintln!("[ERROR] [src/service/proxy.rs] [handle_connect]: (CONNECT is not a socket addr): {:?}", request.uri());
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
                parts.scheme = Some(Scheme::HTTPS);
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

pub async fn send_request(request: Request<Body>) -> Result<Response<Body>, Error> {

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let (request_browser, request_traffic) = clone_request(request).await.unwrap();

    let result = client.request(request_browser).await;
    let mut response = Response::default();
    match result {
        Ok(t) => { response = t; },
        Err(e) => { println!("[ERROR] [src/service/proxy.rs] [send_request]: {}", e); },
    }

    let (response_browser, response_traffic) = clone_response(response).await.unwrap();
    
    let mut traffic = Traffic::new(request_traffic, response_traffic).await;
    tokio::task::spawn(async move {
        process_traffic(&mut traffic).await;
    });
    Ok(response_browser)
}

pub async fn process_traffic(traffic: &mut Traffic) {
    let filter_chain = FILTER_CHAIN.get().expect("Traffic filtering chain not intialized.");
    match filter_chain.filter(traffic).await{
        Ok(_) => {
            store_traffic(traffic).await;
        },
        Err(_) => {
            /* Filtering chain dropped traffic. */
        },
    }
}

pub async fn store_traffic(traffic: &Traffic) {
    let datastore = DATASTORE_CLIENT.get().expect("Datastore not initialized.");
    let result = datastore.add_traffic(&traffic).await;
    match result {
        Ok(()) => {},
        Err(e) => {
            println!("[ERROR] [src/service/proxy.rs] [store_traffic]: {:?}", e);
        },
    }
}

pub async fn store_auth(auth: &AuthInfo) {
    let datastore = DATASTORE_CLIENT.get().expect("Datastore not initialized.");
    let result = datastore.add_authinfo(&auth).await;
    match result {
        Ok(()) => {},
        Err(e) => {
            println!("[ERROR] [src/service/proxy.rs] [store_auth]: {:?}", e);
        },
    }
}

// TODO: Implement .Copy() for hyper::traffic or find a better way.
// "parts.extensions" is not cloned because it doesn't implement the trait and is left out here.

pub async fn clone_request(request: Request<Body>) -> Result<(Request<Body>, Request<Body>), Error> {
    let (parts, body) = request.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await?;
    
    let mut req1 = Request::builder()
        .uri(parts.uri.clone())
        .method(parts.method.clone())
        .version(parts.version.clone());
    {
        let headers = req1.headers_mut().unwrap();
        headers.extend(parts.headers.clone());
    }
    let req1 = req1.body(Body::from(body_bytes.clone()))?;

    let mut req2 = Request::builder()
        .uri(parts.uri.clone())
        .method(parts.method.clone())
        .version(parts.version.clone());
    {
        let headers = req2.headers_mut().unwrap();
        headers.extend(parts.headers.clone());
    }
    let req2 = req2.body(Body::from(body_bytes.clone()))?;
    
    return Ok((req1, req2))
}

pub async fn clone_response(response: Response<Body>) -> Result<(Response<Body>, Response<Body>), Error> {
    let (parts, body) = response.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await?;
    
    let mut res1 = Response::builder()
        .status(parts.status.clone())
        .version(parts.version.clone());
    {
        let headers = res1.headers_mut().unwrap();
        headers.extend(parts.headers.clone());
    }
    let res1 = res1.body(Body::from(body_bytes.clone()))?;
    
    let mut res2 = Response::builder()
        .status(parts.status.clone())
        .version(parts.version.clone());
    {
        let headers = res2.headers_mut().unwrap();
        headers.extend(parts.headers.clone());
    }
    let res2 = res2.body(Body::from(body_bytes.clone()))?;

    return Ok((res1, res2))
}
