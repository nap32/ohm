use mongodb::{Client, Collection, Database, options::ClientOptions, bson::doc, options::FindOptions, options::UpdateOptions};
use futures::stream::TryStreamExt; // Trait required for cursor.try_next().
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use once_cell::sync::OnceCell;

use crate::model::record::Record;

const DB_URL : &str = "mongodb://localhost:27017";
const APP_NAME : &str = "Ohm";
const DB_NAME : &str = "records";
const COLLECTION_NAME : &str = "records";

// Manage and store all datastore interactions.
pub struct Mongo {
    connection : mongodb::Client,
    db : mongodb::Database,
    record_collection : mongodb::Collection<Record>,
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
        // Parse a connection string into an options struct.
        let mut client_options = ClientOptions::parse(DB_URL).await?;

        // Manually set an option.
        client_options.app_name = Some(APP_NAME.to_string());

        let client = Client::with_options(client_options)?;

        Ok(client)
    }

    async fn get_database(client : &mongodb::Client) -> Result<mongodb::Database, mongodb::error::Error>{

        // Get DBs.
        let mut db : Option<mongodb::Database> = None;  
        let mut dbs = HashMap::<String, Option<mongodb::Database>>::new();
        for db_name in client.list_database_names(None, None).await? {
            dbs.insert(db_name, None);
        };

        // Check for 'records' DB, create if missing.
        if dbs.contains_key(DB_NAME) && dbs.get(DB_NAME).is_some() {
            db = Some(client.database(DB_NAME)); 
        } else {
            // I think you can create a database with the same verbiage,
            // so this block is redundant.
            db = Some(client.database(DB_NAME));
        }
        
        // If no value is here will throw an error via .unwrap(), and return E instead of V.
        Ok(db.unwrap())
    }

    async fn get_collection(db : &mongodb::Database) -> Result<mongodb::Collection<Record>, mongodb::error::Error> {
        let filter = doc!{ };
        let mut collection : Option<mongodb::Collection<Record>> = None; 
        let collection_names = db.list_collection_names(filter).await?;

        if collection_names.contains(&COLLECTION_NAME.to_string()) {
           collection = Some(db.collection::<Record>(COLLECTION_NAME));
        } else {
            // Note mongoDB creates collections implicitly when data is inserted,
            // so this method is not needed if no special ops are required.
            db.create_collection(COLLECTION_NAME, None).await?;
            collection = Some(db.collection::<Record>(COLLECTION_NAME));
        }
        Ok(collection.unwrap())
    }
}

pub async fn upsert_traffic(mongo : &Mongo, traffic : &crate::Traffic) -> Result<(), mongodb::error::Error> {
    let record_filter = doc!{ "method": traffic.method.clone(), "host": traffic.host.clone(), "path": traffic.path.clone() };
    let updates = doc!{ "$push": { "traffic": traffic.get_json() } };
    let upsert_options = UpdateOptions::builder().upsert(true).build();
    mongo.record_collection.update_one(record_filter, updates, upsert_options).await?;
    Ok(())
}

async fn add_record(mongo : &Mongo, record: &Record) -> Result<(), mongodb::error::Error> {
    mongo.record_collection.insert_one(record, None).await?;
    Ok(())
}

async fn add_records(mongo : &Mongo, records: &Vec<Record>) -> Result<(), mongodb::error::Error> {
    mongo.record_collection.insert_many(records, None).await?;
    Ok(())
}

async fn get_records(mongo : &Mongo, record_filter : mongodb::bson::Document) -> Result<Vec::<Record>, mongodb::error::Error> {

    let mut results = Vec::<Record>::new();
    //let find_options = FindOptions::builder().sort(doc!{ "method" : 1 }).build();
    let mut cursor = mongo.record_collection.find(record_filter, None).await?;

    while let Some(record) = cursor.try_next().await? {
        results.push(record);
    }

    Ok(results)
}

async fn update_records(mongo : &Mongo, record_filter : mongodb::bson::Document, updates_filter : mongodb::bson::Document) -> Result<(), mongodb::error::Error> {
    mongo.record_collection.update_many(record_filter, updates_filter, None).await?;
    Ok(())
}


async fn delete_records(mongo : &Mongo, record_filter : mongodb::bson::Document) -> Result<(), mongodb::error::Error> {
    mongo.record_collection.delete_many(record_filter, None).await?;
    Ok(())
}

async fn drop_collection(mongo : &Mongo) -> Result<(), mongodb::error::Error> {
    let _result = mongo.record_collection.drop(None).await?;
    Ok(())
}

async fn drop_database(mongo : &Mongo) -> Result<(), mongodb::error::Error> {
    let _options = mongodb::options::DropDatabaseOptions::builder().build();
    let _result = mongo.db.drop(None).await?;
    Ok(())
}
