use crate::Traffic;
use crate::CONFIG;

use std::collections::HashMap;
use std::io::{Read, Write, Error, ErrorKind};
use tokio::net::{TcpStream, TcpListener};
use flate2::{read::GzDecoder, read::DeflateDecoder, Decompress};
use brotli::Decompressor;
use futures::future::{Future, BoxFuture};

// Defining the type that a filtering function takes.
type FilterFunction = fn(&mut Traffic) -> BoxFuture<'_, Result<(), ()>>;

pub struct Filter {
    filters: Vec<FilterFunction> 
}

impl Filter {
    pub async fn new() -> Self {
        Self {
            filters : vec![
                |traffic| Box::pin(check_identity_providers(traffic)),
                |traffic| Box::pin(check_allow_list_host(traffic)),
                |traffic| Box::pin(check_deny_list_host(traffic)),
                |traffic| Box::pin(decompress_gzip(traffic)),
                |traffic| Box::pin(decompress_deflate(traffic)),
                |traffic| Box::pin(decompress_br(traffic)),
                |traffic| Box::pin(parse_utf8_request(traffic)),
                |traffic| Box::pin(parse_utf8_response(traffic)),
            ]
        }
    }

    pub async fn filter(&self, traffic: &mut Traffic) -> Result<(), ()> { 
        for function in &self.filters {
            match (function)(traffic).await{
                Ok(_) => { continue },
                Err(_) => { return Err(()) },
            }
        }
        Ok(())
    }
}

// Filter on config's [filter] vectors. 

pub async fn check_allow_list_host(traffic: &mut Traffic) -> Result<(), ()> {
    let config = CONFIG.get().expect("Config is not initialized, somehow...");
    if config.filter.allow_list_hosts.len().eq(&0) {
        return Ok(()) // If you don't have any entries on the allow list pass everything.
    }
    for allowed_host in &config.filter.allow_list_hosts {
        if traffic.host.contains(allowed_host) {
            return Ok(())
        }
    }
    return Err(()) // Drop this undesirable traffic.
}

pub async fn check_deny_list_host(traffic: &mut Traffic) -> Result<(), ()> {
    let config = CONFIG.get().expect("Config is not intialized, somehow...");
    for denied_host in &config.filter.deny_list_hosts {
        if traffic.host.contains(denied_host) {
            return Err(()) // Drop this undesirable traffic.
        }
    }
    return Ok(())
}

pub async fn check_identity_providers(traffic: &mut Traffic) -> Result<(), ()> {
    let config = CONFIG.get().expect("");
    for idp in &config.filter.identity_providers {
        if traffic.host.contains(idp) {
            // Spawn auth-parsing task. hashmap
            return Err(()) // This traffic is not intended for our collection.
        }
    }
    return Ok(()) // This is not an identity provider, we can proceed.
}

// Parsing strings from bodies.

pub async fn parse_utf8_request(traffic: &mut Traffic) -> Result<(), ()> {
    match std::str::from_utf8(&traffic.request_body) {
        Ok(request_body_string) => {
            traffic.request_body_string = Some(request_body_string.to_string().clone());
            return Ok(())
        },
        Err(e) => {
            traffic.request_body_string = None;
            return Ok(())
        } 
    }
}

pub async fn parse_utf8_response(traffic: &mut Traffic) -> Result<(), ()> {
    match std::str::from_utf8(&traffic.response_body) {
        Ok(response_body_string) => {
            traffic.response_body_string = Some(response_body_string.to_string().clone());
            return Ok(())
        },
        Err(e) => {
            traffic.response_body_string = None;
            return Ok(())
        } 
    }
}


// gzip, br, deflate only.

pub async fn decompress_gzip(traffic: &mut Traffic) -> Result<(), ()> {
    if !(traffic.response_headers.contains_key("content-encoding")){
        return Ok(())
    }
    if !(traffic.response_headers["content-encoding"] == "gzip".to_string()){
        return Ok(())
    }
    let mut encoded_body = traffic.response_body.clone();
    let mut decoded_buffer = Vec::new(); 
    let mut gz = GzDecoder::new(&encoded_body[..]);
    gz.read_to_end(&mut decoded_buffer).unwrap();
    traffic.response_body = decoded_buffer.clone();
    traffic.response_headers.remove("content-encoding");
    Ok(())
}

