use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthInfo {
    pub grant_type: String,
    pub issuer: String,
    pub client_id: String,
    pub redirect_url: String,
    pub scope: String,
}
impl PartialEq for AuthInfo {
    fn eq(&self, other: &Self) -> bool {
        (self.grant_type == other.grant_type)
            && (self.issuer == other.issuer)
            && (self.client_id == other.client_id)
            && (self.scope == other.scope)
            && (self.redirect_url == other.redirect_url)
    }
}
impl Eq for AuthInfo {}
impl fmt::Display for AuthInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_json())
    }
}
impl AuthInfo {
    pub fn new(
        issuer: String,
        grant_type: String,
        client_id: String,
        redirect_url: String,
        scope: String,
    ) -> Self {
        Self {
            issuer,
            grant_type,
            client_id,
            redirect_url,
            scope,
        }
    }

    pub fn get_json(&self) -> std::string::String {
        serde_json::to_string(&self).unwrap()
    }
}
