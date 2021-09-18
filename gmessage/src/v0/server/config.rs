use crate::common::Error;
use crate::Error;
use openssl::pkey::{Public,Private};
use crate::crypt;
use openssl::pkey::PKey;

#[allow(non_camel_case_types)]
pub enum RsaKey{
    public(PKey<Public>),
    private(PKey<Private>),
    none
}

pub struct ServerConfig{
    pub socket:i32,
    pub public_key:RsaKey,
    pub private_key:RsaKey
}

#[allow(dead_code)]
impl ServerConfig{
    pub fn empty() -> ServerConfig{
        ServerConfig{
            socket:1200,
            public_key:RsaKey::none,
            private_key:RsaKey::none
        }
    }
    pub async fn new(port:i32,private_key:String,public_key:String) -> Result<ServerConfig,Error>{
        
        let public:PKey<Public>;
        match crypt::load_public_key(public_key).await{
            Ok(v)=>{public = v;},
            Err(e)=>{
                return Err(Error!("failed-load_public_key"=>e));
            }
        }

        let private:PKey<Private>;
        match crypt::load_private_key(private_key).await{
            Ok(v)=>{private = v;},
            Err(e)=>{
                return Err(Error!("failed-load_private_key"=>e));
            }
        }

        let build = ServerConfig{
            socket:port,
            public_key:RsaKey::public(public),
            private_key:RsaKey::private(private)
        };

        Ok(build)
    }
}



