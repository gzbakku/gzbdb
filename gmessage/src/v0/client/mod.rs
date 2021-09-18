
use crate::messenger::MessageBox;
use crate::common::Error;
use crate::Error;

mod auth;
mod config;
mod ipc;
mod io;
mod runner;

pub use config::ClientConfig;
pub use auth::AuthToken;
pub use runner::Client;

use std::thread::spawn;
// use std::sync::mpsc;
use tokio::sync::mpsc;


#[allow(dead_code)]
pub async fn start(config:ClientConfig) -> Result<Client, Error> {

    let mut connection:MessageBox;
    match MessageBox::connect(config.addr).await{
        Ok(v)=>{connection = v;},
        Err(e)=>{
            println!("{:?}",e);
            return Err(Error!("failed-connect-MessageBox"=>e));
        }
    }

    match config.auth_token.validate(&mut connection).await{
        Ok(_)=>{println!("auth successfull");},
        Err(e)=>{
            return Err(Error!("failed-validate-AuthToken"=>e));
        }
    }

    let (write,read) = mpsc::channel::<ipc::ThreadMessage>(32 * 1024);

    let connection_id = connection.id.clone();
    let connection_password = connection.password.clone();
    spawn(move|| {
        match io::start(connection,read){
            Ok(_)=>{},
            Err(_)=>{}
        }
    });

    return Ok(Client::new(connection_id,write,connection_password));

}