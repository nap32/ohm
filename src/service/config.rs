use serde_yaml;

pub struct Config {
    // SERVER
    port                : i32,
    // CA
    pem_relative_path   : String,
    key_relative_path   : String,
    // DATA
    db_url : String,
    app_name : String,
    db_name : String,
    collection_name : String,
}

impl Config {
    pub async fn new() -> Self {
        Self {
            // SERVER
            port : 8080,
            // CA
            pem_relative_path : "../../ca/ohm.pem".to_string(),
            key_relative_path : "../../ca/ohm.key".to_string(),
            // DATA
            db_url : "mongodb://localhost:27017".to_string(),
            app_name : "Ohm".to_string(),
            db_name : "records".to_string(),
            collection_name : "records".to_string(),
        }
    }
}
