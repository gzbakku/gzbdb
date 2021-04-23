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
    pub fn new(s:TimeoutStream<TcpStream>)->MessageBox{
        let id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
        MessageBox{
            password:Vec::new(),
            id:id,
            stream:s,
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
                // println!("read_first_doc ok");
                match v{
                    gObjectValue::object(v)=>{
                        return Ok(v);
                    },
                    _=>{return Err(Error!("not_found-gObject"));}
                }
            },
            Err(e)=>{
                // println!("read_first_doc error");
                return Err(Error!("failed-read_first"=>e));
            }
        }
    }
    pub async fn read_first(&mut self,timeout:u64) ->Result<gObjectValue,Error>{
        match self.read(timeout).await{
            Ok(v)=>{
                // println!("read worked");
                if v.len() == 0{
                    // println!("read_first not_found");
                    return Err(Error!("failed-not_found"));
                } else {
                    // println!("read_first success");
                    return Ok(v[0].clone());
                }
            },
            Err(e)=>{
                // println!("read_first error");
                return Err(Error!("failed-read_from_socket"=>e));
            }
        }
    }
    pub async fn read(&mut self,timeout:u64) -> Result<Vec<gObjectValue>,Error>{

        let start_at = Instant::now();
        let stream = self.stream.get_mut();
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
                Err(_)=>{}
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
    pub fn try_read(&mut self) -> Result<Vec<gObjectValue>,Error>{

        let stream = self.stream.get_mut();
        let mut base = self.overflow.clone();
        let mut read_some = false;
        
        loop {

            let mut try_buff = [0;100];
            match stream.try_read(&mut try_buff){
                Ok(t)=>{
                    if t == 0 {
                        if !read_some{
                            if self.last_active.elapsed().as_secs() > 21600{
                                return Err(Error!("closed"));
                            }
                        } else {
                            self.overflow = base;
                            self.last_active = Instant::now();
                            return Ok(vec![]);
                        }
                    } else {
                        if !read_some{read_some = true;}
                        for i in 0..t{
                            base.push(try_buff[i]);
                        }
                    }
                },
                Err(e)=>{
                    // println!("e : {:?} {:?}",read_some,e);
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

            match parse(&base){
                Ok(blocks)=>{
                    if blocks.len() > 0{
                        let overflow = blocks.get_overflow();
                        if overflow < base.len(){
                            let mut collect_overflow = Vec::new();
                            for i in overflow+1..base.len(){
                                collect_overflow.push(base[i]);
                            }
                            self.overflow = collect_overflow;
                        } else {
                            self.overflow = vec![];
                        }
                        return Ok(blocks.get());
                    }
                },
                Err(_)=>{}
            }

        }//loop

    }
}