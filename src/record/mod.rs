#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_assignments)]

// ABOUT -
// Proof of concept for working with hyper's library and transforming data.
// Need to define the appropriate structs and implement / define necessary
// traits to support robust and convenient interactions.
// PLAN -
// Use serde and serde_json w/ RedisJSON and RedisSearch.
mod record {

    use crate::record::record;
    use std::collections::HashMap;
    use serde::{Serialize, Deserialize};
    use serde_json::Value;
    use hyper::{Request, Response, Body, StatusCode, Method};

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
        pub query : String,
        pub request_headers : HashMap<String, String>, 
        pub request_body : Vec<u8>,
        pub status: String,
        pub response_headers : HashMap<String, String>, 
        pub response_body : Vec<u8>, 
    }
    impl PartialEq for Record {
        fn eq(&self, other: &Self) -> bool {
            (self.method == other.method) &&
                (self.scheme == other.scheme) &&
                (self.host == other.host) &&
                (self.path == other.path) &&
                (self.query == other.query) &&
                (self.request_headers == other.request_headers) &&
                (self.request_body == other.request_body) &&
                (self.status == other.status) &&
                (self.response_headers == other.response_headers) &&
                (self.response_body == other.response_body)
        }
    }
    impl Eq for Record {}
    impl Record {
        pub fn new(request : hyper::Request<hyper::Body>, response : hyper::Response<hyper::Body>) -> Self {
            let mut me = Self {
                method : "GET".to_string(),
                scheme : request.uri().scheme().unwrap().to_string(),   // I think these .unwrap()'s are
                host : request.uri().host().unwrap().to_string(),       // are going to cause trouble.
                path : request.uri().path().to_string(),
                query : request.uri().query().unwrap().to_string(),
                request_headers : HashMap::<std::string::String, std::string::String>::new(),
                request_body : Vec::<u8>::new(),
                status : response.status().to_string().clone(),
                response_headers : HashMap::<std::string::String, std::string::String>::new(),
                response_body : Vec::<u8>::new(),
            };
            for (key, value) in request.headers() {
                me.request_headers.insert(key.to_string(), std::string::String::from(value.to_str().unwrap()));
            }
            for (key, value) in response.headers() {
                me.response_headers.insert(key.to_string(), std::string::String::from(value.to_str().unwrap()));
            }
            //me.request_body = hyper::body::to_bytes(request.into_body()).await.unwrap().to_vec();
            //me.response_body = hyper::body::to_bytes(request.into_body()).await.unwrap().to_vec();
            return me 
        }
        pub fn get_key(&self) -> std::string::String {
            let key = format!("{}:{}:{}", self.method, self.host, self.path);
            return key
        }
        pub fn get_uri(&self) -> std::string::String {
            return format!("{}://{}/{}", self.scheme, self.host, self.path); 
        }
        pub fn get_json(&self) -> std::string::String {
            let serialized = serde_json::to_string(&self).unwrap();
            return serialized
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Imports names from outer (for mod tests) scope.
    
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
