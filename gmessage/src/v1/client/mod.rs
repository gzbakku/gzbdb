
use crate::messenger::MessageBox;
use crate::common::Error;
use crate::Error;

mod auth;
mod config;
mod ipc;
mod request;
mod book;
mod writer;
mod reader;
mod response;

// mod io;
// mod runner;

pub use config::ClientConfig;
pub use auth::AuthToken;
use tokio::sync::Mutex;
pub use request::RequestBuilder;
// pub use runner::Client;

use std::thread::spawn;
use tokio::sync::mpsc;

// use tokio::sync::Mutex;
use std::sync::Arc;

#[allow(dead_code)]
pub async fn start(config:ClientConfig) -> Result<RequestBuilder, Error> {

    let mut connection:MessageBox;
    match MessageBox::connect(config.addr).await{
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
    let (book_sender,book_receiver) = mpsc::channel::<ipc::ThreadMessage>(1000);
    let (writer_sender,writer_receiver) = mpsc::channel::<ipc::ThreadMessage>(1000);
    let (response_sender,response_receiver) = mpsc::channel::<ipc::ThreadMessage>(1000);

    let book_sender_lock = Arc::new(Mutex::new(book_sender.clone()));
    let password_for_response = connection.password.clone();

    let request_builder = RequestBuilder::new(
        connection.id.clone(),
        connection.password.clone(),
        book_sender,
        writer_sender
    );

    // let connection_id = connection.id.clone();
    // let connection_password = connection.password.clone();

    // let book_sender_lock = Arc::new(Mutex::new(book_sender));

    tokio::spawn(async move {
        book::start(book_receiver).await
    });
    tokio::spawn(async move {
        writer::start(writer_receiver,write_half).await
    });
    tokio::spawn(async move {
        reader::start(read_half,response_sender).await
    });
    spawn(move || {
        match response::start(response_receiver,book_sender_lock,password_for_response){
            Ok(_)=>{},
            Err(_)=>{}
        }
    });

    // return Ok(Client::new(connection_id,write,connection_password));

    return Ok(request_builder);

}