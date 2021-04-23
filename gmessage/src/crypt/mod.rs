

use openssl::pkey::{Public,Private};
use crate::Error;
use crate::common::Error;
use crate::io;
use openssl::pkey::PKey;

pub mod sign;
pub mod rsa;
pub mod aes;

pub async fn load_public_key(p:String) -> Result<PKey<Public>,Error>{
    let as_string:String;
    match io::read_string(&p).await{
        Ok(v)=>{as_string = v;},
        Err(e)=>{return Err(Error!(format!("failed-read_key_file=>{:?}",e)));}
    }
    match openssl::rsa::Rsa::public_key_from_pem(&as_string.into_bytes()){
        Ok(v)=>{
            match PKey::from_rsa(v){
                Ok(f)=>{return Ok(f);},
                Err(e)=>{
                    return Err(Error!(format!("failed-parse_pkey=>{:?}",e)));
                }
            }
        },
        Err(e)=>{return Err(Error!(format!("failed-load_key=>{:?}",e)));}
    }
}

pub async fn load_private_key(p:String) -> Result<PKey<Private>,Error>{
    let as_string:String;
    match io::read_string(&p).await{
        Ok(v)=>{as_string = v;},
        Err(e)=>{return Err(Error!(format!("failed-read_key_file=>{:?}",e)));}
    }
    match openssl::rsa::Rsa::private_key_from_pem(&as_string.into_bytes()){
        Ok(v)=>{
            match PKey::from_rsa(v){
                Ok(f)=>{return Ok(f);},
                Err(e)=>{
                    return Err(Error!(format!("failed-parse_pkey=>{:?}",e)));
                }
            }
        },
        Err(e)=>{return Err(Error!(format!("failed-load_key=>{:?}",e)));}
    }
}