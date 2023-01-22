use mongodb::{Client, Collection, Database, options::ClientOptions, bson::doc, options::FindOptions, options::UpdateOptions};
use futures::stream::TryStreamExt; // Trait required for cursor.try_next().
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use once_cell::sync::OnceCell;
use async_trait::async_trait;

use crate::model::record::Record;
use crate::data::Datastore;

const APP_NAME : &str = "ohm";

// Manage and store all datastore interactions.
pub struct Mongo {
    connection : mongodb::Client,
    db : mongodb::Database,
    record_collection : mongodb::Collection<Record>,
}

#[async_trait]
impl Datastore for Mongo {
    async fn add_traffic(&self, traffic : &crate::Traffic) -> Result<(), Box<dyn std::error::Error>> {
        match self.upsert_traffic(&traffic).await {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl Mongo {

    pub async fn new() -> Self {
        let mut con = Self::get_connection().await.unwrap();
        let mut database = Self::get_database(&con).await.unwrap();
        let mut collection = Self::get_collection(&database).await.unwrap();
        let mut me = Self {
            connection : con, 
            db : database, 
            record_collection : collection, 
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

    async fn get_collection(db : &mongodb::Database) -> Result<mongodb::Collection<Record>, mongodb::error::Error> {

        let collection_name = &crate::CONFIG.get().unwrap().db.collection_name;

        // This is not optimized, but leaving as a skeleton in the event some sort of filtering
        // logic is needed.
        let filter = doc!{ };
        let mut collection : Option<mongodb::Collection<Record>> = None; 
        let collection_names = db.list_collection_names(filter).await?;

        if collection_names.contains(&collection_name.to_string()) {
           collection = Some(db.collection::<Record>(collection_name));
        } else {
            // Note mongoDB creates collections implicitly when data is inserted,
            // so this method is not needed if no special ops are required.
            db.create_collection(collection_name, None).await?;
            collection = Some(db.collection::<Record>(collection_name));
        }
        Ok(collection.unwrap())
    }

    pub async fn upsert_traffic(&self, traffic : &crate::Traffic) -> Result<(), mongodb::error::Error> {
        let record_filter = doc!{ "method": traffic.method.clone(), "host": traffic.host.clone(), "path": traffic.path.clone() };
        let updates = doc!{ "$push": { "traffic": traffic.get_json() } };
        let upsert_options = UpdateOptions::builder().upsert(true).build();
        self.record_collection.update_one(record_filter, updates, upsert_options).await?;
        Ok(())
    }
}
