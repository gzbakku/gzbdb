use tokio_io_timeout::TimeoutStream;
use tokio::net::TcpStream;
use std::time::{Duration,Instant};
use tokio::io::{AsyncWriteExt,AsyncReadExt};
use crate::Error;
use crate::common::Error;

pub struct MessageBox{
    pub password:Vec<u8>,
    pub stream:TimeoutStream<TcpStream>,
    pub overflow:Vec<u8>,
    pub id:String,
    pub last_active:Instant
}

use gobject::parse;
use tokio::time::sleep;
use tokio::io::Interest;
use gobject::{gObjectValue,gObject};
use rand::{distributions::Alphanumeric, Rng};
use std::io::ErrorKind;

impl MessageBox{
    // pub fn inner(&mut self) -> TcpStream{
    //     self.stream.into_inner()
    // }
    pub async fn connect(addr:String) -> Result<MessageBox,Error>{
        let stream_base:TcpStream;
        let id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
        match TcpStream::connect(addr).await{
            Ok(s)=>{stream_base = s;},
            Err(e)=>{
                return Err(Error!(format!("failed_start_connection=>{:?}",e)));
            }
        }
        match stream_base.ready(Interest::READABLE | Interest::WRITABLE).await{
            Ok(_)=>{},
            Err(e)=>{
                return Err(Error!(format!("failed-ready-connection=>{:?}",e)));
            }
        }
        let mut timer = TimeoutStream::new(stream_base);
        timer.set_read_timeout(Some(Duration::from_millis(1000)));
        return Ok(MessageBox{
            password:Vec::new(),
            id:id,
            stream:timer,
            overflow:Vec::new(),
            last_active:Instant::now()
        });
    }
    pub fn new(stream_base:TcpStream)->MessageBox{
        let mut timer = TimeoutStream::new(stream_base);
        timer.set_read_timeout(Some(Duration::from_millis(1000)));
        let id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
        MessageBox{
            password:Vec::new(),
            id:id,
            stream:timer,
            overflow:Vec::new(),
            last_active:Instant::now()
        }
    }
    pub fn add_password(&mut self,p:Vec<u8>){
        self.password = p;
    }
    pub async fn send(&mut self,d:Vec<u8>) -> Result<(),Error>{
        // println!("sending : {:?}",d);
        let stream = self.stream.get_mut();
        // let stream = &mut self.stream;
        match stream.write(&d).await{
            Ok(_)=>{
                // println!("sending : {:?}",w);
                return Ok(());
            },
            Err(e)=>{
                return Err(Error!(format!("failed-write_message=>{:?}",e)));
            }
        }
    }
    pub async fn send_b(&mut self,d:&Vec<u8>) -> Result<(),Error>{
        // println!("sending : {:?}",d);
        let stream = self.stream.get_mut();
        // let stream = &mut self.stream;
        match stream.write(&d).await{
            Ok(_)=>{
                // println!("sending : {:?}",w);
                return Ok(());
            },
            Err(e)=>{
                return Err(Error!(format!("failed-write_message=>{:?}",e)));
            }
        }
    }
    pub async fn read_first_doc(&mut self,timeout:u64) ->Result<gObject,Error>{
        match self.read_first(timeout).await{
            Ok(v)=>{
                match v{
                    gObjectValue::object(v)=>{
                        return Ok(v);
                    },
                    _=>{return Err(Error!("not_found-gObject"));}
                }
            },
            Err(e)=>{
                return Err(Error!("failed-read_first"=>e));
            }
        }
    }
    pub async fn read_first(&mut self,timeout:u64) ->Result<gObjectValue,Error>{
        match self.read(timeout).await{
            Ok(v)=>{
                if v.len() == 0{
                    return Err(Error!("failed-not_found"));
                } else {
                    return Ok(v[0].clone());
                }
            },
            Err(e)=>{
                return Err(Error!("failed-read_from_socket"=>e));
            }
        }
    }
    pub async fn read(&mut self,timeout:u64) -> Result<Vec<gObjectValue>,Error>{

        let start_at = Instant::now();
        let stream = self.stream.get_mut();
        // let stream = &mut self.stream;
        let mut base = self.overflow.clone();
        
        loop {

            let mut try_buff = [0;100];
            match stream.read(&mut try_buff).await{
                Ok(t)=>{
                    if t > 0{
                        // println!("t : {:?}",t);
                        for i in 0..t{
                            base.push(try_buff[i]);
                        }
                    }
                },
                Err(e)=>{
                    let err = Err(Error!("closed"));
                    match e.kind() {
                        ErrorKind::WouldBlock=>{return Err(Error!("blocking"));},
                        ErrorKind::ConnectionAborted=>{return err;},
                        ErrorKind::Interrupted=>{return err;},
                        ErrorKind::PermissionDenied=>{return err;},
                        ErrorKind::BrokenPipe=>{return err;},
                        ErrorKind::ConnectionReset=>{return err;},
                        ErrorKind::NotConnected=>{return err;},
                        _=>{}
                    }
                }
            }

            if base.len() > 0{
                // println!("l : {:?}",base.len());
                match parse(&base){
                    Ok(blocks)=>{
                        if blocks.len() > 0{
                            // println!("block found");
                            let overflow = blocks.get_overflow();
                            if overflow < base.len(){
                                let mut collect_overflow = Vec::new();
                                for i in overflow+1..base.len(){
                                    collect_overflow.push(base[i]);
                                }
                                self.overflow = collect_overflow;
                                // println!("block processed");
                            } else {
                                self.overflow = vec![];
                            }
                            return Ok(blocks.get());
                        }
                    },
                    Err(_)=>{}
                }
            }

            sleep(Duration::from_millis(10)).await;

            if start_at.elapsed().as_secs() > timeout{
                break;
            }

        }//loop

        // println!("conection ended");

        return Ok(vec![]);

    }//read
}

