

use crate::Error;
use crate::common::Error;

use openssl::pkey::{Private,Public};
use openssl::pkey::PKey;
use openssl::sign::{Signer, Verifier};
use openssl::hash::MessageDigest;

pub fn sign(val:Vec<u8>,rsa_key:&PKey<Private>) -> Result<Vec<u8>,Error>{

    let mut signer:Signer;
    match Signer::new(MessageDigest::sha256(), &rsa_key){
        Ok(v)=>{signer = v;},
        Err(_)=>{
            return Err(Error!("failed-make-signer"));
        }
    }

    match signer.update(&val){
        Ok(_)=>{},
        Err(_)=>{
            return Err(Error!("failed-load-signer"));
        }
    }

    match signer.sign_to_vec(){
        Ok(v)=>{
            return Ok(v);
        },
        Err(_)=>{
            return Err(Error!("failed-run-signer"));
        }
    }

}

pub fn verify(val:Vec<u8>,rsa_key:&PKey<Public>,sig:Vec<u8>) -> Result<(),Error>{

    let mut verifier:Verifier;
    match Verifier::new(MessageDigest::sha256(), &rsa_key){
        Ok(v)=>{verifier = v;},
        Err(_)=>{
            return Err(Error!("failed-make-verifier"));
        }
    }

    match verifier.update(&val){
        Ok(_)=>{},
        Err(_)=>{
            return Err(Error!("failed-load-verifier"));
        }
    }

    match verifier.verify(&sig){
        Ok(v)=>{
            if v{
                return Ok(());
            } else {
                return Err(Error!("invalid-sig"));
            }
        },
        Err(_)=>{
            return Err(Error!("failed-run-verifier"));
        }
    }

}