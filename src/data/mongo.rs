use mongodb::{Client, options::ClientOptions, bson::doc};
use async_trait::async_trait;

use crate::model::traffic::Traffic;
use crate::model::auth::AuthInfo;
use crate::data::Datastore;

const APP_NAME : &str = "ohm";

// Manage and store all datastore interactions.
pub struct Mongo {
    connection : mongodb::Client,
    db : mongodb::Database,
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
    async fn add_authinfo(&self, auth : &crate::AuthInfo) -> Result<(), Box<dyn std::error::Error>> {
        match self.insert_auth(&auth).await {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl Mongo {

    pub async fn new() -> Self {
        let con = Self::get_connection().await.unwrap();
        let database = Self::get_database(&con).await.unwrap();
        let traffic_collection = Self::get_traffic_collection(&database).await.unwrap();
        let auth_collection = Self::get_auth_collection(&database).await.unwrap();
        let me = Self {
            connection : con, 
            db : database, 
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

    async fn get_traffic_collection(db : &mongodb::Database) -> Result<mongodb::Collection<Traffic>, mongodb::error::Error> {
        let collection_name = &crate::CONFIG.get().unwrap().db.traffic_collection_name;
        Ok(db.collection::<Traffic>(collection_name))
    }

    async fn get_auth_collection(db : &mongodb::Database) -> Result<mongodb::Collection<AuthInfo>, mongodb::error::Error> {
        let collection_name = &crate::CONFIG.get().unwrap().db.auth_collection_name;
        Ok(db.collection::<AuthInfo>(collection_name))
    }


    pub async fn insert_traffic(&self, traffic : &crate::Traffic) -> Result<(), mongodb::error::Error> {
        self.traffic_collection.insert_one(traffic, None).await?;
        Ok(())
    }

    pub async fn insert_auth(&self, _auth : &crate::AuthInfo) -> Result<(), mongodb::error::Error> {
        let filter = doc! {
            "issuer": stringify!(auth.issuer),
            "grant_type": stringify!(auth.grant_type),
            "client_id": stringify!(auth.client_id),
            "redirect_url": stringify!(auth.redirect_url),
            "scope": stringify!(auth.scope),
        };
        let update = doc!{
            "issuer": stringify!(auth.issuer),
            "grant_type": stringify!(auth.grant_type),
            "client_id": stringify!(auth.client_id),
            "redirect_url": stringify!(auth.redirect_url),
            "scope": stringify!(auth.scope),
        };
        let options = mongodb::options::FindOneAndUpdateOptions::builder()
        .upsert(Some(true))
        .build();
        self.auth_collection.find_one_and_update(filter, update, Some(options)).await?;
        Ok(())
    }

}
