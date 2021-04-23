

use openssl::symm::Cipher;
use openssl::symm::encrypt as encrypter;
use openssl::symm::decrypt as decrypter;
use crate::Error;
use crate::common::Error;
use rand::{distributions::Uniform, Rng}; // 0.8.0
use gobject::{gObject,gObjectValue,gSchema,parse};
// use gobject;

pub struct Encrypted{
    pub iv:Vec<u8>,
    pub cipher:Vec<u8>
}

#[allow(dead_code)]
impl Encrypted{
    pub fn new(i:Vec<u8>,c:Vec<u8>)->Encrypted{
        Encrypted{
            iv:i,
            cipher:c
        }
    }
    pub fn g_object(self)->gObject{
        gObject!{
            "iv"=>gObjectValue::binary(self.iv),
            "cipher"=>gObjectValue::binary(self.cipher)
        }
    }
    pub fn binary(self)->Vec<u8>{
        self.g_object().build()
    }
    pub fn load_from_g_object(o:&gObject)->Result<Encrypted,Error>{
        match gSchema!{
            "iv"=>gSchemaValue::binary,
            "cipher"=>gSchemaValue::binary
        }.validate(&o){
            Ok(_)=>{
                let iv:Vec<u8>;
                match &o["iv"]{
                    gObjectValue::binary(v)=>{iv = v.clone();}
                    _=>{return Err(Error!("invalid-value-iv"));}
                }
                let cipher:Vec<u8>;
                match &o["cipher"]{
                    gObjectValue::binary(v)=>{cipher = v.clone();}
                    _=>{return Err(Error!("invalid-value-cipher"));}
                }
                Ok(Encrypted{iv:iv,cipher:cipher})
            },
            Err(_)=>{
                return Err(Error!("invalid-schema"));
            }
        }
    }
    pub fn unlock_first_g_object(self,key:&Vec<u8>)->Result<gObjectValue,Error>{
        match self.unlock_g_object(&key){
            Ok(docs)=>{
                if docs.len() > 0{
                    return Ok(docs[0].clone());
                } else {
                    return Err(Error!("no_docs"));
                }
            },
            Err(e)=>{return Err(Error!("failed-unlock_g_object"=>e));}
        }
    }
    pub fn unlock_g_object(self,key:&Vec<u8>) -> Result<Vec<gObjectValue>,Error>{
        match self.decrypt(key){
            Ok(binary)=>{
                match parse(&binary){
                    Ok(blocks)=>{
                        return Ok(blocks.get());
                    },
                    Err(_)=>{return Err(Error!("failed-parse-unencrypted_data"));}
                }
            },
            Err(_)=>{return Err(Error!("failed-decrypt"));}
        }
    }
    pub fn decrypt(self,key:&Vec<u8>) -> Result<Vec<u8>,Error>{
        match decrypt(key, &self.iv,&self.cipher){
            Ok(v)=>{return Ok(v);},
            Err(_)=>{return Err(Error!("failed-decrypt"));}
        }
    }
}

pub fn encrypt(key:&Vec<u8>,data:&Vec<u8>) -> Result<Encrypted,Error>{
    let iv: Vec<u8> = rand::thread_rng().sample_iter(&Uniform::from(0..=255)).take(16).collect();
    match encrypter(Cipher::aes_256_cbc(),&key,Some(&iv),data){
        Ok(ciphertext)=>{return Ok(Encrypted::new(iv,ciphertext));},
        Err(e)=>{return Err(Error!(format!("failed-encrypt=>{:?}",e)));}
    }
}

pub fn decrypt(key:&Vec<u8>,iv:&Vec<u8>,cipher:&Vec<u8>) -> Result<Vec<u8>,Error>{
    match decrypter(Cipher::aes_256_cbc(),key,Some(iv),cipher){
        Ok(v)=>{return Ok(v);},
        Err(_)=>{return Err(Error!("failed-decrypt"));}
    }   
}