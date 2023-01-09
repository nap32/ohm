use futures::stream::TryStreamExt; // Trait required for cursor.try_next().
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use once_cell::sync::OnceCell;
use async_trait::async_trait;

use crate::model::record::Record;
use crate::data::Datastore;

pub struct Redis {
    pub client : redis::Client,
    pub conn : redis::Connection,
}
#[async_trait]
impl Datastore for Redis {
    async fn add_traffic(&self, traffic : &crate::Traffic) -> Result<(), Box<dyn std::error::Error>> {
        self.insert_traffic(traffic).await.unwrap();
        Ok(())
    }
}
impl Redis {
    pub async fn new() -> Self {
        let db_url = &crate::CONFIG.get().unwrap().db.db_url;
        let client = redis::Client::open(db_url.clone()).unwrap();
        let mut conn = client.get_connection().unwrap();
        Self {
            client,
            conn
        }
    }

    pub async fn insert_traffic(&self, traffic : &crate::Traffic) -> Result<(), Box<dyn std::error::Error>> {
        let key = self.get_traffic_key(traffic).await.unwrap();
        let mut conn = self.client.get_connection().unwrap();
        let _ : () = redis::cmd("SET").arg(key).arg(traffic.to_string()).query(&mut conn).unwrap();
        Ok(())
    }

    pub async fn get_traffic_key(&self, traffic : &crate::Traffic) -> Result<String, std::io::Error> {
        let mut key = String::new();
        key.push_str(&traffic.method);
        key.push(':');
        key.push_str(&traffic.host);
        key.push(':');
        key.push_str(&traffic.path);
        Ok(key)
    }
}
