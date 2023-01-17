#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_assignments)]

use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use hyper::{Request, Response, Body, StatusCode, Method};
use flate2::read::GzDecoder;
use std::io::prelude;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Record {
    pub method      : String,
    pub host        : String,
    pub path        : String,
    pub auth        : Option<crate::AuthInfo>,
    pub traffic     : Vec::<crate::Traffic>,
}
impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        (self.method == other.method) &&
            (self.host == other.host) &&
            (self.path == other.path)
    }
}
impl Eq for Record {}
impl fmt::Display for Record {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_json())
    }
}
impl Record {

    pub async fn new(traffic: &mut crate::model::traffic::Traffic) -> Self {
        let mut me = Self {
            method  :   traffic.method.clone(),
            host    :   traffic.host.clone(),
            path    :   traffic.path.clone(),
            auth    :   None,
            traffic :   vec![traffic.clone()], 
        };
        return me 
    }

    pub fn get_key(&self) -> std::string::String {
        let key = format!("{}:{}:{}", self.method, self.host, self.path);
        return key
    }

    pub fn get_url(&self) -> std::string::String {
        let mut url : String = String::from(format!("{}://{}{}", "https", self.host, self.path));
        url
    }

    pub fn get_json(&self) -> std::string::String {
        let serialized = serde_json::to_string(&self).unwrap();
        return serialized
    }

}
