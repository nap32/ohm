use crate::Traffic;
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthInfo {
    pub grant_type      : String,
    pub issuer          : String,
    pub client_id       : String,
    pub redirect_url    : String,
    pub scope           : String,
}
impl PartialEq for AuthInfo {
    fn eq(&self, other: &Self) -> bool {
        (self.grant_type == other.grant_type) &&
            (self.issuer == other.issuer) &&
            (self.client_id == other.client_id) &&
            (self.scope == other.scope) &&
            (self.redirect_url == other.redirect_url)
    }
}
impl Eq for AuthInfo {}
impl fmt::Display for AuthInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_json())
    }
}
impl AuthInfo {

    pub fn new(traffic : &mut Traffic) -> Self {
        let query_pairs = traffic.get_query_map();

        let mut grant_type = String::default();
        let mut client_id = String::default();
        let mut redirect_url = String::default();
        let mut scope = String::default();

        let issuer = traffic.host.clone();

        if query_pairs.contains_key("response_type") {
            grant_type = query_pairs.get("response_type").unwrap().to_string();
        }
        if query_pairs.contains_key("client_id") {
            client_id = query_pairs.get("client_id").unwrap().to_string();
        }
        if query_pairs.contains_key("scope") {
            scope = query_pairs.get("scope").unwrap().to_string();
        }

        if traffic.response_headers.contains_key("location") {
            match traffic.response_headers.get("location") {
                Some(val) => {
                    println!("{}", val);
                    redirect_url = val.to_string();
                },
                None => { }
            }
        }

        Self {
            issuer,
            grant_type,
            client_id,
            redirect_url,
            scope,
        }
    }

    pub fn get_json(&self) -> std::string::String {
        let serialized = serde_json::to_string(&self).unwrap();
        return serialized
    }
}
