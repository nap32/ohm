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

// #[derive(...)]
// pub struct N {}
// impl PartialEq for N { fn eq(&self, other:&Self) -> bool {...} }
// impl Eq for N {}
// impl N {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Record {
    pub method : String,
    pub scheme : String,
    pub host : String,
    pub path : String,

    pub traffic : Vec::<crate::Traffic>,
}
impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        (self.method == other.method) &&
            (self.scheme == other.scheme) &&
            (self.host == other.host) &&
            (self.path == other.path)
    }
}
impl Eq for Record {}
impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_json())
    }
}
impl Record {

    pub async fn new(request : hyper::Request<hyper::Body>, response : hyper::Response<hyper::Body>) -> Self {
        // thread 'tokio-runtime-worker' panicked at 'called `Option::unwrap()` on a `None` value',
        // src/record/mod.rs:54:47 -> 55 w/ this comment -> query is 'None'.
        // Do we either set everything to Option(String) or do we parse and pass empty-string?
        let mut me = Self {
            method : match *request.method() {
                Method::GET => "GET".to_string(),
                Method::PUT => "PUT".to_string(),
                Method::POST => "POST".to_string(),
                Method::HEAD => "HEAD".to_string(),
                Method::PATCH => "PATCH".to_string(),
                Method::TRACE => "TRACE".to_string(),
                Method::DELETE => "DELETE".to_string(),
                Method::OPTIONS => "OPTIONS".to_string(),
                Method::CONNECT => "CONNECT".to_string(),
                _ => "?".to_string(),
            },
            scheme : request.uri().scheme().unwrap().to_string(),
            host : request.uri().host().unwrap().to_string(),
            path : request.uri().path().to_string(),
            traffic : Vec::<crate::Traffic>::new(),
        };
        return me 
    }

    pub fn get_key(&self) -> std::string::String {
        let key = format!("{}:{}:{}", self.method, self.host, self.path);
        return key
    }

    pub fn get_url(&self) -> std::string::String {
        let mut url : String = String::from(format!("{}://{}{}", self.scheme, self.host, self.path));
        url
    }

    pub fn get_json(&self) -> std::string::String {
        let serialized = serde_json::to_string(&self).unwrap();
        return serialized
    }

}

#[cfg(test)]
mod tests {
    use super::*; 
    
    #[test]
    fn test_hyper_to_record() -> Result<(), std::io::Error> {
        
        // Use module to convert hyper request+response to record.
        // Use module to convert record to hyper request+hyper response.
        // assert_eq!
        Ok(())
    }

    #[test]
    fn test_record_to_hyper() -> Result<(), std::io::Error> {
        // Use module to convert record to hyper request+response.
        // Use module to convert hyper request+response to record.
        // assert_eq!
        Ok(())
    }

}
