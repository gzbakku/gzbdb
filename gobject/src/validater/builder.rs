use std::collections::HashMap;

#[allow(non_camel_case_types)]
#[macro_export]
macro_rules! gSchema {

    ($($key:expr => $value:expr),* )=>{{
        use std::collections::HashMap;
        use gobject::validater::{gSchema,gSchemaValue};
        let mut temp:HashMap<String,gSchemaValue> = HashMap::new();
        $(temp.insert($key.to_string(),$value);)*
        gSchema{map:temp}
    }};

    ( $object:expr => $value:expr => $($path:expr),* ) => {{

        use crate::value::{gSchemaValue};

        let mut collect:Vec<&str> = vec![];
        $(collect.push($path);)*
        let last = collect.pop().unwrap();

        fn update_object(obj:&gSchemaValue,map:Vec<&str>,key:&str) -> Result<gSchemaValue,()>{
            if map.len() > 0{
                match obj{
                    gSchemaValue::object(oval) => {
                        let first = map[0];
                        match oval.map.get(first){
                            Some(v)=>{
                                let mut remap = map;
                                remap.remove(0);
                                match update_object(v,remap,key){
                                    Ok(updated)=>{
                                        let mut nval = oval.clone();
                                        match nval.map.insert(first.to_string(),updated){
                                            Some(_)=>{return Ok(gSchemaValue::object(nval));},
                                            false=>{return Err(());}
                                        }
                                    },
                                    Err(_)=>{return Err(());}
                                }
                            },
                            false=>{return Err(());}
                        }
                    }
                    _=>{return Err(());}
                }
            } else {
                // println!("here");

                match obj{
                    gSchemaValue::object(oval) => {
                        let mut mval = oval.clone();
                        match mval.map.insert(key.to_string(),$value){
                            Some(_)=>{},
                            false=>{}
                        }
                        return Ok(gSchemaValue::object(mval));
                    }
                    _=>{return Err(());}
                }
            }
        }//update function ends here

        let update = update_object(&gSchemaValue::object($object),collect,last);
        match update{
            Ok(val)=>{
                match val{
                    gSchemaValue::object(oval)=>{Ok(oval)},
                    _=>{Err(())}
                }
            },
            Err(_)=>{Err(())}
        }

    }};

}



#[allow(non_camel_case_types,dead_code)]
#[derive(Debug,Clone)]
pub enum gSchemaValue{

    null,undefined,
    document,
    body,
    header,
    string,
    object,
    object_check(gSchema),
    array,
    array_check(gSchema),
    binary,
    bool,

    i32,i64,i128,
    u32,u64,u128,
    f32,f64

}

#[allow(dead_code)]
impl gSchemaValue{
    pub fn string(&self) -> bool{match self{gSchemaValue::string=>{true},_=>{false}}}
    pub fn array(&self) -> bool{match self{gSchemaValue::array=>{true},_=>{false}}}
    pub fn binary(&self) -> bool{match self{gSchemaValue::binary=>{true},_=>{false}}}
    pub fn bool(&self) -> bool{match self{gSchemaValue::bool=>{true},_=>{false}}}
    pub fn i32(&self) -> bool{match self{gSchemaValue::i32=>{true},_=>{false}}}
    pub fn i64(&self) -> bool{match self{gSchemaValue::i64=>{true},_=>{false}}}
    pub fn i128(&self) -> bool{match self{gSchemaValue::i128=>{true},_=>{false}}}
    pub fn u32(&self) -> bool{match self{gSchemaValue::u32=>{true},_=>{false}}}
    pub fn u64(&self) -> bool{match self{gSchemaValue::u64=>{true},_=>{false}}}
    pub fn u128(&self) -> bool{match self{gSchemaValue::u128=>{true},_=>{false}}}
    pub fn f32(&self) -> bool{match self{gSchemaValue::f32=>{true},_=>{false}}}
    pub fn f64(&self) -> bool{match self{gSchemaValue::f64=>{true},_=>{false}}}
}

#[allow(non_camel_case_types)]
#[derive(Debug,Clone)]
pub struct gSchema{
    pub map:HashMap<String,gSchemaValue>
}

use crate::validater::{validate as validate_func,validate_value};
use crate::{Error,gObject,gObjectValue};

#[allow(dead_code)]
impl gSchema{
    pub fn new() -> gSchema{
        gSchema{
            map:HashMap::new()
        }
    }
    pub fn insert(&mut self,key:&str,value:gSchemaValue){
        match self.map.insert(key.to_string(),value){
            Some(_)=>{},
            None=>{}
        }
    }
    pub fn validate(&self,object:&gObject) -> Result<(),Error>{
        validate_func(&self,&object)
    }
    pub fn validate_value(&self,value:&gObjectValue) -> Result<(),Error>{
        validate_value(&self,&value)
    }
    pub fn keys(&self) -> Vec<String>{
        let mut collect = Vec::new();
        for key in self.map.keys(){
            collect.push(key.to_string());
        }
        return collect;
    }
}

impl std::ops::Index<&str> for gSchema{

    type Output = gSchemaValue;

    fn index(&self, key:&str) -> &Self::Output{
        match self.map.get(key){
            Some(v)=>{v},
            None=>{&gSchemaValue::undefined}
        }
    }

}