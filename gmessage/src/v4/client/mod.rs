use crate::messenger::MessageBox;
use crate::common::Error;
use crate::Error;
use dashmap::DashMap;

mod auth;
mod config;
mod ipc;
mod request;
mod writer;
mod reader;
mod response;

// mod io;
// mod runner;

pub use config::ClientConfig;
pub use auth::AuthToken;
pub use request::RequestBuilder;
// pub use runner::Client;

use std::thread::spawn;
use tokio::sync::mpsc;
use tokio::net::tcp::{OwnedReadHalf,OwnedWriteHalf};

// use tokio::sync::Mutex;
use std::sync::Arc;
use ipc::WakerBook;
use std::collections::HashMap;


#[allow(dead_code)]
pub async fn start(config:ClientConfig) -> Result<RequestBuilder, Error> {

    let waker_book:Arc<DashMap<String,WakerBook>> = Arc::new(DashMap::new());

    let mut readers:Vec<(u32,OwnedReadHalf,Vec<u8>)> = Vec::new();
    let mut writers:HashMap<u32,OwnedWriteHalf> = HashMap::new();
    let mut passwords:HashMap<u32,Vec<u8>> = HashMap::new();

    for id in 1..=config.channels{

        // println!("{:?}",id);

        let mut connection:MessageBox;
        match MessageBox::connect(config.addr.clone()).await{
            Ok(v)=>{connection = v;},
            Err(e)=>{
                println!("{:?}",e);
                return Err(Error!("failed-connect-MessageBox"=>e));
            }
        }
        match config.auth_token.validate(&mut connection).await{
            Ok(_)=>{},
            Err(e)=>{
                return Err(Error!("failed-validate-AuthToken"=>e));
            }
        }

        let (read_half, write_half) = connection.stream.into_inner().into_split();
        let password_for_response = connection.password.clone();

        readers.push((id,read_half,password_for_response.clone()));
        // writers.push((id,write_half));
        // passwords.push((id,password_for_response));
        match passwords.insert(id,password_for_response){
            Some(_)=>{},None=>{}
        }
        match writers.insert(id,write_half){
            Some(_)=>{},None=>{}
        }

    }
    
    let (writer_sender,writer_receiver) = mpsc::channel::<ipc::ThreadMessage>(10000);
    let (response_sender,response_receiver) = mpsc::channel::<ipc::ThreadMessage>(10000);
    

    let request_builder = RequestBuilder::new(
        config.channels,
        // connection.id.clone(),
        // connection.password.clone(),
        passwords.clone(),
        writer_sender,
        waker_book.clone()
    );

    tokio::spawn(async move {
        writer::start(writer_receiver,writers).await
    });
    for (_,read_half,password) in readers{
        let local_response_sender = response_sender.clone();
        tokio::spawn(async move {
            reader::start(read_half,local_response_sender,password).await
        });
    }
    // tokio::spawn(async move {
    //     reader::start(read_half,response_sender).await
    // });
    spawn(move || {
        match response::start(response_receiver,waker_book){
            Ok(_)=>{},
            Err(_)=>{}
        }
    });

    return Ok(request_builder);

    // return Err(Error!("no_error"));

}