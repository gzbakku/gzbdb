use tokio::runtime::{Builder,Runtime};
use crate::v1::client::ipc::{ThreadMessage,ResponseAdd,BookFinish};
use crate::common::Error;
use crate::Error;
use tokio::sync::mpsc::{Receiver,Sender};
use tokio::sync::{Mutex,RwLock};
use std::sync::Arc;
use crate::query::{Response,ResponseType};
use crate::crypt::aes::Encrypted;
use gobject::{gObjectValue,gObject};

pub fn start(
    receiver:Receiver<ThreadMessage>,
    book_sender:Arc<Mutex<Sender<ThreadMessage>>>,
    password:Vec<u8>
) -> Result<(),Error>{

    let password_read_lock = Arc::new(RwLock::new(password));

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

    let mut local_receiver = receiver;

    runtime.block_on(async {
        let hold_password_read_lock = password_read_lock.clone();
        let hold_book_sender = book_sender.clone();
        loop{
            let local_password_read_lock = hold_password_read_lock.clone();
            let local_book_sender = hold_book_sender.clone();
            match local_receiver.recv().await{
                Some(msg)=>{
                    match msg{
                        ThreadMessage::ResponseAdd(data)=>{
                            runtime.spawn(async move {
                                let mut hold = data;
                                process_response(&mut hold,local_password_read_lock,local_book_sender).await
                            });
                        },
                        _=>{}
                    }
                },
                None=>{
                    // println!("response main receiver dropped");
                    break;
                }
            }
        }
    });

    Ok(())

}

async fn process_response(
    data:&mut ResponseAdd,
    password:Arc<RwLock<Vec<u8>>>,
    book_sender:Arc<Mutex<Sender<ThreadMessage>>>,
){

    // println!("processing response");

    match parse_response(&data.response, password).await{
        Ok(parsed)=>{
            let locked = book_sender.lock().await;
            match locked.send(ThreadMessage::BookFinish(BookFinish{
                response:parsed
            })).await{
                Ok(_)=>{
                    // println!("book finish sent");
                },
                Err(_)=>{}
            }
        },
        Err(_)=>{
            // println!("response parse failed");
        }
    }

}

async fn parse_response(obj:&gObjectValue,password_lock:Arc<RwLock<Vec<u8>>>) -> Result<Response,Error>{

    let response_type:i32;
    match &obj["response_type"]{
        gObjectValue::i32(v)=>{response_type = v.clone();},
        _=>{return Err(Error!("failed-extract-response_type"));}
    }

    let id:String;
    match &obj["id"]{
        gObjectValue::string(v)=>{id = v.clone();},
        _=>{return Err(Error!("failed-extract-id"));}
    }

    let data:gObject;
    match &obj["body"]{
        gObjectValue::object(v)=>{data = v.clone();},
        _=>{return Err(Error!("failed-extract-data"));}
    }

    let make:Response;
    if response_type == 0{
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Ok
        };
    } 
    else if response_type == 1{
        let body:gObject;
        match &data["data"]{
            gObjectValue::object(v)=>{body = v.clone();},
            _=>{return Err(Error!("invalid-body-data"));}
        }
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Data(body)
        };
    } 
    else if response_type == 2{
        let error:String;
        match &data["error"]{
            gObjectValue::string(v)=>{error = v.clone();},
            _=>{return Err(Error!("invalid-body-data"));}
        }
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Error(error)
        };
    } 
    else if response_type == 3{
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Down
        };
    } 
    else if response_type == 4{
        let encrypted:gObject;
        match &data["encrypted"]{
            gObjectValue::object(v)=>{encrypted = v.clone();},
            _=>{return Err(Error!("invalid-body-encrypted-data"));}
        }
        let unlocked:gObject;
        let password = password_lock.read().await;
        match Encrypted::load_from_g_object(&encrypted){
            Ok(e)=>{
                match e.unlock_first_g_object(&password){
                    Ok(doc)=>{
                        match doc{
                            gObjectValue::object(v)=>{unlocked = v;}
                            _=>{return Err(Error!("invalid-unlocked-g_object"));}
                        }
                    },
                    Err(e)=>{return Err(Error!("failed-unlock_first_g_object"=>e));}
                }
            },
            Err(e)=>{
                return Err(Error!("failed-load_from_g_object"=>e));
            }
        }
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Encrypted(unlocked)
        };
    } 
    else {return Err(Error!("failed-invalid-response_type"));}

    Ok(make)

}