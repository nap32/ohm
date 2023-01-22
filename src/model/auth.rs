use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use hyper::{Request, Response, Body, StatusCode, Method};
use flate2::read::GzDecoder;
use std::io::prelude;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthInfo {
    pub grant_type      : String,
    pub issuer          : String,
    pub client_id       : String,
    pub redirect_url    : String,

    pub token_format    : String,
    pub token_key       : String,
    pub token_val       : String,
}
impl PartialEq for AuthInfo {
    fn eq(&self, other: &Self) -> bool {
        (self.grant_type == other.grant_type) &&
            (self.issuer == other.issuer) &&
            (self.client_id == other.client_id) &&
            (self.redirect_url == other.redirect_url) &&
            (self.token_format == other.token_format) &&
            (self.token_key == other.token_key) &&
            (self.token_val == other.token_val)
    }
}
impl Eq for AuthInfo {}
impl fmt::Display for AuthInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_json())
    }
}
impl AuthInfo {
    pub fn new() -> Self {
        Self {
            grant_type   : String::default(),
            issuer       : String::default(),
            client_id    : String::default(),
            redirect_url : String::default(),
            token_format : String::default(),
            token_key    : String::default(),
            token_val    : String::default(),
        }
    }
    pub fn get_json(&self) -> std::string::String {
        let serialized = serde_json::to_string(&self).unwrap();
        return serialized
    }
}
