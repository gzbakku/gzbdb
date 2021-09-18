use tokio::runtime::{Builder,Runtime};
use std::sync::{Mutex,Arc};
use crate::server::{ipc::{SharedData,ThreadMessage},ServerConfig,RsaKey};
use crate::common::Error;
use crate::Error;
use tokio::net::{TcpListener,TcpStream};

pub fn start(d:Arc<Mutex<SharedData>>,config:ServerConfig,channel_write:Arc<Mutex<Sender<ThreadMessage>>>) -> Result<(),Error>{

    let builder = Builder::new_multi_thread()
    .worker_threads(10)
    .thread_name("connection listener")
    .enable_all()
    .thread_stack_size(32 * 1024 * 1024)
    .build();

    let runtime:Runtime;
    match builder{
        Ok(v)=>{runtime = v;},
        Err(_)=>{return Err(Error!("failed-start-tokio-runtime"));}
    }

    match runtime.block_on(async {
        start_server(d,config,channel_write).await
    }){
        Ok(_)=>{},
        Err(_)=>{}
    }

    Ok(())

}

use std::sync::mpsc::{Sender};

async fn start_server(d:Arc<Mutex<SharedData>>,config:ServerConfig,channel_write:Arc<Mutex<Sender<ThreadMessage>>>) -> Result<(),Error>{


    let listener = TcpListener::bind(format!("127.0.0.1:{:?}",config.socket)).await;
    match listener{
        Ok(server)=>{
            loop {
                match server.accept().await{
                    Ok((socket,_))=>{
                        process_socket(socket,&d,&config,&channel_write).await;
                    },
                    Err(_)=>{}
                }
            }
        },
        Err(_)=>{
            return Err(Error!("failed-start_server"));
        }
    }

}

use tokio_io_timeout::TimeoutStream;
use std::time::{Duration};
use gobject::{gObjectValue,gObject,gSchema};
use crate::messenger::MessageBox;
use tokio::io::Interest;

async fn process_socket(stream_base:TcpStream,d:&Arc<Mutex<SharedData>>,config:&ServerConfig,sender:&Arc<Mutex<Sender<ThreadMessage>>>){

    match stream_base.ready(Interest::READABLE | Interest::WRITABLE).await{
        Ok(_)=>{},
        Err(_)=>{return;}
    }

    // let mut timer = TimeoutStream::new(stream_base);
    // timer.set_read_timeout(Some(Duration::from_millis(10)));

    let mut conn = MessageBox::new(stream_base);

    let protocol:gObject;
    match conn.read_first_doc(10).await{
        Ok(v)=>{protocol = v;},
        Err(_)=>{
            // println!("protocol failed");
            return;
        }
    }

    match gSchema!{
        "connect"=>gSchemaValue::bool,
        "protocol"=>gSchemaValue::string
    }.validate(&protocol){
        Ok(_)=>{},
        Err(_)=>{
            return;
        }
    }

    let _error:Error;
    if protocol["protocol"].string().unwrap() == "key"{
        match validate_key(&mut conn,&config).await{
            Ok(_)=>{
                match sender.lock(){
                    Ok(sender_unlocked)=>{
                        match sender_unlocked.send(ThreadMessage::MessageBox(conn)){
                            Ok(_)=>{
                                // println!("message box sent");
                                return;
                            },
                            Err(_)=>{_error = Error!("failed-pass_conn_to_read");}
                        }
                    },
                    Err(_)=>{_error = Error!("failed-lock_sender");}
                }
            },
            Err(e)=>{_error = Error!("failed-validate_key"=>e);}
        }
        println!("{:?}",_error);
    } else
    if protocol["protocol"].string().unwrap() == "session"{
        validate_session(&mut conn,&d).await;
    }

}

use rand::{distributions::{Alphanumeric,Uniform}, Rng};
use crate::crypt;
use openssl::pkey::PKey;
use openssl::pkey::{Public};

