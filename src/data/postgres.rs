
use crate::model::traffic::Traffic;
use crate::data::Datastore;

use tokio_postgres::Error;
use postgres_openssl::MakeTlsConnector;
use openssl::ssl::{SslConnector, SslMethod};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct Postgres {
    pub client : tokio_postgres::Client,
}

#[async_trait]
impl Datastore for Postgres {
    async fn add_traffic(&self, traffic : &crate::Traffic) -> Result<(), Box<dyn std::error::Error>> {
        match self.insert_traffic(traffic).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
    async fn add_authinfo(&self, auth : &crate::AuthInfo) -> Result<(), Box<dyn std::error::Error>> {
        // TO-DO.
        Ok(())
    }
}

impl Postgres {

    pub async fn new() -> Self {
        let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
        builder.set_ca_file("database_cert.pem").unwrap();
        let connector = MakeTlsConnector::new(builder.build());
        let connect_future = tokio_postgres::connect(
            "host=localhost user=postgres sslmode=require",
            connector,
        );

        let (client, connection) = connect_future.await.unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        Self {
            client
        }
    }

    pub async fn create_table(client: tokio_postgres::Client) -> Result<(), tokio_postgres::Error> {
        client.batch_execute("
            CREATE TABLE traffic (
                method                  TEXT
                scheme                  TEXT
                host                    TEXT
                path                    TEXT
                query                   TEXT
                request_headers         TEXT[2][]
                request_body            BYTEA
                request_body_string     TEXT
                status                  TEXT
                response_headers        TEXT[2][]
                response_body           BYTEA
                response_body_string    TEXT
                version                 TEXT
            )
        ").await?;
        Ok(())
    }

    pub async fn insert_traffic(&self, traffic: &Traffic) -> Result<(), tokio_postgres::Error> {
        // '{ { KEY, VAL }, { KEY, VAL } , { KEY, VAL } }' to insert array - utility function used.
        self.client.execute(
            "INSERT INTO traffic (method, scheme, host, path, query, request_headers, request_body, request_body_string, status, response_headers, response_body, response_body_string, version) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
            &[  &traffic.method,
                &traffic.scheme,
                &traffic.host,
                &traffic.path,
                &traffic.query,
                &Postgres::prepare_tuple_array(&traffic.request_headers).await.unwrap(),
                &traffic.request_body,
                &traffic.request_body_string,
                &traffic.status.to_string(),
                &Postgres::prepare_tuple_array(&traffic.response_headers).await.unwrap(),
                &traffic.response_body,
                &traffic.response_body_string,
                &traffic.version.to_string()
            ],
        ).await?;
        Ok(())
    }

    pub async fn prepare_tuple_array(tuple_array: &HashMap<String, String>) -> Result<String, std::io::Error> {
        let mut prepared_string = String::new();
        prepared_string.push('{');
        for (key, value) in tuple_array.iter() {
            // '{' and '}' need escaped by doubles - "{{" and "}}". Alternative is r###"foobar"###.
            prepared_string.push_str(format!("{{{},{}}}", key, value).as_str());
        }
        prepared_string.push('}');
        Ok(prepared_string)
    }

}

