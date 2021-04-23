
use std::option::Option;
use std::collections::HashMap;

#[allow(non_camel_case_types)]
#[macro_export]
macro_rules! gObject {

    ($($key:expr => $value:expr),* )=>{{
        use std::collections::HashMap;
        use gobject::value::{gObjectValue,gObject};
        let mut temp:HashMap<String,gObjectValue> = HashMap::new();
        $(temp.insert($key.to_string(),$value);)*
        gObject{map:temp}
    }};

    ( $object:expr => $value:expr => $($path:expr),* ) => {{

        use crate::value::{gObjectValue};

        let mut collect:Vec<&str> = vec![];
        $(collect.push($path);)*
        let last = collect.pop().unwrap();

        fn update_object(obj:&gObjectValue,map:Vec<&str>,key:&str) -> Result<gObjectValue,()>{
            if map.len() > 0{
                match obj{
                    gObjectValue::object(oval) => {
                        let first = map[0];
                        match oval.map.get(first){
                            Some(v)=>{
                                let mut remap = map;
                                remap.remove(0);
                                match update_object(v,remap,key){
                                    Ok(updated)=>{
                                        let mut nval = oval.clone();
                                        match nval.map.insert(first.to_string(),updated){
                                            Some(_)=>{return Ok(gObjectValue::object(nval));},
                                            None=>{return Err(());}
                                        }
                                    },
                                    Err(_)=>{return Err(());}
                                }
                            },
                            None=>{return Err(());}
                        }
                    }
                    _=>{return Err(());}
                }
            } else {
                // println!("here");

                match obj{
                    gObjectValue::object(oval) => {
                        let mut mval = oval.clone();
                        match mval.map.insert(key.to_string(),$value){
                            Some(_)=>{},
                            None=>{}
                        }
                        return Ok(gObjectValue::object(mval));
                    }
                    _=>{return Err(());}
                }
            }
        }//update function ends here

        let update = update_object(&gObjectValue::object($object),collect,last);
        match update{
            Ok(val)=>{
                match val{
                    gObjectValue::object(oval)=>{Ok(oval)},
                    _=>{Err(())}
                }
            },
            Err(_)=>{Err(())}
        }

    }};

}

#[allow(non_camel_case_types,dead_code)]
#[derive(Debug,Clone)]
pub enum gObjectValue{

    null,undefined,
    document(Vec<u8>),
    body(Vec<u8>),
    header(String),
    string(String),
    object(gObject),
    array(Vec<gObjectValue>),
    binary(Vec<u8>),
    bool(bool),

    i32(i32),i64(i64),i128(i128),
    u32(u32),u64(u64),u128(u128),
    f32(f32),f64(f64)

}

#[allow(dead_code)]
impl gObjectValue{
    pub fn string(&self) -> Option<String>{match self{gObjectValue::string(v)=>{Some(v.to_string())},_=>{None}}}
    pub fn array(&self) -> Option<Vec<gObjectValue>>{match self{gObjectValue::array(v)=>{Some(v.to_vec())},_=>{None}}}
    pub fn binary(&self) -> Option<Vec<u8>>{match self{gObjectValue::binary(v)=>{Some(v.to_vec())},_=>{None}}}
    pub fn bool(&self) -> Option<bool>{match self{gObjectValue::bool(v)=>{Some(v.clone())},_=>{None}}}
    pub fn i32(&self) -> Option<i32>{match self{gObjectValue::i32(v)=>{Some(v.clone())},_=>{None}}}
    pub fn i64(&self) -> Option<i64>{match self{gObjectValue::i64(v)=>{Some(v.clone())},_=>{None}}}
    pub fn i128(&self) -> Option<i128>{match self{gObjectValue::i128(v)=>{Some(v.clone())},_=>{None}}}
    pub fn u32(&self) -> Option<u32>{match self{gObjectValue::u32(v)=>{Some(v.clone())},_=>{None}}}
    pub fn u64(&self) -> Option<u64>{match self{gObjectValue::u64(v)=>{Some(v.clone())},_=>{None}}}
    pub fn u128(&self) -> Option<u128>{match self{gObjectValue::u128(v)=>{Some(v.clone())},_=>{None}}}
    pub fn f32(&self) -> Option<f32>{match self{gObjectValue::f32(v)=>{Some(v.clone())},_=>{None}}}
    pub fn f64(&self) -> Option<f64>{match self{gObjectValue::f64(v)=>{Some(v.clone())},_=>{None}}}
}

#[allow(non_camel_case_types)]
#[derive(Debug,Clone)]
pub struct gObject{
    pub map:HashMap<String,gObjectValue>
}

use crate::build;

#[allow(dead_code)]
impl gObject{
    pub fn build(self) -> Vec<u8>{
        return build::start(self);
    }
    pub fn new() -> gObject{
        gObject{
            map:HashMap::new()
        }
    }
    pub fn insert(&mut self,key:&str,value:gObjectValue){
        match self.map.insert(key.to_string(),value){
            Some(_)=>{},
            None=>{}
        }
    }
    pub fn get(&mut self,key:&str) -> &gObjectValue{
        match self.map.get(key){
            Some(v)=>{v},
            None=>{&gObjectValue::undefined}
        }
    }
    pub fn keys(&self) -> Vec<String>{
        let mut collect = Vec::new();
        for key in self.map.keys(){
            collect.push(key.to_string());
        }
        return collect;
    }
}

impl std::ops::Index<&str> for gObject{

    type Output = gObjectValue;

    fn index(&self, key:&str) -> &Self::Output{
        match self.map.get(key){
            Some(v)=>{v},
            None=>{&gObjectValue::undefined}
        }
    }

}

impl std::ops::Index<&str> for gObjectValue{

    type Output = gObjectValue;

    fn index(&self, key:&str) -> &Self::Output{
        match self{
            gObjectValue::object(vobj)=>{
                match vobj.map.get(key){
                    Some(v)=>{v},
                    None=>{&gObjectValue::undefined}
                }
            },
            _=>{&gObjectValue::undefined}
        }
    }

}
