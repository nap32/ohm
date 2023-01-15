use crate::Traffic;

use std::io::{Read, Write, Error, ErrorKind};
use tokio::net::{TcpStream, TcpListener};
use flate2::{read::GzDecoder, read::DeflateDecoder, Decompress};
use brotli::Decompressor;

pub struct Filter {
//    filters: Vec<fn(&self, &mut Traffic) -> Result<&mut Traffic, std::io::Error> > 
}

impl Filter {
//    pub async fn filter(&self, traffic: &mut Traffic) -> Result<&mut Traffic, std::io::Error> { 
//        for f in &self.filters {
//            println!("{}", &self.(f)(traffic));
//        }
//    }
}

// gzip, br, deflate only.

pub async fn decompress_gzip(traffic: &mut Traffic) -> Result<&mut Traffic, std::io::Error> {
    let mut encoded_body = traffic.response_body.clone();
    let mut decoded_buffer = Vec::new(); 
    let mut gz = GzDecoder::new(&encoded_body[..]);
    gz.read_to_end(&mut decoded_buffer).unwrap();
    traffic.response_body = decoded_buffer.clone();
    Ok(traffic)
}

pub async fn decompress_deflate(traffic: &mut Traffic) -> Result<&mut Traffic, std::io::Error> {
    let mut encoded_body = traffic.response_body.clone();
    let mut decoded_buffer = Vec::new();
    let mut deflate = DeflateDecoder::new(&encoded_body[..]);
    deflate.read_to_end(&mut decoded_buffer).unwrap();
    traffic.response_body = decoded_buffer.clone();
    Ok(traffic)
}

pub async fn decompress_br(traffic: &mut Traffic) -> Result<&mut Traffic, std::io::Error> {
    let mut encoded_body = traffic.response_body.clone();
    let mut decoded_buffer = Vec::new();
    let mut brotli = brotli::DecompressorWriter::new(&mut decoded_buffer[..], 4096);
    brotli.write_all(&encoded_body[..]).unwrap();
    brotli.into_inner().unwrap();
    traffic.response_body = decoded_buffer.clone();
    Ok(traffic)
}
