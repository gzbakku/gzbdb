

pub mod build;
pub mod value;
mod common;
pub mod parser;
pub mod validater;
pub mod reader;

pub use value::{gObject,gObjectValue};
pub use common::Error;
pub use parser::start as parse;
pub use validater::{validate,gSchema,gSchemaValue};
pub use parser::{Blocks,Block};
pub use reader::{gObjectReader,gObjectReaderStatus};

fn main(){

    // let make = gObject!{
    //     "name"=>gObjectValue::string("akku".to_string()),
    //     "age"=>gObjectValue::f64(1.0)
    // };
    // let mut data = make.build();
    // let mut parts:Vec<Vec<u8>> = Vec::new();
    

    let mut reader = gObjectReader::new();

    if true{
        let mut parts:Vec<Vec<u8>> = Vec::new();
        let split_at = 100;
        for _ in 0..5{
            let mut make = gObject!{
                "name"=>gObjectValue::string("akku".to_string()),
                "age"=>gObjectValue::f64(1.0)
            }.build();
            loop{
                if make.len() > split_at{
                    let new = make.split_off(10);
                    parts.push(make);
                    make = new;
                } else {
                    parts.push(make);
                    break;
                }
            }
        }
        for part in parts{
            match reader.push(&part){
                gObjectReaderStatus::Doc=>{
                    println!("doc found");
                },
                _=>{}
            }
        }
    }

    if false{
        let mut multi_docs:Vec<u8> = Vec::new();
        for _ in 0..3{
            multi_docs.append(&mut gObject!{
                "name"=>gObjectValue::string("akku".to_string()),
                "age"=>gObjectValue::f64(1.0)
            }.build());
        }
        reader.push(&multi_docs);
    }

    // println!("{:?}",reader.pop());

}