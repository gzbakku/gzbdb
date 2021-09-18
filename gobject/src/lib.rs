
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

#[cfg(test)]
mod tests{

    use crate::{gObjectValue,gObject,parse,Error,gSchema,gSchemaValue};

    #[test]
    fn make_g_object_struct(){
        let mut make = gObject::new();
        make.insert("one",gObjectValue::string("some string".to_string()));
        println!("gObject : {:?}",make);
    }

    #[test]
    fn make_g_object_macro(){

        let macro_make = gObject!{

            "object_tree"=>gObjectValue::object(gObject!{
                "one"=>gObjectValue::object(gObject!{
                    "two"=>gObjectValue::object(gObject!{
                        "three"=>gObjectValue::object(gObject!{
                            "string"=>gObjectValue::string("akku".to_string())
                        })
                    })
                })
            }),

            "pool" => gObjectValue::array(vec![
                gObjectValue::null,
                gObjectValue::string("ak".to_string()),
                gObjectValue::string("me".to_string())
            ]),

            "binary"=>gObjectValue::binary(vec![1,2,3]),

            "string"=>gObjectValue::string("akku".to_string()),
            "null"=>gObjectValue::null,
            "undefined"=>gObjectValue::undefined,
            "bool"=>gObjectValue::bool(false),

            "i32"=>gObjectValue::i32(213232),
            "i64"=>gObjectValue::i64(4212312351),
            "i128"=>gObjectValue::i128(564564434343464),
            "u32"=>gObjectValue::u32(420),
            "u64"=>gObjectValue::u64(4251),
            "u128"=>gObjectValue::u128(56456464),
            "f32"=>gObjectValue::f32(3434.23),
            "f64"=>gObjectValue::f64(21321323.5656)

        };

        println!("macro_make : {:?}",macro_make);

    }

    #[test]
    fn validate_gobject(){
        let data = gObject!{
            "name"=>gObjectValue::string("akku".to_string()),
            "array"=>gObjectValue::array(vec![
                gObjectValue::object(gObject!{
                    "one"=>gObjectValue::i32(12),
                    "two"=>gObjectValue::i64(12)
                }),
                gObjectValue::object(gObject!{
                    "one"=>gObjectValue::i32(12),
                    "two"=>gObjectValue::i64(12)
                })
            ]),
            "bool"=>gObjectValue::bool(false),
            "data"=>gObjectValue::object(gObject!{
                "one"=>gObjectValue::i32(12),
                "two"=>gObjectValue::i64(12),
                "three"=>gObjectValue::i128(12)
            }),
            "bank"=>gObjectValue::object(gObject!{
                "one"=>gObjectValue::i32(12),
                "two"=>gObjectValue::i64(12)
            })
        };
    
        let schema = gSchema!{
            "name"=>gSchemaValue::string,
            "array"=>gSchemaValue::array_check(gSchema!{
                "one"=>gSchemaValue::i32,
                "two"=>gSchemaValue::i64
            }),
            "bool"=>gSchemaValue::bool,
            "data"=>gSchemaValue::object_check(gSchema!{
                "one"=>gSchemaValue::i32,
                "two"=>gSchemaValue::i64,
                "three"=>gSchemaValue::i128
            }),
            "bank"=>gSchemaValue::object
        };
    
        println!("{:?}",schema.validate(&data));
    }

    #[test]
    fn get_g_object_value_from_g_object_tree(){

        let macro_make = gObject!{
            "object_tree"=>gObjectValue::object(gObject!{
                "one"=>gObjectValue::object(gObject!{
                    "two"=>gObjectValue::object(gObject!{
                        "three"=>gObjectValue::object(gObject!{
                            "string"=>gObjectValue::string("akku".to_string())
                        })
                    })
                })
            })
        };

        let string_from_from_third_nexted_object = macro_make["object_tree"]["one"]["two"]["three"]["string"].string();

        println!("string_from_from_third_nexted_object : {:?}",string_from_from_third_nexted_object);

    }

    #[test]
    fn update_g_object_value_in_nested_g_object_tree(){

        let macro_make = gObject!{
            "object_tree"=>gObjectValue::object(gObject!{
                "one"=>gObjectValue::object(gObject!{
                    "two"=>gObjectValue::object(gObject!{
                        "three"=>gObjectValue::object(gObject!{
                            "string"=>gObjectValue::string("akku".to_string())
                        })
                    })
                })
            })
        };

        //this returns a new gObject and consumes the previous one
        let updated = gObject!(macro_make => gObjectValue::string("new akku".to_string()) => "object_tree","one","two","three","string");
        
        println!("updated gObject : {:#?}",updated);

    }

    #[test]
    fn parse_g_object_to_binary(){

        let macro_make = gObject!{
            "string"=>gObjectValue::string("akku".to_string()),
            "null"=>gObjectValue::null,
            "undefined"=>gObjectValue::undefined,
            "bool"=>gObjectValue::bool(false)
        };

        let built = macro_make.build();

        println!("g_object_as_binary_vector_u8 : {:?}",built);

    }

    #[test]
    fn parse_binary_vector_u8_to_g_object(){

        let macro_make = gObject!{
            "string"=>gObjectValue::string("akku".to_string()),
            "null"=>gObjectValue::null,
            "undefined"=>gObjectValue::undefined,
            "bool"=>gObjectValue::bool(false)
        };

        let built = macro_make.build();

        let parsed = parse(&built).unwrap();

        println!("g_object_as_binary_vector_u8 : {:?}",parsed);

    }

    #[test]
    fn test_over_under_over_flow(){

        let macro_make = gObject!{
            "string"=>gObjectValue::string("akku".to_string()),
            "null"=>gObjectValue::null,
            "undefined"=>gObjectValue::undefined,
            "bool"=>gObjectValue::bool(false)
        };

        let built = macro_make.build();

        let mut make_fail_case:Vec<u8> = Vec::new();

        make_fail_case.extend(&vec![1,2,3]);
        make_fail_case.extend(&built);
        make_fail_case.extend(&vec![1,2,3]);

        let parsed = parse(&make_fail_case).unwrap();

        println!("overflow = {:?}",parsed.get_overflow());
        println!("underflow = {:?}",parsed.get_underflow());

        println!("g_object_as_binary_vector_u8 : {:?}",parsed);

    }

    #[test]
    fn make_error_with_macro(){
        let error = Error!("first error");
        let chain_error = Error!("secondError"=>error);
        println!("chain_error : {:?}",chain_error);
    }

}
