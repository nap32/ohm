use async_trait::async_trait;

pub mod mongo;
pub mod postgres;
pub mod redis;

// https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/
#[async_trait]
pub trait Datastore {
    async fn add_traffic(&self, traffic : &crate::Traffic) -> Result<(), Box<dyn std::error::Error>>; 
    //async fn add_record(&self, record : &crate::Record) -> Result<(), Box<dyn std::error::Error>>;
    //async fn add_auth(&self, auth : &crate::Auth) -> Result<(), Box<dyn std::error::Error>>;
}
