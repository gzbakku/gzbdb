

use crate::v4::client::auth::AuthToken;

pub struct ClientConfig{
    pub channels:u32,
    pub addr:String,
    pub auth_token:AuthToken
}

#[allow(dead_code)]
impl ClientConfig{
    pub fn new(auth:AuthToken,addr:String,channels:u32) -> ClientConfig{
        ClientConfig{
            channels:channels,
            addr:addr,
            auth_token:auth
        }
    }
}