async fn validate_key(connection:&mut MessageBox,config:&ServerConfig) -> Result<(),Error>{

    //------------------------------------------
    //send connection confirmation

    match connection.send(gObject!{
        "connect"=>gObjectValue::bool(true)
    }.build()).await{
        Ok(_)=>{},
        Err(_)=>{
            return Err(Error!("failed-request-connect"));
        }
    }

    //------------------------------------------
    //make challenge

    let send_challenge:gObject;
    match connection.read_first_doc(10).await{
        Ok(v)=>{
            match gSchema!{
                "get_challenge"=>gSchemaValue::bool
            }.validate(&v){Ok(_)=>{send_challenge = v;},Err(_)=>{
                return Err(Error!("failed-request-send_challenge"));
            }}
        },
        Err(_)=>{
            return Err(Error!("failed-request-send_challenge"));
        }
    }
    match send_challenge["get_challenge"]{
        gObjectValue::bool(v)=>{
            if v != true{
                return Err(Error!("failed-request-send_challenge"));
            }
        },
        _=>{
            return Err(Error!("failed-request-send_challenge"));
        }
    }

    let challenge: String = rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(7)
    .map(char::from)
    .collect();

    match connection.send(gObject!{
        "challenge"=>gObjectValue::string(challenge.clone())
    }.build()).await{
        Ok(_)=>{},
        Err(_)=>{
            return Err(Error!("failed-request-challenge"));
        }
    }

    //------------------------------------------
    //get signature

    let signature_object:gObject;
    match connection.read_first_doc(10).await{
        Ok(v)=>{
            match gSchema!{
                "signature"=>gSchemaValue::binary
            }.validate(&v){Ok(_)=>{signature_object = v;},Err(_)=>{
                return Err(Error!("failed-validate-signature"));
            }}
        },
        Err(_)=>{
            return Err(Error!("failed-request-signature"));
        }
    }
    let signature:Vec<u8>;
    match &signature_object["signature"]{
        gObjectValue::binary(v)=>{
            signature = v.to_vec();
        },
        _=>{
            return Err(Error!("failed-extract-signature"));
        }
    }

    //------------------------------------------
    //verify signature

    let pub_key:&PKey<Public>;
    match &config.public_key{
        RsaKey::public(v)=>{pub_key = v;},
        _=>{
            return Err(Error!("failed-extract-public_key"));
        }
    }

    match crypt::sign::verify(challenge.as_bytes().to_vec(),&pub_key,signature){
        Ok(_)=>{},
        Err(_)=>{
            return Err(Error!("failed-verify-sig"));
        }
    }

    //------------------------------------------
    //send password


    let password: Vec<u8> = rand::thread_rng().sample_iter(&Uniform::from(0..=255)).take(32).collect();
    let encrypted_password:gObjectValue;
    match crypt::rsa::encrypt(password.clone(),&pub_key){
        Ok(v)=>{
            encrypted_password = gObjectValue::binary(v);
        },
        Err(_)=>{
            return Err(Error!("failed-encypt-password"));
        }
    }

    match connection.send(gObject!{
        "password"=>encrypted_password
    }.build()).await{
        Ok(_)=>{},
        Err(_)=>{
            return Err(Error!("failed-request-password"));
        }
    }

    //------------------------------------------
    //get start message

    let start_object;
    match connection.read_first_doc(10).await{
        Ok(v)=>{
            match gSchema!{
                "start"=>gSchemaValue::bool
            }.validate(&v){Ok(_)=>{start_object = v;},Err(_)=>{
                return Err(Error!("failed-validate-start"));
            }}
        },
        Err(_)=>{
            return Err(Error!("failed-request-start"));
        }
    }
    let start:bool;
    match &start_object["start"]{
        gObjectValue::bool(v)=>{
            start = v.clone();
        },
        _=>{
            return Err(Error!("failed-parse-start"));
        }
    }

    if start == false{
        return Err(Error!("failed-denied-start"));
    }

    connection.add_password(password);
    
    return Ok(());

}

async fn validate_session(_connection:&mut MessageBox,_d:&Arc<Mutex<SharedData>>){

    

}