pub async fn decompress_deflate(traffic: &mut Traffic) -> Result<(), ()> {
    if !(traffic.response_headers.contains_key("content-encoding")) {
        return Ok(())
    }
    if !(traffic.response_headers["content-encoding"] == "deflate".to_string()) {
        return Ok(())
    }
    let mut encoded_body = traffic.response_body.clone();
    let mut decoded_buffer = Vec::new();
    let mut deflate = DeflateDecoder::new(&encoded_body[..]);
    deflate.read_to_end(&mut decoded_buffer).unwrap();
    traffic.response_body = decoded_buffer.clone();
    traffic.response_headers.remove("content-encoding");
    Ok(())
}

pub async fn decompress_br(traffic: &mut Traffic) -> Result<(), ()> {
    if !(traffic.response_headers.contains_key("content-encoding")) {
        return Ok(())
    }
    if !(traffic.response_headers["content_encoding"] == "br".to_string()) {
        return Ok(())
    }
    let mut encoded_body = traffic.response_body.clone();
    let mut decoded_buffer = Vec::new();
    let mut brotli = brotli::DecompressorWriter::new(&mut decoded_buffer[..], 4096);
    brotli.write_all(&encoded_body[..]).unwrap();
    brotli.into_inner().unwrap();
    traffic.response_body = decoded_buffer.clone();
    traffic.response_headers.remove("content-encoding");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_decompress_gzip() -> Result<(), std::io::Error> {
        let decoded_string = r###"try{
s_a("i9SNBf");
var s_EGf=s_H("dXIA6");var s_FGf=function(a){s_o.call(this,a.Ka);this.rootElement=this.getRoot().el();this.RQ=s_K(this,"MPu53c").el();if(a=s_Rq(this.rootElement,"labelledby")){var b=document.getElementById(a);b&&(b.setAttribute("for",this.RQ.getAttribute("id")),s_Pq(this.RQ,"labelledby",a))}};s_w(s_FGf,s_o);s_FGf.Fa=s_o.Fa;s_FGf.prototype.Hm=function(a,b){this.RQ.checked!==a&&(this.RQ.checked=a,(void 0===b||b)&&this.trigger(s_EGf))};s_T(s_4Ta,s_FGf);
s_b();
}catch(e){_DumpException(e)}
try{
var s_ETd=s_H("Lhx8ef");
}catch(e){_DumpException(e)}
try{
s_a("w4UyN");
var s_7R=function(a){s_o.call(this,a.Ka);this.ka=!1;this.oa=s_Ib("elPddd");this.rootElement=this.getRoot().el()};s_w(s_7R,s_o);s_7R.Fa=s_o.Fa;s_7R.prototype.vmf=function(){""===s_i.getStyle(this.oa,"transform")?(s_U(this.rootElement),s_xd(document,s_ETd),this.ka||(this.gA(),this.ka=!0)):s_i.setStyle(this.oa,"transform","");this.Ua("suEOdc").setStyle("visibility","hidden")};s_7R.prototype.showTooltip=function(){this.Ua("suEOdc").setStyle("visibility","inherit")};
s_7R.prototype.uj=function(){this.Ua("suEOdc").setStyle("visibility","hidden")};s_7R.prototype.gA=function(){var a=s_Bx(new s_Ax,s_Dx(new s_Cx,134634));s_xd(document,s_Ex,{q4:a})};s_L(s_7R.prototype,"LfDNce",function(){return this.uj});s_L(s_7R.prototype,"eGiyHb",function(){return this.showTooltip});s_L(s_7R.prototype,"HfCvm",function(){return this.vmf});s_T(s_KRa,s_7R);
s_b();
}catch(e){_DumpException(e)}
// Google Inc.
"###; 

        let mut traffic = Traffic{
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

        let encoded_body = traffic.response_body.clone();
        let result = decompress_gzip(&mut traffic).await.unwrap();
        let decoded_traffic = traffic.response_body.clone();
        println!("{}", std::str::from_utf8(&decoded_traffic).unwrap());
        assert_ne!(encoded_body, decoded_traffic);
        assert_eq!(decoded_string, std::str::from_utf8(&decoded_traffic).unwrap());
        Ok(())
    }

}
