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

///
/// REQUEST
///
    
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct RequestRecord {
        pub method : String,
        pub scheme : String,
        pub host : String,
        pub path : String,
        pub query : String,
        pub headers : HashMap<String, String>, // std::collections::HashMap;
        pub body : String, // Is this best as String or Vec<u8> byte-string?
    }
    impl PartialEq for RequestRecord {
        fn eq(&self, other: &Self) -> bool {
            (self.method == other.method) &&
                (self.scheme == other.scheme) &&
                (self.host == other.host) &&
                (self.path == other.path) &&
                (self.query == other.query) &&
                (self.headers == other.headers) &&
                (self.body == other.body)
        }
    }
    impl Eq for RequestRecord {}
    impl RequestRecord {
        pub fn new(request : hyper::Request<hyper::Body>) -> Self {
            let (parts, _body) = request.into_parts();
            let me = Self {
                method : "GET".to_string(),
                scheme : parts.uri.scheme().unwrap().to_string(),   // I think these .unwrap()'s are
                host : parts.uri.host().unwrap().to_string(),       // are going to cause trouble.
                path : parts.uri.path().to_string(),
                query : parts.uri.query().unwrap().to_string(),
                headers : HashMap::<std::string::String, std::string::String>::new(),
                body : "Foobar".to_string(),
            };
            return me;
        }
        pub fn get_uri(&self) -> std::string::String {
            let uri = format!("{}://{}/{}", self.scheme, self.host, self.path); 
            return uri
        }
        pub fn get_json(&self) -> std::string::String {
            let serialized = serde_json::to_string(&self).unwrap();
            return serialized
        }
    }

///
/// RESPONSE
///

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ResponseRecord {
        pub status: String,
        pub headers : HashMap<String, String>, // std::collections::HashMap;
        pub body : String, // Is this best as String or Vec<u8> byte-string?
    }
    impl PartialEq for ResponseRecord {
        fn eq(&self, other: &Self) -> bool {
            (self.status == other.status) &&
                (self.headers == other.headers) &&
                (self.body == other.body)
        }
    }
    impl Eq for ResponseRecord {}
    impl ResponseRecord {
        pub fn new(response : hyper::Response<hyper::Body>) -> Self {
            let mut me = Self {
                status : response.status().to_string().clone(),
                headers : HashMap::<std::string::String, std::string::String>::new(),
                body : "Foobar".to_string(),
            };
            for (key, value) in response.headers() {
                me.headers.insert(key.to_string(), std::string::String::from(value.to_str().unwrap()));
            }
            return me
        }
        pub fn get_json(&self) -> std::string::String {
            let serialized = serde_json::to_string(&self).unwrap();
            return serialized
        }
    }

///
/// RECORD
///

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Record {
        pub uri : String,
        pub request : record::RequestRecord,
        pub response : record::ResponseRecord,
    }
    impl PartialEq for Record {
        fn eq(&self, other: &Self) -> bool {
            (self.uri == other.uri) &&
                (self.request == other.request) &&
                (self.response == other.response)
        }
    }
    impl Eq for Record {}
    impl Record {
        pub fn new(request : hyper::Request<hyper::Body>, response : hyper::Response<hyper::Body>) -> Self {
            let ohm_request = record::RequestRecord::new(request);
            let ohm_response = record::ResponseRecord::new(response);
            let me = Self {
                uri : ohm_request.get_uri(),
                request : ohm_request,
                response : ohm_response,
            };
            return me 
        }
        pub fn get_key(&self) -> std::string::String {
            let key = format!("{}:{}:{}", self.request.method, self.request.host, self.request.path);
            return key
        }
        pub fn get_uri(&self) -> std::string::String {
            return self.request.get_uri()
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
