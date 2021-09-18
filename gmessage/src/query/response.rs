

use gobject::{gObject,gObjectValue};
use gobject;
use crate::query::Request;
use crate::common::Error;
use crate::Error;
use crate::crypt::aes::encrypt;

#[derive(Debug,Clone)]
pub enum ResponseType{
    Ok,
    Data(gObject),
    Error(String),
    Down,
    Encrypted(gObject)
}

#[derive(Debug,Clone)]
pub struct Response{
    pub channel_id:String,
    pub request_id:String,
    pub data:ResponseType
}

#[allow(dead_code)]
impl Response{
    pub fn ok(req:&Request)->Response{
        Response{
            channel_id:req.channel_id.clone(),
            request_id:req.request_id.clone(),
            data:ResponseType::Ok
        }
    }
    pub fn data(req:&Request,d:gObject)->Response{
        Response{
            channel_id:req.channel_id.clone(),
            request_id:req.request_id.clone(),
            data:ResponseType::Data(d)
        }
    }
    pub fn error(req:&Request,e:String)->Response{
        Response{
            channel_id:req.channel_id.clone(),
            request_id:req.request_id.clone(),
            data:ResponseType::Error(e)
        }
    }
    pub fn down(req:&Request)->Response{
        Response{
            channel_id:req.channel_id.clone(),
            request_id:req.request_id.clone(),
            data:ResponseType::Down
        }
    }
    pub fn encrypted(req:&Request,d:gObject)->Response{
        Response{
            channel_id:req.channel_id.clone(),
            request_id:req.request_id.clone(),
            data:ResponseType::Encrypted(d)
        }
    }
    pub fn parse(&self,password:&Vec<u8>)->Result<gObject,Error>{

        let data:gObjectValue;
        let response_type:gObjectValue;
        match &self.data{
            ResponseType::Ok=>{
                response_type = gObjectValue::i32(0);
                data =  gObjectValue::object(gObject::new());
            },
            ResponseType::Data(v)=>{
                response_type = gObjectValue::i32(1);
                data = gObjectValue::object(gObject!{
                    "data"=> gObjectValue::object(v.clone())
                });
            },
            ResponseType::Error(e)=>{
                response_type = gObjectValue::i32(2);
                data = gObjectValue::object(gObject!{
                    "error"=> gObjectValue::string(e.clone())
                });
            },
            ResponseType::Down=>{
                response_type = gObjectValue::i32(3);
                data =  gObjectValue::object(gObject::new());
            },
            ResponseType::Encrypted(v)=>{
                response_type = gObjectValue::i32(4);
                match encrypt(&password,&v.clone().build()){
                    Ok(e)=>{
                        data = gObjectValue::object(gObject!{
                            "encrypted"=> gObjectValue::object(e.g_object())
                        });
                    },
                    Err(e)=>{
                        return Err(Error!("failed-encypt-response"=>e));
                    }
                }
            }
        }

        let make = gObject!{
            "id"=>gObjectValue::string(self.request_id.clone()),
            "response_type"=>response_type,
            "body"=>data
        };
        Ok(make)

    }
}