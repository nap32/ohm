pub mod model;
use crate::model::auth::AuthInfo;
use crate::model::traffic::Traffic;

pub mod data;
use crate::data::mongo::Mongo;

pub mod service;
use crate::service::config::Config;
use crate::service::filter::Filter;

use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use once_cell::sync::OnceCell;

static CONFIG: OnceCell<Config> = OnceCell::new();
static DATASTORE_CLIENT: OnceCell<Mongo> = OnceCell::new();
static FILTER_CHAIN: OnceCell<Filter> = OnceCell::new();

#[tokio::main]
async fn main() {
    match CONFIG.set(Config::new(get_config_argument().await).await) {
        Ok(()) => (),
        Err(_e) => {
            panic!("Error setting Config.");
        }
    }
    match DATASTORE_CLIENT.set(Mongo::new().await) {
        Ok(()) => (),
        Err(_e) => {
            panic!("Error setting Mongo");
        }
    };
    match FILTER_CHAIN.set(Filter::new().await) {
        Ok(()) => (),
        Err(_e) => {
            panic!("Error setting Filter.");
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], CONFIG.get().unwrap().net.port));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(crate::service::proxy::handle_request))
    });

    let server = Server::bind(&addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(make_svc);

    println!(
        "[ohm] Serving on 127.0.0.1:{}...",
        CONFIG.get().unwrap().net.port
    );

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn get_config_argument() -> String {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            // No config argument.
            "./config/config.toml".to_string()
        }
        2 => args[1].to_string(),
        _ => {
            panic!("Usage: ohm [path/to/custom/config/file]");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    #[tokio::test]
    async fn test_server_creation() -> Result<(), Error> {
        let addr = SocketAddr::from(([127, 0, 0, 1], 8085));

        let make_svc = make_service_fn(|_conn| async {
            Ok::<_, Infallible>(service_fn(crate::service::proxy::handle_request))
        });

        let _ = Server::bind(&addr)
            .http1_preserve_header_case(true)
            .http1_title_case_headers(true)
            .serve(make_svc);

        // TODO: .await( ) -> Close it down.

        Ok(())
    }
}
