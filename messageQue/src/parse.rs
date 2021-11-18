
use std::io::{Cursor};
use byteorder::{BigEndian, WriteBytesExt,ReadBytesExt};

pub fn parse_message(map:&mut crate::Map,message:&str)->Vec<u8>{
    map.biggest += 1;
    let mut collect:Vec<u8> = vec![0,1,0];
    collect.push(0); 
    collect.append(&mut u64_to_bytes(map.biggest));
    let mut message_as_bytes = message.as_bytes().to_vec();
    collect.push(0); 
    collect.append(&mut u64_to_bytes(message_as_bytes.len() as u64));
    collect.push(0); 
    collect.append(&mut message_as_bytes);
    collect.append(&mut vec![0,2,0]);
    return collect;
}

fn u64_to_bytes(len:u64)->Vec<u8>{
    let mut collect:Vec<u8> = Vec::new();
    match collect.write_u64::<BigEndian>(len){
        Ok(_)=>{
            return collect;
        },
        Err(_)=>{
            return collect;
        }
    }
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