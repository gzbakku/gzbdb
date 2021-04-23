
use gmessage::{server,query};
use tokio;
use gobject::{gObject};

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

    match server::start(server_config,process_request){
        Ok(_)=>{},
        Err(_)=>{}
    }

}

async fn process_request(request:query::Request) -> query::Response{

    // println!("{:?}",request);

    query::Response::encrypted(&request,gObject!{
        "one"=>gObjectValue::bool(true)
    })

}
