
use byteorder::{BigEndian, WriteBytesExt,ReadBytesExt};
use crate::Value;
use crate::mapify::Pointer;
use std::io::{Cursor};

pub fn parse_frame_to_pointer(pool:&Vec<u8>,start:u64,end:u64)->Result<Pointer,()>{

    let len = pool.len();

    //----------------------------------
    //check starting and ending flags
    //----------------------------------
    if 
        pool[0] != 0 ||
        pool[1] != 1 ||
        pool[2] != 0 ||
        pool[3] != 0 ||
        pool[len-1] != 0 ||
        pool[len-2] != 2 ||
        pool[len-3] != 0
    {
        println!("invalid pool parse_frame_to_pointer");
        return Err(());
    }

    //----------------------------------
    //get key cursor
    //----------------------------------
    let key_size:u64;
    let key_cursor:u64;
    match read_size_int(&pool,4){
        Ok((n,c))=>{
            key_size = n;
            key_cursor = c;
        },
        Err(_)=>{
            println!("failed read_size_int key parse_frame_to_pointer");
            return Err(());
        }
    }

    //----------------------------------
    //get value type
    //----------------------------------
    let value_type:u8 = pool[(key_cursor+key_size+3) as usize] as u8;
    let value_type_cursor = key_cursor+key_size+3;

    //----------------------------------
    //get value cursor
    //----------------------------------
    let value_size:u64;
    let value_cursor:u64;
    let value_size_int_cursor = value_type_cursor+2;
    match read_size_int(&pool,value_size_int_cursor as usize){
        Ok((n,c))=>{
            value_size = n;
            value_cursor = c;
        },
        Err(_)=>{
            println!("!!! failed read_size_int value parse_frame_to_pointer");
            return Err(());
        }
    }

    let value_start = value_cursor+2;
    let value_end = value_cursor+value_size+1;
    let value_len = value_end - value_start + 1;

    //----------------------------------
    //make pointer
    //----------------------------------

    return Ok(Pointer{
        start:start,
        end:end,
        value_start:start+value_start,
        value_end:start+value_end,
        value_size:value_len,
        value_type:value_type
    });

}

pub fn read_size_int(pool:&Vec<u8>,key_val_size_int_pos:usize) -> Result<(u64,u64),()>{

    let key_val_size_int = pool[key_val_size_int_pos] as usize;

    let mut collect:Vec<u8> = Vec::new();

    for i in key_val_size_int_pos+2..=key_val_size_int_pos+key_val_size_int+1{
        collect.push(pool[i as usize]);
    }

    match read_u64_num(collect){
        Ok(v)=>{
            return Ok((v,(key_val_size_int_pos+key_val_size_int+1) as u64));
        },
        Err(e)=>{
            println!("!!! failed-read_u64_num-read_size_int => {:?}",e);
            return Err(());
        }
    }

}

pub fn parse_keyval(key:&String,value:&Value)->Vec<u8>{
    let mut collect:Vec<u8> = vec![0,1,0];
    let mut key_len_as_bytes = Vec::new();
    match key_len_as_bytes.write_u64::<BigEndian>(key.len() as u64){
        Ok(_)=>{},
        Err(_)=>{}
    }
    collect.push(0);
    collect.push(key_len_as_bytes.len() as u8);
    collect.push(0);
    collect.append(&mut key_len_as_bytes);
    collect.push(0);
    collect.append(&mut key.as_bytes().to_vec());
    match value{
        Value::String(v)=>{
            collect.append(&mut vec![0,3]);
            let mut value_len_as_bytes = Vec::new();
            match value_len_as_bytes.write_u64::<BigEndian>(v.len() as u64){
                Ok(_)=>{},
                Err(_)=>{}
            }
            collect.push(0);
            collect.push(value_len_as_bytes.len() as u8);
            collect.push(0);
            collect.append(&mut value_len_as_bytes);
            collect.push(0);
            collect.append(&mut v.as_bytes().to_vec());
        },
        Value::Binary(v)=>{
            collect.append(&mut vec![0,4]);
            let mut value_len_as_bytes = Vec::new();
            match value_len_as_bytes.write_u64::<BigEndian>(v.len() as u64){
                Ok(_)=>{},
                Err(_)=>{}
            }
            collect.push(0);
            collect.push(value_len_as_bytes.len() as u8);
            collect.push(0);
            collect.append(&mut value_len_as_bytes);
            collect.push(0);
            collect.append(&mut v.clone());
        },
        Value::I64(v)=>{
            collect.append(&mut vec![0,5]);
            let mut value_as_bytes = Vec::new();
            match value_as_bytes.write_i64::<BigEndian>(*v){
                Ok(_)=>{},
                Err(_)=>{
                    value_as_bytes = vec![0];
                }
            }
            let mut value_len_as_bytes = Vec::new();
            match value_len_as_bytes.write_u64::<BigEndian>(value_as_bytes.len() as u64){
                Ok(_)=>{},
                Err(_)=>{}
            }
            collect.push(0);
            collect.push(value_len_as_bytes.len() as u8);
            collect.push(0);
            collect.append(&mut value_len_as_bytes);
            collect.push(0);
            collect.append(&mut value_as_bytes);
        },
        Value::U64(v)=>{
            collect.append(&mut vec![0,6]);
            let mut value_as_bytes = Vec::new();
            match value_as_bytes.write_u64::<BigEndian>(*v){
                Ok(_)=>{},
                Err(_)=>{
                    value_as_bytes = vec![0];
                }
            }
            let mut value_len_as_bytes = Vec::new();
            match value_len_as_bytes.write_u64::<BigEndian>(value_as_bytes.len() as u64){
                Ok(_)=>{},
                Err(_)=>{}
            }
            collect.push(0);
            collect.push(value_len_as_bytes.len() as u8);
            collect.push(0);
            collect.append(&mut value_len_as_bytes);
            collect.push(0);
            collect.append(&mut value_as_bytes);
        },
        Value::F64(v)=>{
            collect.append(&mut vec![0,7]);
            let mut value_as_bytes = Vec::new();
            match value_as_bytes.write_f64::<BigEndian>(*v){
                Ok(_)=>{},
                Err(_)=>{
                    // value_as_bytes = vec![0];
                }
            }
            let mut value_len_as_bytes = Vec::new();
            match value_len_as_bytes.write_u64::<BigEndian>(value_as_bytes.len() as u64){
                Ok(_)=>{},
                Err(_)=>{}
            }
            collect.push(0);
            collect.push(value_len_as_bytes.len() as u8);
            collect.push(0);
            collect.append(&mut value_len_as_bytes);
            collect.push(0);
            collect.append(&mut value_as_bytes);
        }
    }

    collect.append(&mut vec![0,2,0]);

    return collect;
}

pub fn read_u64_num(pool:Vec<u8>)->Result<u64,()>{
    let mut rdr = Cursor::new(pool);
    match rdr.read_u64::<BigEndian>(){
        Ok(v)=>{return Ok(v)},
        Err(_e)=>{
            return Err(());
        }
    }
}