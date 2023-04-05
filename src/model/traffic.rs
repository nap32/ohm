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
        request.push_str(&self.get_decoded_request_body());
        request.push_str("\r\n");
        request.push_str("\r\n");
        request
    }

    pub fn get_raw_request_title(&self) -> std::string::String {
        let mut line : String = String::new();
        line.push_str(&self.method);
        line.push(' ');
        line.push_str(&self.get_url());
        line.push(' ');
        line.push_str(&self.version);
        line.push_str("\r\n");
        line
    }

    pub fn get_raw_request_headers(&self) -> std::string::String {
        let mut line : String = String::new();
        for (key, val) in &self.request_headers {
            line.push_str(format!("{}: {}\r\n", &key, &val).as_str());
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
        response.push_str(&self.get_decoded_response_body());
        response.push_str("\r\n");
        response.push_str("\r\n");
        response
    }

    pub fn get_raw_response_title(&self) -> std::string::String {
        format!("{} {}\r\n", &self.version, &self.status)
    }

    pub fn get_raw_response_headers(&self) -> std::string::String {
        let mut line : String = String::new();
        for (key, val) in &self.response_headers {
            line.push_str(format!("{}: {}\r\n", &key, &val).as_str());
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
    use lazy_static::lazy_static;

    lazy_static! {
        static ref TRAFFIC_ONE : Traffic = Traffic{
            method:"GET".to_string(),
            scheme:"https".to_string(),
            host:"www.google.com".to_string(),
            path:"/".to_string(),
            query:"xjs=s2".to_string(),
            request_headers: HashMap::from([
                ("cookie".to_string(),                          "foo=bar".to_string()),
                ("sec-fetch-site".to_string(),                  "same-origin".to_string()),
                ("user-agent".to_string(),                      "Mozilla/5.0 (X11; Linux x86_64; rv:107.0) Gecko/20100101 Firefox/107.0".to_string()),
                ("sec-fetch-dest".to_string(),                  "script".to_string()),
                ("sec-fetch-mode".to_string(),                  "no-cors".to_string()),
                ("accept-language".to_string(),                 "en-US,en;q=0.5".to_string()),
                ("accept-encoding".to_string(),                 "gzip, deflate, br".to_string()),
                ("accept".to_string(),                          "*/*".to_string()),
                ("referer".to_string(),                         "https://www.google.com/".to_string()),
                ("host".to_string(),                            "www.google.com".to_string()),
                ("connection".to_string(),                      "keep-alive".to_string()),
            ]),
            request_body:[].to_vec(),
            request_body_string:None,
            status:200,
            response_headers: HashMap::from([
                ("vary".to_string(),                            "Accept-Encoding, Origin".to_string()),
                ("expires".to_string(),                         "Thu, 04 Jan 2024 01:28:10 GMT".to_string()),
                ("server".to_string(),                          "sffe".to_string()),
                ("content-type".to_string(),                    "text/javascript; charset=UTF-8".to_string()),
                ("content-encoding".to_string(),                "gzip".to_string()),
                ("date".to_string(),                            "Wed, 04 Jan 2023 01:28:10 GMT".to_string()),
                ("last-modified".to_string(),                   "Tue, 03 Jan 2023 09:12:52 GMT".to_string()),
                ("cross-origin-resource-policy".to_string(),    "cross-origin".to_string()),
                ("cross-origin-opener-policy".to_string(),      "same-origin; report-to=gws-team".to_string()),
                ("content-length".to_string(),                  "658".to_string()),
                ("cache-control".to_string(),                   "public, max-age=31536000".to_string())
            ]),
            response_body:[31,139,8,0,0,0,0,0,0,0,157,83,209,110,218,48,20,125,231,43,168,31,144,45,89,105,171,178,178,21,69,19,109,105,65,237,186,54,165,210,222,34,39,118,136,219,16,179,216,1,162,192,191,207,54,161,13,108,108,104,79,137,175,125,207,185,62,231,88,101,69,217,144,62,129,128,127,121,126,184,140,0,234,54,102,36,107,74,191,127,27,185,210,31,64,64,127,12,123,231,186,190,46,223,232,114,148,167,161,226,34,133,4,149,210,23,78,72,146,4,170,152,75,76,156,59,130,186,230,215,201,132,80,253,132,77,88,170,92,91,24,51,229,233,26,68,14,75,96,117,200,123,210,28,119,235,94,240,237,49,255,116,22,130,106,159,71,144,232,77,239,39,220,133,195,32,33,1,75,18,70,131,2,32,84,154,193,2,151,138,48,55,187,134,167,58,120,89,12,169,158,177,27,180,90,48,112,36,83,61,165,50,30,228,138,65,16,137,12,224,106,6,211,82,219,226,84,163,98,233,63,86,212,222,211,22,35,38,8,173,86,93,233,207,161,149,67,159,20,168,107,127,157,27,51,178,208,159,106,61,205,132,18,170,152,50,103,48,169,201,134,3,84,110,184,195,152,133,111,140,30,185,46,209,99,238,84,93,130,225,76,112,218,60,113,93,55,88,46,3,212,106,217,35,122,216,241,152,101,208,250,164,231,209,124,35,189,104,143,8,182,204,218,70,233,7,90,198,198,42,36,42,140,33,67,165,127,157,79,166,253,69,200,166,118,10,134,86,13,101,236,175,252,30,209,181,223,247,241,226,51,179,65,248,119,171,77,206,188,253,82,60,124,4,167,227,29,22,144,55,226,30,157,174,127,133,145,109,24,64,192,146,71,74,181,252,7,69,104,227,65,199,219,88,208,241,182,28,208,203,15,3,102,147,90,112,81,9,128,86,84,250,220,96,62,171,34,97,176,154,4,3,149,145,84,234,124,76,0,250,170,225,95,126,11,160,9,199,130,194,77,226,176,21,15,225,234,86,203,229,186,97,220,131,239,53,247,232,4,161,11,67,39,255,66,135,193,230,230,47,90,86,153,247,191,83,243,28,222,91,192,140,75,30,240,132,43,157,66,16,115,74,89,10,172,12,91,55,149,177,152,143,132,72,20,159,214,111,124,48,48,79,99,150,113,101,144,27,59,208,249,235,127,33,238,29,117,220,171,227,153,252,24,251,46,23,48,101,115,29,165,222,66,107,123,189,89,93,45,240,233,89,251,252,172,141,140,215,59,6,44,112,249,179,125,65,86,150,227,30,110,243,96,112,31,93,63,132,12,224,26,91,198,84,158,165,77,123,137,252,117,133,254,216,199,110,121,49,8,246,246,213,164,222,3,48,136,174,102,147,189,253,58,148,182,207,188,221,59,207,188,221,142,119,224,211,61,62,110,222,10,49,78,88,115,152,134,78,227,23,158,240,192,30,201,5,0,0].to_vec(),
            response_body_string:None,
            version:"HTTP/1.1".to_string()
        };

        static ref TRAFFIC_TWO : Traffic = Traffic{
            method:"GET".to_string(),
            scheme:"https".to_string(),
            host:"foobar.com".to_string(),
            path:"/".to_string(),
            query:"".to_string(),
            request_headers: HashMap::from([
                ("user-agent".to_string(),                      "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/110.0".to_string()),
                ("host".to_string(),                            "foobar.com".to_string()),
                ("accept-encoding".to_string(),                 "gzip, deflate, br".to_string()),
                ("accept-language".to_string(),                 "en-US,en;q=0.5".to_string()),
                ("accept".to_string(),                          "*/*".to_string()),
                ("connection".to_string(),                      "keep-alive".to_string()),
            ]),
            request_body:[].to_vec(),
            request_body_string:None,
            status:200,
            response_headers: HashMap::from([]),
            response_body:[].to_vec(),
            response_body_string:None,
            version:"HTTP/1.1".to_string()
        };
    }

    #[test]
    fn test_get_url() {
        assert_eq!(TRAFFIC_ONE.get_url(), "https://www.google.com/?xjs=s2");
        assert_eq!(TRAFFIC_TWO.get_url(), "https://foobar.com/");
        assert_ne!(TRAFFIC_ONE.get_url(), TRAFFIC_TWO.get_url());
        assert_eq!(TRAFFIC_ONE.get_url(), TRAFFIC_ONE.get_url());
        assert_eq!(TRAFFIC_TWO.get_url(), TRAFFIC_TWO.get_url());
    }

    #[test]
    fn test_get_json() {
        assert_eq!(TRAFFIC_ONE.get_json(), TRAFFIC_ONE.get_json());
        assert_eq!(TRAFFIC_TWO.get_json(), TRAFFIC_TWO.get_json());
    }

}

