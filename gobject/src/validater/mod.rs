

pub mod builder;

// pub use builder;

pub use builder::{gSchema,gSchemaValue};

use crate::{gObject,Error,gObjectValue};

pub fn validate(schema:&gSchema,object:&gObject) -> Result<(),Error>{

    for key in schema.keys() {

        let base = &schema[&key];
        let check = &object[&key];

        match base{

            gSchemaValue::null=>{
                match check{gObjectValue::null=>{},_=>{return Err(Error!(&key));}}
            },
            gSchemaValue::undefined=>{
                match check{gObjectValue::undefined=>{},_=>{return Err(Error!(&key));}}
            },
            gSchemaValue::string=>{
                match check{gObjectValue::string(_)=>{},_=>{return Err(Error!(&key));}}
            },
            gSchemaValue::array=>{
                match check{gObjectValue::array(_)=>{},_=>{return Err(Error!(&key));}}
            },
            gSchemaValue::array_check(round_schema)=>{
                match check{gObjectValue::array(v)=>{
                    for array_item in v.iter(){
                        
                        match array_item{
                            gObjectValue::object(array_item)=>{
                                match validate(round_schema,&array_item){
                                    Ok(_)=>{},
                                    Err(e)=>{
                                        return Err(Error!(&key=>e));
                                    }
                                }
                            },
                            _=>{return Err(Error!(&key));}
                        }

                    }
                },_=>{return Err(Error!(&key));}}
            },
            gSchemaValue::binary=>{
                match check{gObjectValue::binary(_)=>{},_=>{return Err(Error!(&key));}}
            },
            gSchemaValue::bool=>{
                match check{gObjectValue::bool(_)=>{},_=>{return Err(Error!(&key));}}
            },
            gSchemaValue::object=>{
                match check{gObjectValue::object(_)=>{},_=>{return Err(Error!(&key));}}
            },
            gSchemaValue::object_check(next_schema)=>{
                match check{
                    gObjectValue::object(next_object)=>{
                        match validate(next_schema,&next_object){
                            Ok(_)=>{},
                            Err(e)=>{
                                return Err(Error!(&key=>e));
                            }
                        }
                    },
                    _=>{return Err(Error!(&key));}
                }
            },

            gSchemaValue::i32=>{
                match check{gObjectValue::i32(_)=>{},_=>{return Err(Error!(&key));}}
            },
            gSchemaValue::i64=>{
                match check{gObjectValue::i64(_)=>{},_=>{return Err(Error!(&key));}}
            },
            gSchemaValue::i128=>{
                match check{gObjectValue::i128(_)=>{},_=>{return Err(Error!(&key));}}
            },

            gSchemaValue::u32=>{
                match check{gObjectValue::u32(_)=>{},_=>{return Err(Error!(&key));}}
            },
            gSchemaValue::u64=>{
                match check{gObjectValue::u64(_)=>{},_=>{return Err(Error!(&key));}}
            },
            gSchemaValue::u128=>{
                match check{gObjectValue::u128(_)=>{},_=>{return Err(Error!(&key));}}
            },

            gSchemaValue::f32=>{
                match check{gObjectValue::f32(_)=>{},_=>{return Err(Error!(&key));}}
            },
            gSchemaValue::f64=>{
                match check{gObjectValue::f64(_)=>{},_=>{return Err(Error!(&key));}}
            }

            _=>{}
        }//match base val with schem val


    }//loop keys in object

    Ok(())

}