use async_trait::async_trait;

pub mod mongo;

// https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/
#[async_trait]
pub trait Datastore {
    async fn add_traffic(&self, traffic : &crate::Traffic) -> Result<(), Box<dyn std::error::Error>>; 
    async fn add_authinfo(&self, auth : &crate::AuthInfo) -> Result<(), Box<dyn std::error::Error>>;
}
