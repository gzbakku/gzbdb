
// use tokio::runtime::Builder;

// use tokio;
use tonic;

mod server;

#[tokio::main]
async fn main(){
    
    server::start().await;

}