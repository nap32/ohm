use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub net : Net,
    pub ca : Ca,
    pub db : Db,
    pub filter : Filter,
}

#[derive(Serialize, Deserialize)]
pub struct Net {
    pub port : u16,
}

#[derive(Serialize, Deserialize)]
pub struct Ca {
    pub pem_relative_path : String,
    pub key_relative_path : String,
}

#[derive(Serialize, Deserialize)]
pub struct Db {
    pub db_url : String,
    pub app_name : String,
    pub db_name : String,
    pub traffic_collection_name : String,
    pub auth_collection_name : String,
}

#[derive(Serialize, Deserialize)]
pub struct Filter {
    pub allow_list_hosts : Vec<String>,
    pub deny_list_hosts : Vec<String>,
    pub identity_providers : Vec<String>,
}

impl Config {
    pub async fn new(config_path : String) -> Self {
        let config_string = std::fs::read_to_string(config_path).unwrap();
        let config_toml : Config = toml::from_str(&config_string).unwrap();
        Self {
            net : config_toml.net,
            ca : config_toml.ca,
            db : config_toml.db,
            filter: config_toml.filter,
        }
    }
}
