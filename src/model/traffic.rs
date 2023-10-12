use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    pub method: String,
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub version: String,
}

impl Request {
    pub async fn new(
        method: String,
        scheme: String,
        host: String,
        path: String,
        query: HashMap<String, String>,
        headers: HashMap<String, String>,
        body: String,
        version: String,
    ) -> Self {
        Self {
            method,
            scheme,
            host,
            path,
            query,
            headers,
            body,
            version,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub version: String,
}

impl Response {
    pub async fn new(
        status: u16,
        headers: HashMap<String, String>,
        body: String,
        version: String,
    ) -> Self {
        Self {
            status,
            headers,
            body,
            version,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Traffic {
    pub request: Request,
    pub response: Response,
}

impl Traffic {
    pub async fn new(request: Request, response: Response) -> Self {
        Self { request, response }
    }
}

// Ensure we can get the URL and get the JSON.

//impl Url
//    pub fn get_url(&self) -> std::string::String {
//        let mut url: String = format!("{}://{}{}", self.scheme, self.host, self.path);
//        if !self.query.is_empty() {
//            url.push('?');
//            url.push_str(&self.query);
//        }
//        url
//    }
//
//    pub fn get_json(&self) -> std::string::String {
//        serde_json::to_string(&self).unwrap()
//    }
//}
