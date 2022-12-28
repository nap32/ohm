
use crate::model::record::Record;
use crate::model::traffic::Traffic;

use tokio_postgres::Error;
use postgres_openssl::MakeTlsConnector;
use openssl::ssl::{SslConnector, SslMethod};

pub struct Postgres {

}

impl Postgres {

    pub async fn new() -> Result<(), tokio_postgres::Error> {
        let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
        builder.set_ca_file("database_cert.pem").unwrap();
        let connector = MakeTlsConnector::new(builder.build());
        let connect_future = tokio_postgres::connect(
            "host=localhost user=postgres sslmode=require",
            connector,
        );
        let (client, connection) = connect_future.await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });
        // Set the client somewhere.
        Ok(())
    }

    pub async fn create_table(client: tokio_postgres::Client) -> Result<(), tokio_postgres::Error> {
        client.batch_execute("
            CREATE TABLE traffic (
                method              TEXT
                scheme              TEXT
                host                TEXT
                path                TEXT
                query               TEXT
                request_headers     TEXT[2][]
                request_body        BYTEA
                response_headers    TEXT[2][]
                response_body       BYTEA
                status              INT
            )
        ").await?;
        Ok(())
    }

    pub async fn query_traffic(client: tokio_postgres::Client) -> Result<(), tokio_postgres::Error> {
        let name = "Foobar";
        let data = None::<&[u8]>;

        for row in client.query("SELECT id, name, data FROM person", &[]).await? {
            let id: i32 = row.get(0);
            let name: &str = row.get(1);
            let data: Option<&[u8]> = row.get(2);
            println!("found person: {} {} {:?}", id, name, data);
        }

        Ok(())
    }

    pub async fn insert_traffic(client: tokio_postgres::Client, traffic: Traffic) -> Result<(), tokio_postgres::Error> {
        client.execute(
            "INSERT INTO traffic (method, scheme, host, path, query, request_headers, request_body, response_headers, response_body, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10))",
            &[&traffic.method, &traffic.scheme, &traffic.host, &traffic.path, &traffic.query, &traffic.request_headers[&"Host".to_string()], &traffic.request_body, &traffic.response_headers[&"Host".to_string()], &traffic.response_body, &traffic.status.to_string() ],
        ).await?;
        Ok(())
    }
}
