use tokio::runtime::{Builder,Runtime};
use crate::v4::client::ipc::{ThreadMessage,ResponseAdd,WakerBook,ResponseHolderValue};
use crate::common::Error;
use crate::Error;
use tokio::sync::mpsc::{Receiver};
use tokio::sync::{RwLock};
use std::sync::Arc;
use crate::query::{Response,ResponseType};
use crate::crypt::aes::Encrypted;
use gobject::{gObjectValue,gObject};
use dashmap::DashMap;

pub fn start(
    receiver:Receiver<ThreadMessage>,
    // book_sender:Arc<Mutex<Sender<ThreadMessage>>>,
    // password:Vec<u8>,
    waker_book:Arc<DashMap<String,WakerBook>>
) -> Result<(),Error>{

    // let password_read_lock = Arc::new(RwLock::new(password));

    let builder = Builder::new_multi_thread()
    .worker_threads(400)
    // .worker_threads(200)
    .thread_name("response listener")
    .enable_all()
    .thread_stack_size(4 * 1024 * 1024)
    .build();

    let runtime:Runtime;
    match builder{
        Ok(v)=>{runtime = v;},
        Err(_)=>{return Err(Error!("failed-start-tokio-runtime"));}
    }

    let mut local_receiver = receiver;

    runtime.block_on(async {
        // let hold_password_read_lock = password_read_lock.clone();
        let hold_waker_book = waker_book.clone();
        loop{
            // let local_password_read_lock = hold_password_read_lock.clone();
            let local_waker_book = hold_waker_book.clone();
            match local_receiver.recv().await{
                Some(msg)=>{
                    match msg{
                        ThreadMessage::ResponseAdd(data)=>{
                            runtime.spawn(async move {
                                let mut hold = data;
                                process_response(&mut hold,local_waker_book).await
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
    // password:Arc<RwLock<Vec<u8>>>,
    // book_sender:Arc<Mutex<Sender<ThreadMessage>>>,
    // password:Arc<RwLock<Vec<u8>>>,
    waker_book:Arc<DashMap<String,WakerBook>>
){

    // println!("processing response");

    match parse_response(&data.response, &data.password).await{
        Ok(parsed)=>{
            // println!("response received");
            // println!("{:?}",parsed);

            let request_id = parsed.request_id.clone();
            match waker_book.remove(&request_id){
                Some((_,worker))=>{
                    match worker.response_holder.lock(){
                        Ok(mut locked)=>{
                            locked.hold = ResponseHolderValue::Response(parsed);
                        },
                        Err(_)=>{return;}
                    }
                    worker.notify.notify_waiters();
                },
                None=>{
                    println!("no request id found");
                }
            }

            // let locked = book_sender.lock().await;
            // match locked.send(ThreadMessage::BookFinish(BookFinish{
            //     response:parsed
            // })).await{
            //     Ok(_)=>{
            //         // println!("book finish sent");
            //     },
            //     Err(_)=>{}
            // }
        },
        Err(_)=>{
            // println!("response parse failed");
        }
    }

}

async fn parse_response(obj:&gObjectValue,password_lock:&Arc<RwLock<Vec<u8>>>) -> Result<Response,Error>{

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