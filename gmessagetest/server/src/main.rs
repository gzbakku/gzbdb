
// use gmessage::{{server},query};
use tokio;
use gobject::{gObject};
use gmessage::util::{TokioMutex,Arc};
use gmessage::v2;

use gmessage::query;
use v2::server;

#[tokio::main]
async fn main() {

    let keys_dir = "D:/workstation/expo/rust/gzbdb/keys";
    let private_key_path = format!("{}/private.pem",keys_dir);
    let public_key_path = format!("{}/public.pem",keys_dir);

    let server_config;
    match server::ServerConfig::new(
        1200, private_key_path, public_key_path
    ).await{
        Ok(v)=>{server_config = v;},
        Err(e)=>{
            println!("{:?}",e);
            return;
        }
    }

    println!("starting server");

    match server::start(server_config,process_request,Vec::new()){
        Ok(_)=>{},
        Err(_)=>{}
    }

}

async fn process_request(_shared:Arc<TokioMutex<server::SharedData<Vec<u8>>>>,request:query::Request) -> query::Response{

    // println!("request : {:?}",request);

    // query::Response::encrypted(&request,gObject!{
    //     "one"=>gObjectValue::bool(true)
    // })

    query::Response::data(&request,gObject!{
        "one"=>gObjectValue::bool(true)
    })

}
