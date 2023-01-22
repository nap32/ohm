use mongodb::{Client, Collection, Database, options::ClientOptions, bson::doc, options::FindOptions, options::UpdateOptions};
use futures::stream::TryStreamExt; // Trait required for cursor.try_next().
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use once_cell::sync::OnceCell;
use async_trait::async_trait;
use mongodb::bson;

use crate::model::record::Record;
use crate::model::traffic::Traffic;
use crate::model::auth::AuthInfo;
use crate::data::Datastore;

const APP_NAME : &str = "ohm";

// Manage and store all datastore interactions.
pub struct Mongo {
    connection : mongodb::Client,
    db : mongodb::Database,
    record_collection : mongodb::Collection<Record>,
    traffic_collection : mongodb::Collection<Traffic>,
    auth_collection : mongodb::Collection<AuthInfo>,
}

#[async_trait]
impl Datastore for Mongo {
    async fn add_traffic(&self, traffic : &crate::Traffic) -> Result<(), Box<dyn std::error::Error>> {
        match self.insert_traffic(&traffic).await {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl Mongo {

    pub async fn new() -> Self {
        let mut con = Self::get_connection().await.unwrap();
        let mut database = Self::get_database(&con).await.unwrap();
        let mut record_collection = Self::get_record_collection(&database).await.unwrap();
        let mut traffic_collection = Self::get_traffic_collection(&database).await.unwrap();
        let mut auth_collection = Self::get_auth_collection(&database).await.unwrap();
        let mut me = Self {
            connection : con, 
            db : database, 
            record_collection,
            traffic_collection,
            auth_collection,
        };
        return me
    }

    async fn get_connection() -> Result<mongodb::Client, mongodb::error::Error>{

        let db_url = &crate::CONFIG.get().unwrap().db.db_url;

        // Parse a connection string into an options struct.
        let mut client_options = ClientOptions::parse(db_url).await?;
        client_options.app_name = Some(APP_NAME.to_string());
        let client = Client::with_options(client_options)?;

        Ok(client)
    }

    async fn get_database(client : &mongodb::Client) -> Result<mongodb::Database, mongodb::error::Error>{
        let db_name = &crate::CONFIG.get().unwrap().db.db_name;
        let db = Some(client.database(db_name)); 
        Ok(db.unwrap())
    }

    async fn get_record_collection(db : &mongodb::Database) -> Result<mongodb::Collection<Record>, mongodb::error::Error> {
        let collection_name = &crate::CONFIG.get().unwrap().db.record_collection_name;
        Ok(db.collection::<Record>(collection_name))
    }

    async fn get_traffic_collection(db : &mongodb::Database) -> Result<mongodb::Collection<Traffic>, mongodb::error::Error> {
        let collection_name = &crate::CONFIG.get().unwrap().db.traffic_collection_name;
        Ok(db.collection::<Traffic>(collection_name))
    }

    async fn get_auth_collection(db : &mongodb::Database) -> Result<mongodb::Collection<AuthInfo>, mongodb::error::Error> {
        let collection_name = &crate::CONFIG.get().unwrap().db.auth_collection_name;
        Ok(db.collection::<AuthInfo>(collection_name))
    }


    pub async fn insert_record(&self, record : &crate::Record) -> Result<(), mongodb::error::Error> {
        self.record_collection.insert_one(record, None).await?;
        Ok(())
    }

    pub async fn insert_traffic(&self, traffic : &crate::Traffic) -> Result<(), mongodb::error::Error> {
        self.traffic_collection.insert_one(traffic, None).await?;
        Ok(())
    }

    pub async fn insert_auth(&self, auth : &crate::AuthInfo) -> Result<(), mongodb::error::Error> {
        self.auth_collection.insert_one(auth, None).await?;
        Ok(())
    }

    pub async fn upsert_traffic_to_record(&self, traffic : &crate::Traffic) -> Result<(), mongodb::error::Error> {
        let record_filter = doc!{ "method": traffic.method.clone(), "host": traffic.host.clone(), "path": traffic.path.clone() };
        let updates = doc!{ "$push": { "traffic": traffic.get_json() } };
        let upsert_options = UpdateOptions::builder().upsert(true).build();
        self.record_collection.update_one(record_filter, updates, upsert_options).await?;
        Ok(())
    }

}
