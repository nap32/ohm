pub mod datastore {

    use mongodb::{Client, Collection, Database, options::ClientOptions, bson::doc, options::FindOptions};
    use futures::stream::TryStreamExt; // Trait required for cursor.try_next().
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    use crate::datastore::datastore;

    use crate::record::record;
    use crate::Record;

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
            let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;

            // Manually set an option.
            client_options.app_name = Some("ohm".to_string());

            let client = Client::with_options(client_options)?;

            for db_name in client.list_database_names(None, None).await? {
                println!("{}", db_name);
            };

            Ok(client)
        }

        async fn get_database(client : &mongodb::Client) -> Result<mongodb::Database, mongodb::error::Error>{

            // Get DBs.
            let mut db : Option<mongodb::Database> = None;  
            let mut dbs = HashMap::<String, Option<mongodb::Database>>::new();
            for db_name in client.list_database_names(None, None).await? {
                println!("{}", db_name);
                dbs.insert(db_name, None);
            };

            // Check for 'records' DB, create if missing.
            if dbs.contains_key("records") && dbs.get("records").is_some() {
                db = Some(client.database("records")); 
            } else {
                // I think you can create a database with the same verbiage,
                // so this block is redundant.
                db = Some(client.database("records"));
            }
            
            // If no value is here will throw an error via .unwrap(), and return E instead of V.
            Ok(db.unwrap())
        }

        async fn get_collection(db : &mongodb::Database) -> Result<mongodb::Collection<Record>, mongodb::error::Error> {
            let filter = doc!{ };
            let mut collection : Option<mongodb::Collection<Record>> = None; 
            let collection_names = db.list_collection_names(filter).await?;
            for name in &collection_names {
                println!("{}", name);
            }

            let collection_name : String = "records".to_string();
            if collection_names.contains(&collection_name) {
               collection = Some(db.collection::<Record>("records"));
            } else {
                // Note mongoDB creates collections implicitly when data is inserted,
                // so this method is not needed if no special ops are required.
                db.create_collection(&collection_name, None).await?;
                collection = Some(db.collection::<Record>("records"));

            }
            Ok(collection.unwrap())
        }

        pub async fn write_records(&mut self, records: Vec<Record>) -> Result<(), mongodb::error::Error> {
            self.record_collection.insert_many(records, None).await?;
            Ok(())
        }

        pub async fn read_records(&mut self) -> Result<(), mongodb::error::Error> {

            let filter = doc!{ "host" : "foo.bar" };
            let find_options = FindOptions::builder().sort(doc!{ "method" : 1 }).build();
            let mut cursor = self.record_collection.find(filter, find_options).await?;

            while let Some(record) = cursor.try_next().await? {
                println!("method: {}", record.method);
                println!("host: {}", record.host);
                println!("path: {}", record.path);
            }

            Ok(())
        }

        pub async fn update_records(&mut self) -> Result<(), mongodb::error::Error> {
            let filter = doc!{ "host" : "foo.bar" };
            let updates = doc!{ "$set" : { "host" : "bar.foo" } };
            self.record_collection.update_many(filter, updates, None).await?;
            Ok(())
        }

        pub async fn delete_records(&mut self) -> Result<(), mongodb::error::Error> {
            let filter = doc!{ "host" : "bar.foo" };
            self.record_collection.delete_many(filter, None).await?;
            Ok(())
        }

        pub async fn drop_collection(&mut self) -> Result<(), mongodb::error::Error> {
            let _result = self.record_collection.drop(None).await?;
            Ok(())
        }

        pub async fn drop_database(&mut self) -> Result<(), mongodb::error::Error> {
            let _options = mongodb::options::DropDatabaseOptions::builder().build();
            let _result = self.db.drop(None).await?;
            Ok(())
        }

    }

}
