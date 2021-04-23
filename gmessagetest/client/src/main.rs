use tokio;
use gmessage::client;
use gobject::{gObject};

#[tokio::main]
async fn main(){

    let addr = "127.0.0.1:1200".to_string();
    let keys_dir = "D:/workstation/expo/rust/gzbdb/keys";
    let private_key_path = format!("{}/private.pem",keys_dir);
    let public_key_path = format!("{}/public.pem",keys_dir);

    let auth_token:client::AuthToken;
    match client::AuthToken::key(public_key_path,private_key_path).await{
        Ok(v)=>{auth_token = v;},
        Err(_)=>{return;}
    }

    let config = client::ClientConfig::new(auth_token,addr);
    
    let mut conn:client::Client;
    match client::start(config).await{
        Ok(v)=>{conn = v;},
        Err(_)=>{return}
    }

    for _ in 0..1{
        match conn.send(gObject!{
            "non"=>gObjectValue::null
        },3000,true){
            Ok(r)=>{
                println!("{:?}",r);
            },
            Err(e)=>{
                println!("e : {:?}",e);
            }
        }
    }
    
}
