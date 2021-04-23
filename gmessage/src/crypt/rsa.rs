
use openssl::pkey::{PKey,Public,Private};
use openssl::rsa::Padding;
use openssl::encrypt::{Encrypter, Decrypter};

use crate::Error;
use crate::common::Error;

pub fn encrypt(data:Vec<u8>,public:&PKey<Public>) -> Result<Vec<u8>,Error>{

    let mut encrypter;
    match Encrypter::new(&public){
        Ok(v)=>{encrypter = v;},
        Err(_)=>{
            return Err(Error!("make-encrypter-encrypt-rsa"));
        }
    }

    match encrypter.set_rsa_padding(Padding::PKCS1){
        Ok(_)=>{},
        Err(_)=>{
            return Err(Error!("apply-padding-encrypt-rsa"));
        }
    }

    let buffer_len:usize;
    match encrypter.encrypt_len(&data){
        Ok(v)=>{buffer_len = v;},
        Err(_)=>{
            return Err(Error!("get-buffer_len-encrypt-rsa"));
        }
    }

    let mut encrypted = vec![0; buffer_len];
    let encrypted_len:usize;
    match encrypter.encrypt(&data, &mut encrypted){
        Ok(v)=>{encrypted_len = v;},
        Err(_)=>{
            return Err(Error!("generate-encrypted-encrypt-rsa"));
        }
    }
    encrypted.truncate(encrypted_len);

    return Ok(encrypted);

}

pub fn decrypt(data:Vec<u8>,private:&PKey<Private>) -> Result<Vec<u8>,Error>{

    let mut decrypter;
    match Decrypter::new(&private){
        Ok(v)=>{decrypter = v;},
        Err(_)=>{
            return Err(Error!("make-decrypter-decrypt-rsa"));
        }
    }

    match decrypter.set_rsa_padding(Padding::PKCS1){
        Ok(_)=>{},
        Err(_)=>{
            return Err(Error!("apply-padding-decrypt-rsa"));
        }
    }

    let buffer_len:usize;
    match decrypter.decrypt_len(&data){
        Ok(v)=>{buffer_len = v;},
        Err(_)=>{
            return Err(Error!("get-buffer_len-decrypt-rsa"));
        }
    }

    let mut decrypted = vec![0; buffer_len];
    let decrypted_len:usize;
    match decrypter.decrypt(&data, &mut decrypted){
        Ok(v)=>{decrypted_len = v;},
        Err(_)=>{
            return Err(Error!("generate-decrypted-decrypt-rsa"));
        }
    }
    decrypted.truncate(decrypted_len);

    return Ok(decrypted);

}