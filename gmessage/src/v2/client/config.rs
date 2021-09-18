

use crate::v2::client::auth::AuthToken;

pub struct ClientConfig{
    pub addr:String,
    pub auth_token:AuthToken
}

#[allow(dead_code)]
impl ClientConfig{
    pub fn new(auth:AuthToken,addr:String) -> ClientConfig{
        ClientConfig{
            addr:addr,
            auth_token:auth
        }
    }
}