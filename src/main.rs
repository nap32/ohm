#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

pub mod model;
use model::record::*;
use crate::model::record::Record;
use model::traffic::*;
use crate::model::traffic::Traffic;

pub mod data;
use crate::data::Datastore;
use data::mongo::*;
use crate::data::mongo::Mongo;

pub mod service;
use service::ca::*;
use crate::service::ca::CA;
use service::proxy::*;
use service::config::*;
use crate::service::config::Config;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::io::Read;
use std::env;

use hyper::{Body, Request, Response, Server, Client, Method, StatusCode, Uri};
use hyper::service::{make_service_fn, service_fn};
use hyper::upgrade::Upgraded;
use hyper::server::conn::Http;

use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
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

use once_cell::sync::OnceCell;

// https://hyper.rs/guides/server/hello-world/
// https://hyper.rs/guides/client/advanced/
// https://tokio.rs/tokio/tutorial/async
// https://github.com/hyperium/hyper/blob/master/examples/upgrades.rs
// https://github.com/hyperium/hyper/issues/1884
// https://github.com/omjadas/hudsucker/blob/main/src/certificate_authority/openssl_authority.rs
// https://docs.rs/crate/openssl/latest/source/examples/mk_certs.rs

type Error = Box<dyn std::error::Error + Send + Sync>;

static CONFIG : OnceCell<Config> = OnceCell::new();
static DATASTORE_CLIENT : OnceCell<Mongo> = OnceCell::new(); 

#[tokio::main]
async fn main() { 

    match CONFIG.set(Config::new(get_config_argument().await).await) {
        Ok(()) => (),
        Err(e) => { panic!("Error setting Config."); },
    }
    match DATASTORE_CLIENT.set(Mongo::new().await) {
        Ok(()) => (),
        Err(e) => { panic!("Error setting OnceCell<Mongo>"); },
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], CONFIG.get().unwrap().net.port));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(crate::service::proxy::handle_request))
    });

    let server = Server::bind(&addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(make_svc);

    println!("[ohm] Serving on 127.0.0.1:{}...", CONFIG.get().unwrap().net.port);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn get_config_argument() -> String {
    let args : Vec<String> = env::args().collect();
    match args.len() {
        1 => { // No config argument.
            return "./config/config.toml".to_string();
        },
        2 => {
            return args[2].to_string();
        },
        _ => {
            panic!("Usage: ohm [path/to/custom/config/file]");
        },
    }
}

#[cfg(test)]
mod tests {
   use super::*;
    
   #[tokio::test]
   async fn test_main() -> Result<(), Error> {
        Ok(())
   }
}