use tokio::net::tcp::OwnedReadHalf;
// use tokio::time::sleep;
// use core::time::Duration;

pub async fn read_tcp_stream(reader:&mut OwnedReadHalf,overflow:&mut Vec<u8>) -> Result<Vec<gObjectValue>,Error>{

    let mut base:Vec<u8> = overflow.to_vec();

    let mut wait:u64 = 10;
    
    loop {

        let mut try_buff = [0;100];
        match reader.read(&mut try_buff).await{
            Ok(t)=>{
                if t > 0{
                    // println!("t : {:?}",t);
                    for i in 0..t{
                        base.push(try_buff[i]);
                    }
                    wait = 0;
                } else {
                    if wait < 100{
                        wait += 10;
                    }
                    sleep(Duration::from_millis(wait)).await;
                    // println!("waiting");
                }
            },
            Err(e)=>{
                // println!("{:?}",e);
                let err = Err(Error!("closed"));
                match e.kind() {
                    ErrorKind::WouldBlock=>{return Err(Error!("blocking"));},
                    ErrorKind::ConnectionAborted=>{return err;},
                    ErrorKind::Interrupted=>{return err;},
                    ErrorKind::PermissionDenied=>{return err;},
                    ErrorKind::BrokenPipe=>{return err;},
                    ErrorKind::ConnectionReset=>{return err;},
                    ErrorKind::NotConnected=>{return err;},
                    _=>{}
                }
            }
        }

        // println!("reading tcp stream data");

        if base.len() > 0{
            // println!("l : {:?}",base.len());
            match parse(&base){
                Ok(blocks)=>{
                    // println!("blocks : {:?}",blocks.len());
                    if blocks.len() > 0{
                        // println!("block found");
                        let current_overflow = blocks.get_overflow();
                        if current_overflow < base.len(){
                            let mut collect_overflow = Vec::new();
                            for i in current_overflow+1..base.len(){
                                collect_overflow.push(base[i]);
                            }
                            // overflow = collect_overflow;
                            overflow.clear();
                            overflow.extend(&collect_overflow);
                            // println!("overflow : {:?}",overflow);
                            // println!("block processed");
                        } else {
                            overflow.clear();
                        }
                        return Ok(blocks.get());
                    }
                },
                Err(_)=>{
                    // println!("block not found");
                }
            }
        }

    }//loop

}//read