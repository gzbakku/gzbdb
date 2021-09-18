

use crate::Error;
use crate::common::Error;
use crate::crypt;

pub enum AuthToken{
    Session(SessionAuth),
    Key(KeyAuth)
}

use openssl::pkey::{Public,Private};
use openssl::pkey::PKey;

#[allow(dead_code)]
impl AuthToken{
    pub fn session(id:String,password:String) -> AuthToken{
        AuthToken::Session(SessionAuth{
            id:id,
            password:password
        })
    }
    pub async fn key(public_key:String,private_key:String) -> Result<AuthToken,Error>{

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

        Ok(AuthToken::Key(KeyAuth{
            public:public,
            private:private
        }))

    }
    pub async fn validate(&self,stream:&mut MessageBox) -> Result<(),Error>{
        match self{
            AuthToken::Session(t)=>{
                match validate_session(t,stream).await{
                    Ok(_)=>{return Ok(());},
                    Err(e)=>{return Err(Error!("failed-validate_session"=>e));}
                }
            },
            AuthToken::Key(t)=>{
                match validate_key(t,stream).await{
                    Ok(_)=>{return Ok(());},
                    Err(e)=>{return Err(Error!("failed-validate_key"=>e));}
                }
            }
        }
    }
}

pub struct SessionAuth{
    pub id:String,
    pub password:String
}

pub struct KeyAuth{
    pub public:PKey<Public>,
    pub private:PKey<Private>
}

use gobject::{gObject,gObjectValue,gSchema};
use crate::messenger::MessageBox;

async fn validate_key(token:&KeyAuth,stream:&mut MessageBox) -> Result<(),Error>{

    // println!("validating key");

    //get challenge
    match stream.send(gObject!{
        "connect"=>gObjectValue::bool(true),
        "protocol"=>gObjectValue::string("key".to_string())
    }.build()).await{
        Ok(_)=>{},
        Err(e)=>{
            println!("send failed");
            return Err(Error!("failed-send_protocol_message"=>e));
        }
    }

    //------------------------------------------
    //get confirmation

    let confirmation:gObject;
    match stream.read_first_doc(10).await{
        Ok(v)=>{
            match gSchema!{
                "connect"=>gSchemaValue::bool
            }.validate(&v){Ok(_)=>{confirmation = v;},Err(_)=>{
                return Err(Error!("failed-validate-confirmation-request"));
            }}
        },
        Err(_)=>{
            return Err(Error!("failed-read-confirmation-request"));
        }
    }
    match confirmation["connect"]{
        gObjectValue::bool(v)=>{
            if v == false {
                return Err(Error!("failed-denied-confirmation-request"));
            }
        },
        _=>{
            return Err(Error!("failed-denied-confirmation-request"));
        }
    }

    //------------------------------------------
    //get challenge

    match stream.send(gObject!{
        "get_challenge"=>gObjectValue::bool(true)
    }.build()).await{
        Ok(_)=>{},
        Err(e)=>{
            println!("send failed");
            return Err(Error!("failed-send_protocol_message"=>e));
        }
    }

    let challenge_object:gObject;
    match stream.read_first_doc(10).await{
        Ok(v)=>{
            match gSchema!{
                "challenge"=>gSchemaValue::string
            }.validate(&v){Ok(_)=>{challenge_object = v;},Err(_)=>{
                return Err(Error!("failed-validate-challenge-request"));
            }}
        },
        Err(_)=>{
            return Err(Error!("failed-read-challenge-request"));
        }
    }
    let challenge:String;
    match &challenge_object["challenge"]{
        gObjectValue::string(v)=>{
            challenge = v.to_string();
        },
        _=>{
            return Err(Error!("failed-extract_challenge-challenge-request"));
        }
    }

    //------------------------------------------
    //make sig

    let sig:Vec<u8>;
    match crypt::sign::sign(challenge.as_bytes().to_vec(),&token.private){
        Ok(v)=>{sig = v;},
        Err(_)=>{
            return Err(Error!("failed-generate-sign-request"));
        }
    }
    match stream.send(gObject!{
        "signature"=>gObjectValue::binary(sig)
    }.build()).await{
        Ok(_)=>{},
        Err(e)=>{
            return Err(Error!("failed-send_signature"=>e));
        }
    }

    //------------------------------------------
    //get password

    let password_object:gObject;
    match stream.read_first_doc(10).await{
        Ok(v)=>{
            match gSchema!{
                "password"=>gSchemaValue::binary
            }.validate(&v){Ok(_)=>{password_object = v;},Err(_)=>{
                return Err(Error!("failed-validate-password_object-request"));
            }}
        },
        Err(_)=>{
            return Err(Error!("failed-read-password_object-request"));
        }
    }
    let password_encrypted:Vec<u8>;
    match &password_object["password"]{
        gObjectValue::binary(v)=>{
            password_encrypted = v.clone();
        },
        _=>{
            return Err(Error!("failed-extract-password-password_object-request"));
        }
    }

    //------------------------------------------
    //decrypt password

    match crypt::rsa::decrypt(password_encrypted, &token.private){
        Ok(v)=>{stream.add_password(v);},
        Err(_)=>{
            return Err(Error!("failed-decrypt-password-password_object-request"));
        }
    }

    match stream.send(gObject!{
        "start"=>gObjectValue::bool(true)
    }.build()).await{
        Ok(_)=>{},
        Err(e)=>{
            return Err(Error!("failed-send_start"=>e));
        }
    }

    //------------------------------------------
    //wait for server to be ready

    let ready_object:gObject;
    match stream.read_first_doc(10).await{
        Ok(v)=>{
            match gSchema!{
                "ready"=>gSchemaValue::bool
            }.validate(&v){Ok(_)=>{ready_object = v;},Err(_)=>{
                return Err(Error!("failed-validate-ready_object-request"));
            }}
        },
        Err(_)=>{
            return Err(Error!("failed-read-ready_object-request"));
        }
    }
    match &ready_object["ready"]{
        gObjectValue::bool(ready)=>{
            if !ready{
                return Err(Error!("server_not_ready-ready_object-request"));
            }
        },
        _=>{
            return Err(Error!("invalid-value-ready_object-request"));
        }
    }

    // println!("{:?}",ready_object);

    // return Err(Error!("no_error"));

    Ok(())

}

async fn validate_session(_token:&SessionAuth,_stream:&mut MessageBox) -> Result<(),Error>{

    println!("validating session");

    Err(Error!("no_error"))

}