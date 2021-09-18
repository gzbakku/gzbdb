
/*
    000 1 000 -  id len bytes num
    000 2 000 -  id len is length of an string converted to a bytes array represented in kb(f64 int)
    000 3 000 -  id is a string of len parsed in 2 flag
    000 4 000 -  data type
    000 5 000 -  data len bytes num
    000 6 000 -  data len is length of a array of bytes represented in kb(f64 int)
    000 7 000 -  data
    000 8 000
*/

use crate::value::{gObject,gObjectValue};
use md5;

pub fn start(obj:gObject) -> Vec<u8>{
    let body = parse_object_body(obj);
    let mut make:Vec<u8> = Vec::new();
    make.extend(&make_item("hash",&gObjectValue::header("".to_string())));
    make.extend(&make_item("body",&gObjectValue::body(body)));
    return make_item("gObject Document Schema : v1",&gObjectValue::document(make));
}

pub fn start_with_hash(obj:gObject) -> Vec<u8>{
    let body = parse_object_body(obj);
    let hash = format!("{:x}",md5::compute(&body));
    let mut make:Vec<u8> = Vec::new();
    make.extend(&make_item("hash",&gObjectValue::header(hash)));
    make.extend(&make_item("body",&gObjectValue::body(body)));
    return make_item("gObject Document Schema : v1",&gObjectValue::document(make));
}

pub fn parse_object_body(obj:gObject) -> Vec<u8>{
    let mut collect:Vec<u8> = Vec::new();
    for key in obj.map.keys(){
        match obj.map.get(key){
            Some(value)=>{
                collect.extend(&make_item(key,&value));
            },
            None=>{}
        }
    }
    collect
}

pub fn make_item(id:&str,value:&gObjectValue) -> Vec<u8>{

    let mut make:Vec<u8> = vec![];
    let id_as_bytes = id.as_bytes().to_vec();
    let id_size = get_size(id_as_bytes.len());
    let data_type_int = get_data_type_int(&value);
    let data = parse_data_to_bytes_array(&value);
    let data_size = get_size(data.len());

    push_flag(&mut make,1);make.push(id_size.no_of_bytes);
    push_flag(&mut make,2);make.extend(&id_size.num_as_bytes);
    push_flag(&mut make,3);make.extend(&id_as_bytes);
    push_flag(&mut make,4);make.push(data_type_int);
    push_flag(&mut make,5);make.push(data_size.no_of_bytes);
    push_flag(&mut make,6);make.extend(&data_size.num_as_bytes);
    push_flag(&mut make,7);make.extend(&data);
    push_flag(&mut make,8);

    fn push_flag(m:&mut Vec<u8>,f:u8){for _ in 0..3{m.push(0);}m.push(f);for _ in 0..3{m.push(0);}}

    make

}

#[derive(Debug,Clone)]
struct Size {
    size_in_kb:f64,
    num_as_bytes:Vec<u8>,
    no_of_bytes:u8
}

use std::f64;

fn get_size(base:usize) -> Size {
    let as_f64 = base.to_string().parse::<f64>().unwrap() / 1000.0;
    let as_bytes = as_f64.to_be_bytes().to_vec();
    let no_of_bytes = as_bytes.len().to_string().parse::<u8>().unwrap();
    Size{
        size_in_kb:as_f64,
        num_as_bytes:as_bytes,
        no_of_bytes:no_of_bytes
    }
}

fn parse_data_to_bytes_array(data:&gObjectValue) -> Vec<u8>{

    match data{

        gObjectValue::document(v)=>{v.to_vec()},
        gObjectValue::header(v)=>{v.as_bytes().to_vec()},
        gObjectValue::body(v)=>{v.to_vec()},
        gObjectValue::null=>{vec![]},
        gObjectValue::undefined=>{vec![]},

        gObjectValue::string(v)=>{v.as_bytes().to_vec()},
        gObjectValue::bool(v)=>{if !v {vec![0]} else {vec![1]}},
        gObjectValue::binary(v)=>{v.clone()},

        gObjectValue::object(v)=>{
            let mut collect:Vec<u8> = Vec::new();
            for key in v.map.keys(){
                match v.map.get(key){
                    Some(value)=>{
                        collect.extend(&make_item(key,&value));
                    },
                    None=>{}
                }
            }
            collect
        },
        gObjectValue::array(v)=>{
            let mut collect:Vec<u8> = Vec::new();
            for i in 0..v.len() {
                collect.extend(&make_item(&i.to_string(),&v[i]));
            }
            collect
        },

        gObjectValue::i32(v)=>{v.to_be_bytes().to_vec()},
        gObjectValue::i64(v)=>{v.to_be_bytes().to_vec()},
        gObjectValue::i128(v)=>{v.to_be_bytes().to_vec()},

        gObjectValue::u32(v)=>{v.to_be_bytes().to_vec()},
        gObjectValue::u64(v)=>{v.to_be_bytes().to_vec()},
        gObjectValue::u128(v)=>{v.to_be_bytes().to_vec()},

        gObjectValue::f32(v)=>{v.to_be_bytes().to_vec()},
        gObjectValue::f64(v)=>{v.to_be_bytes().to_vec()},

    }

}

fn get_data_type_int(data_type:&gObjectValue) -> u8{
    let data_type_int:u8;
    match data_type{

        gObjectValue::document(_)=>{data_type_int      = 1;}
        gObjectValue::header(_)=>{data_type_int        = 2;}
        gObjectValue::body(_)=>{data_type_int          = 3;}
        gObjectValue::null=>{data_type_int             = 4;}
        gObjectValue::undefined=>{data_type_int        = 5;}

        gObjectValue::string(_)=>{data_type_int    = 21;}
        gObjectValue::object(_)=>{data_type_int    = 22;}
        gObjectValue::array(_)=>{data_type_int     = 23;}
        gObjectValue::binary(_)=>{data_type_int    = 24;}
        gObjectValue::bool(_)=>{data_type_int      = 25;}

        gObjectValue::i32(_)=>{data_type_int       = 31;}
        gObjectValue::i64(_)=>{data_type_int       = 32;}
        gObjectValue::i128(_)=>{data_type_int      = 33;}

        gObjectValue::u32(_)=>{data_type_int       = 34;}
        gObjectValue::u64(_)=>{data_type_int       = 35;}
        gObjectValue::u128(_)=>{data_type_int      = 36;}

        gObjectValue::f32(_)=>{data_type_int       = 37;}
        gObjectValue::f64(_)=>{data_type_int       = 38;}

    }
    return data_type_int;
}
