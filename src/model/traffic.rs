use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};
use hyper::Method;
use flate2::read::GzDecoder;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Traffic {
    pub method : String,
    pub scheme : String,
    pub host : String,
    pub path : String,
    pub query : String,
    pub request_headers : HashMap<String, String>, 
    pub request_body : Vec<u8>,
    pub request_body_string : Option<String>,
    pub status: u16,
    pub response_headers : HashMap<String, String>, 
    pub response_body : Vec<u8>,
    pub response_body_string : Option<String>,
    pub version : String,
}
impl PartialEq for Traffic {
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
impl Eq for Traffic {}
impl fmt::Display for Traffic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_json())
    }
}
impl Traffic {

    pub async fn new(request : hyper::Request<hyper::Body>, response : hyper::Response<hyper::Body>) -> Self {
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
            query : match request.uri().query(){
                Some(q) => q.to_string(),
                None => "".to_string(),
            },
            request_headers : HashMap::<std::string::String, std::string::String>::new(),
            request_body : Vec::<u8>::new(),
            request_body_string : None,
            status : response.status().as_u16(),
            response_headers : HashMap::<std::string::String, std::string::String>::new(),
            response_body : Vec::<u8>::new(),
            response_body_string : None,
            version : match request.version() {
                hyper::Version::HTTP_2 => "HTTP/2.0".to_string(),
                hyper::Version::HTTP_3 => "HTTP/3.0".to_string(),
                hyper::Version::HTTP_10 => "HTTP/1.0".to_string(),
                hyper::Version::HTTP_11 => "HTTP/1.1".to_string(),
                _ => "HTTP/1.1".to_string(),
            },
        };
        for (key, value) in request.headers() {
            me.request_headers.insert(key.to_string(), std::string::String::from(value.to_str().unwrap()));
        }
        for (key, value) in response.headers() {
            me.response_headers.insert(key.to_string(), std::string::String::from(value.to_str().unwrap()));
        }
        me.request_body = hyper::body::to_bytes(request.into_body()).await.unwrap().to_vec();
        me.response_body = hyper::body::to_bytes(response.into_body()).await.unwrap().to_vec();
        return me 
    }

    pub fn get_key(&self) -> std::string::String {
        let key = format!("{}:{}:{}", self.method, self.host, self.path);
        return key
    }

    pub fn get_url(&self) -> std::string::String {
        let mut url : String = String::from(format!("{}://{}{}", self.scheme, self.host, self.path));
        if !self.query.is_empty() {
            url.push('?');
            url.push_str(&self.query);
        }
        url
    }

    pub fn get_json(&self) -> std::string::String {
        let serialized = serde_json::to_string(&self).unwrap();
        return serialized
    }

    pub fn get_query_map(&self) -> HashMap<String, String> {
        let query_string = self.query.clone();
        let mut map = HashMap::<String, String>::new();
        for pair in query_string.split('&') {
            let mut query_param = pair.split('=').take(2);
            let _keyval = match (query_param.next(), query_param.next()) {
                (Some(key), Some(val)) => {
                    map.insert(key.to_string(), val.to_string());
                },
                _ => continue,
            };
        }
        map 
    }

    pub fn get_hyper_request(&self) -> Result<hyper::Request<hyper::Body>, std::io::Error> {
        let mut request = hyper::Request::builder()
            .method(hyper::Method::from_bytes(self.method.as_bytes()).unwrap())
            .uri(format!("{}://{}{}?{}", self.scheme, self.host, self.path, self.query));
        for (key, val) in &self.request_headers {
           request = request.header(key, val);
        }
        let request = request.body(hyper::Body::from(self.request_body.clone()))
            .unwrap();
        return Ok(request)
    }

    pub fn get_hyper_response(&self) -> Result<hyper::Response<hyper::Body>, std::io::Error> {
        let mut response = hyper::Response::builder()
            .status(self.status);
        for (key, val) in &self.response_headers {
           response = response.header(key, val);
        }
        let request = response.body(hyper::Body::from(self.response_body.clone()))
            .unwrap();
        return Ok(request)
    }
    
    pub fn get_hyper_pair(&self) -> Result<(hyper::Request<hyper::Body>, hyper::Response<hyper::Body>), std::io::Error> {
        let request = self.get_hyper_request().unwrap();
        let response = self.get_hyper_response().unwrap();
        return Ok((request, response))
    }

    pub fn get_raw_request(&self) -> std::string::String {
        let mut request : String = String::new();
        request.push_str(&self.get_raw_request_title());
        request.push_str(&self.get_raw_request_headers());
        request.push('\n');
        request.push_str(&self.get_decoded_request_body());
        request
    }

    pub fn get_raw_request_title(&self) -> std::string::String {
        let mut line : String = String::new();
        line.push_str(&self.method);
        line.push(' ');
        line.push_str(&self.get_url());
        line.push(' ');
        line.push_str(&self.version);
        line.push('\n');
        line
    }

    pub fn get_raw_request_headers(&self) -> std::string::String {
        let mut line : String = String::new();
        for (key, val) in &self.request_headers {
            line.push_str(format!("{}: {}\n", &key, &val).as_str());
        }
        line
    }

    pub fn get_decoded_request_body(&self) -> std::string::String {
        let _line : String = String::new();
        if self.request_headers.contains_key("content-encoding") {
            match self.request_headers["content-encoding"].as_str() {
                "gzip" => {
                    let mut body = String::new();
                    let mut gz = GzDecoder::new(&*self.request_body);
                    gz.read_to_string(&mut body).unwrap();
                    return body
                },
                _ => {
                    panic!("New encoding type! {}", self.request_headers["content-encoding"]);
                }
            }
        } else {
            return stringify!(self.request_body).to_string();
        }
    }

    pub fn get_raw_response(&self) -> std::string::String {
        let mut response : String = String::new();
        response.push_str(&self.get_raw_response_title());
        response.push_str(&self.get_raw_response_headers());
        response.push('\n');
        response.push_str(&self.get_decoded_response_body());
        response
    }

    pub fn get_raw_response_title(&self) -> std::string::String {
        format!("{} {}\n", &self.version, &self.status)
    }

    pub fn get_raw_response_headers(&self) -> std::string::String {
        let mut line : String = String::new();
        for (key, val) in &self.response_headers {
            line.push_str(format!("{}: {}\n", &key, &val).as_str());
        }
        line
    }

    pub fn get_decoded_response_body(&self) -> std::string::String {
        let _line : String = String::new();
        if self.response_headers.contains_key("content-encoding") {
            match self.response_headers["content-encoding"].as_str() {
                "gzip" => {
                    let mut body = String::new();
                    let mut gz = GzDecoder::new(&*self.response_body);
                    gz.read_to_string(&mut body).unwrap();
                    return body
                },
                _ => {
                    panic!("New encoding type! {}", self.response_headers["content-encoding"]);
                }
            }
        } else {
            return stringify!(self.response_body).to_string();
        }
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
