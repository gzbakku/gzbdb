
// use crate::common::Error;
use crate::value::{gObjectValue,gObject};
use crate::Error;

#[derive(Debug,Clone)]
pub struct Blocks{
    pub map:Vec<Block>,
    data_size:usize
}

#[allow(dead_code)]
impl Blocks{
    pub fn len(&self) -> usize{
        self.map.len()
    }
    pub fn get(self)->Vec<gObjectValue>{
        let mut collect:Vec<gObjectValue> = Vec::new();
        for block in self.map{
            collect.push(block.get_body());
        }
        return collect;
    }
    pub fn get_documents(self)->Vec<gObjectValue>{
        let mut collect:Vec<gObjectValue> = Vec::new();
        for block in self.map{
            collect.push(block.get_document());
        }
        return collect;
    }
    pub fn get_overflow(&self) -> usize{
        let mut biggest_end:usize = 0;
        for block in &self.map{
            if block.cursor_end > biggest_end{
                biggest_end = block.cursor_end;
            }
        }
        if self.data_size == biggest_end+1{
            return self.data_size;
        } else {
            return biggest_end;
        }
    }
    pub fn get_underflow(&self) -> usize{
        let mut smallest_start:usize = self.data_size;
        for block in &self.map{
            if block.cursor_start < smallest_start{
                smallest_start = block.cursor_start;
            }
        }
        return smallest_start;
    }
}

pub fn start(pool:&Vec<u8>) -> Result<Blocks,Error>{

    match parse_group(&pool,true){
        Ok(v)=>{
            return Ok(Blocks{
                map:v,
                data_size:pool.len()
            });
        },
        Err(e)=>{
            return Err(Error!("failed-parse_group"=>e));
        }
    }
}

#[derive(Debug,Clone)]
pub struct DataPart{
    pub start:usize,
    pub end:usize
}

#[derive(Debug,Clone)]
pub struct Group{
    pub blocks:Vec<Block>,
    pub failed_parts:Vec<DataPart>
}

pub fn parse_group(pool:&Vec<u8>,find_document:bool) -> Result<Vec<Block>,Error>{


    let mut cursor = 0;
    let mut collect:Vec<Block> = Vec::new();

    // let mut index = 0;
    loop{
        let get_flag_pos_1 = read_flag_pos(&pool,&1,cursor);
        // println!("get_flag_pos_1 : {:?} {:?}",cursor,get_flag_pos_1);
        if get_flag_pos_1 == 0{
            break;
        }
        // println!("index : {:?}",index);
        
        match read_block(&pool,get_flag_pos_1-6,find_document){
            Ok(v)=>{
                match v{
                    BlockType::document(v)=>{
                        // println!("is_document");
                        cursor = v.cursor_end-2;
                        collect.push(v);
                    }
                    BlockType::cursor(v)=>{
                        // println!("is_cursor");
                        cursor = v;
                    }
                    BlockType::part(v)=>{
                        // println!("is_part");
                        // println!("{:?}",v);
                        cursor = v.cursor_end-2;
                        if !find_document{
                            collect.push(v);
                        }
                    }
                }
            }
            Err(v)=>{
                // println!("{:?}",v);
                // println!("is_error");
                cursor = v.cursor;
                // break;
            }
        }
        // if index == 10{break;}
        // index += 1;
    }

    // println!("collect : {:?}",collect);

    Ok(collect)

}

#[allow(non_camel_case_types)]
#[derive(Debug,Clone)]
pub enum BlockType{
    document(Block),
    part(Block),
    cursor(usize)
}

#[derive(Debug,Clone)]
pub struct Block{
    pub id:String,
    pub value:gObjectValue,
    pub cursor_start:usize,
    pub cursor_end:usize
}

#[allow(dead_code)]
impl Block{
    pub fn get_document(self) -> gObjectValue{
        return self.value;
    }
    pub fn get_body(self) -> gObjectValue{
        return self.value["body"].clone();
    }
    pub fn get_hash(self) -> gObjectValue{
        return self.value["hash"].clone();
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct localError{
    pub e:Error,
    pub cursor:usize
}
impl localError{
    fn new(e:Error,c:usize) -> localError{
        localError{
            e:e,
            cursor:c
        }
    }
}

#[allow(non_snake_case)]
pub fn read_block(pool:&Vec<u8>,start_cursor:usize,only_schema:bool) -> Result<BlockType,localError>{

    //**************************************|
    //  id                                  |
    //**************************************|

    //**************************************
    //get id number of bytes in number that represents length of array of bytes of the number that reprenets the id in kb as f64.

    let get_flag_pos_1 = point_flag_pos(&pool,&1,start_cursor);
    if get_flag_pos_1 == 0 {
        return Err(localError::new(
            Error!("invalid_schema"),
            start_cursor
        ));
    }
    let get_id_bytes_len = get_value(get_flag_pos_1,&pool,1);
    if get_id_bytes_len.len() == 0 || get_id_bytes_len.len() > 1 {
        return Err(localError::new(
            Error!("invalid_schema=>get_id_bytes_len"),
            get_flag_pos_1
        ));
    }
    let val_id_bytes_len = get_id_bytes_len[0] as usize;

    //**************************************
    //get the number which reprenets id length in kb

    let get_flag_pos_2 = point_flag_pos(&pool,&2,get_flag_pos_1+1);
    if get_flag_pos_1+1+7 != get_flag_pos_2 {
        return Err(localError::new(
            Error!("invalid_schema=>not_found-valid-flag_2"),
            get_flag_pos_1
        ));
    }
    let id_length_number_as_bytes = get_value(get_flag_pos_2,&pool,val_id_bytes_len);
    let id_length_number_as_bytes_len = id_length_number_as_bytes.len();
    // let cursor_id_length_number_as_bytes = get_flag_pos_2 + val_id_bytes_len;
    let id_length_in_kb = parse_f64(id_length_number_as_bytes);

    //**************************************
    //get id and parse it as a string

    let get_flag_pos_3 = point_flag_pos(&pool,&3,get_flag_pos_2+id_length_number_as_bytes_len);
    if get_flag_pos_3 == 0{
        return Err(localError::new(
            Error!("invalid_schema=>not_found-valid-flag_3"),
            get_flag_pos_2
        ));
    }
    let get_id_bytes = get_data(get_flag_pos_3,&pool,id_length_in_kb);
    let id_bytes_length = get_id_bytes.len();
    let id:String;
    match String::from_utf8(get_id_bytes){
        Ok(v)=>{id = v;},
        Err(_)=>{
            return Err(localError::new(
                Error!("failed-parse_id_to_string"),
                get_flag_pos_3
            ));
        }
    }

    if only_schema{
        if id != "gObject Document Schema : v1"{
            return Ok(BlockType::cursor(get_flag_pos_3));
        }
    }

    //**************************************|
    //  data                                  |
    //**************************************|

    //**************************************
    //get data type

    let get_flag_pos_4 = point_flag_pos(&pool,&4,get_flag_pos_3+id_bytes_length);
    if get_flag_pos_4 == 0 || get_flag_pos_4 != (get_flag_pos_3 + id_bytes_length + 1 + 6){
        return Err(localError::new(
            Error!("invalid_schema=>not_found-valid-flag_4"),
            get_flag_pos_3
        ));
    }
    let get_data_type_vector = get_value(get_flag_pos_4,&pool,1);
    if get_data_type_vector.len() == 0 || get_data_type_vector[0] == 0{
        return Err(localError::new(
            Error!("invalid_schema=>invalid-get_data_type_vector"),
            get_flag_pos_4
        ));
    }
    let data_type_int = get_data_type_vector[0];

    //**************************************
    //data data length number as a vector length in f64

    let get_flag_pos_5 = point_flag_pos(&pool,&5,get_flag_pos_4+1);
    if get_flag_pos_5 == 0 || get_flag_pos_5 != (get_flag_pos_4 + 1 + 1 + 6){
        return Err(localError::new(
            Error!("invalid_schema=>not_found-valid-flag_5"),
            get_flag_pos_4
        ));
    }
    let get_data_len_num_vector = get_value(get_flag_pos_5,&pool,1);
    if get_data_len_num_vector.len() == 0 || get_data_len_num_vector[0] == 0{
        return Err(localError::new(
            Error!("invalid_schema=>invalid-get_data_len_num_vector"),
            get_flag_pos_5
        ));
    }
    let data_len_num_bytes_len = get_data_len_num_vector[0] as usize;

    // println!("data_len_num_bytes_len : {:?}",data_len_num_bytes_len);

    //**************************************
    //get data len bytes

    let get_flag_pos_6 = point_flag_pos(&pool,&6,get_flag_pos_5+1);
    if get_flag_pos_6 == 0 || get_flag_pos_6 != (get_flag_pos_5 + 1 + 1 + 6){
        return Err(localError::new(
            Error!("invalid_schema=>not_found-valid-flag_6"),
            get_flag_pos_5
        ));
    }
    let get_data_len_bytes_array = get_value(get_flag_pos_6,&pool,data_len_num_bytes_len);
    let data_len_bytes_len = get_data_len_bytes_array.len();
    let data_len_num = parse_f64(get_data_len_bytes_array);

    //**************************************
    //get data bytes vector

    let get_flag_pos_7 = point_flag_pos(&pool,&7,get_flag_pos_6+data_len_bytes_len);
    if get_flag_pos_7 == 0 || get_flag_pos_7 != (get_flag_pos_6 + data_len_bytes_len + 1 + 6){
        return Err(localError::new(
            Error!("invalid_schema=>not_found-valid-flag_7"),
            get_flag_pos_6
        ));
    }
    let data = get_data(get_flag_pos_7,&pool,data_len_num);
    let data_len = data.len();

    //**************************************
    //get data bytes vector

    let get_flag_pos_8 = point_flag_pos(&pool,&8,get_flag_pos_7+data_len);
    if get_flag_pos_8 == 0 || get_flag_pos_8 != (get_flag_pos_7 + data_len + 1 + 6){
        return Err(localError::new(
            Error!("invalid_schema=>not_found-valid-flag_8"),
            get_flag_pos_7
        ));
    }

    let val:gObjectValue;
    if 
        data_type_int == 21 ||  //string
        data_type_int == 2      //header
    {
        match String::from_utf8(data){
            Ok(v)=>{val = gObjectValue::string(v);},
            Err(_)=>{
                return Err(localError::new(
                    Error!("failed-parse_string"),
                    get_flag_pos_8
                ));
            }
        }
    } else if 
        data_type_int == 1 ||   //document
        data_type_int == 3 ||   //body
        data_type_int == 22     //object
    {
        let mut make_gObject = gObject::new();
        match parse_group(&data,false){
            Ok(blocks)=>{
                for block in blocks{
                    make_gObject.insert(&block.id,block.value);
                }
            },
            Err(_)=>{
                return Err(localError::new(
                    Error!("failed-parse_object-group"),
                    get_flag_pos_8
                ));
            }
        }
        val = gObjectValue::object(make_gObject);
    } else if data_type_int == 23 { //array
        let mut make_gObject_array:Vec<gObjectValue> = Vec::new();
        match parse_group(&data,false){
            Ok(blocks)=>{
                println!("\nblocks : {:?}\n",blocks);
                for block in blocks{
                    make_gObject_array.push(block.value);
                }
            },
            Err(_)=>{
                return Err(localError::new(
                    Error!("failed-parse_object-group"),
                    get_flag_pos_8
                ));
            }
        }
        val = gObjectValue::array(make_gObject_array);
    } 
    else if data_type_int == 4 {val = gObjectValue::null;}//null
    else if data_type_int == 5 {val = gObjectValue::undefined;}//undefined
    else if data_type_int == 24 {val = gObjectValue::binary(data);}//binary
    else if data_type_int == 25 {//bool
        if data.len() != 1{val = gObjectValue::null;} else
        if data[0] == 0{val = gObjectValue::bool(false);} else
        if data[0] == 1{val = gObjectValue::bool(true);} 
        else {val = gObjectValue::null;}
    }
    else if data_type_int == 31{val = parse_number(31,data);}//i32
    else if data_type_int == 32{val = parse_number(32,data);}//i64
    else if data_type_int == 33{val = parse_number(33,data);}//i128
    else if data_type_int == 34{val = parse_number(34,data);}//u32
    else if data_type_int == 35{val = parse_number(35,data);}//u64
    else if data_type_int == 36{val = parse_number(36,data);}//u128
    else if data_type_int == 37{val = parse_number(37,data);}//f32
    else if data_type_int == 38{val = parse_number(38,data);}//f64
    else {val = gObjectValue::null;}//undefined value

    let b = Block{
        id:id.clone(),
        value:val,
        cursor_start:start_cursor,
        cursor_end:get_flag_pos_8-1
    };

    if id == "gObject Document Schema : v1"{
        return Ok(BlockType::document(b));
    } else {
        return Ok(BlockType::part(b));
    }

}

fn get_data(cursor_g:usize,pool:&Vec<u8>,len:f64) -> Vec<u8>{

    let mut collect:Vec<u8> = Vec::new();

    let mut cursor = cursor_g;
    let mut local = len;
    let data_len = pool.len();

    while local > 10000.0{
        for i in cursor..cursor+(10000*1000){
            if i < data_len{
                collect.push(pool[i].into());
            }
        }
        local -= 10000.0;
        cursor += 10000;
    }

    let last = (local * 1000.0) as usize;

    for i in cursor..cursor+last{
        if i < data_len{
            collect.push(pool[i].into());
        }
    }

    collect

}

fn get_value(cursor:usize,pool:&Vec<u8>,len:usize) -> Vec<u8>{
    let mut collect:Vec<u8> = Vec::new();
    let data_len = pool.len();
    if pool.len() < cursor+len {return collect;}
    for i in cursor..cursor+len{
        if i < data_len{
            collect.push(pool[i]);
        }
    }
    return collect;
}

fn point_flag_pos(data:&Vec<u8>,flag:&u8,cursor:usize) -> usize{
    if data.len() < cursor + 6{return 0;}
    let mut collect:Vec<u8> = Vec::new();
    let data_len = data.len();
    for i in cursor..cursor+7{
        // println!("i : {:?}",i);
        if i < data_len{
            collect.push(data[i]);
        }
    }
    // println!("flag : {:?} {:?}",flag,collect);
    if
        collect.len() != 7 ||
        collect[0] != 0 || collect[1] != 0 || collect[2] != 0 ||
        &collect[3] != flag ||
        collect[4] != 0 || collect[5] != 0 || collect[6] != 0
    {
        return 0;
    } else {
        return cursor+6+1;
    }
}

fn read_flag_pos(data:&Vec<u8>,flag:&u8,start_cursor:usize) -> usize{

    let mut before_flag_count = 0;
    let mut after_flag_count = 0;
    let mut flag_found = false;
    let data_len = data.len();
    for n in start_cursor..data.len()+1{
        if n < data_len{//check if len of data vector is more then n
            let i = data[n];//value from array
            if i == 0{//process zero value
                if flag_found{//process zeros after flag
                    after_flag_count += 1;
                    if after_flag_count == 3{
                        return n;
                    }
                } else {//process zeros before flag
                    before_flag_count += 1;
                    if before_flag_count == 4{
                        before_flag_count = 1;
                        after_flag_count = 0;
                        flag_found = false;
                    }
                }
            } else {//process data value
                if before_flag_count == 3 && after_flag_count == 0 && &i == flag{
                    flag_found = true;
                } else {
                    before_flag_count = 0;
                    after_flag_count = 0;
                    flag_found = false;
                }
            }//process data value
        }//check if len of data vector is more then n
    }//loop data
    return 0;
}//fn read_flag_pos

use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

fn parse_number(int_type:u8,pool:Vec<u8>) -> gObjectValue{

    let null_val = gObjectValue::null;
    let mut data =  Cursor::new(pool);

    if int_type == 31{
        match data.read_i32::<BigEndian>(){
            Ok(v)=>{return gObjectValue::i32(v);},
            Err(_)=>{return null_val;}
        }
    } else if int_type == 32{
        match data.read_i64::<BigEndian>(){
            Ok(v)=>{return gObjectValue::i64(v);},
            Err(_)=>{return null_val;}
        }
    } else if int_type == 33{
        match data.read_i128::<BigEndian>(){
            Ok(v)=>{return gObjectValue::i128(v);},
            Err(_)=>{return null_val;}
        }
    } 
    
    else if int_type == 34{
        match data.read_u32::<BigEndian>(){
            Ok(v)=>{return gObjectValue::u32(v);},
            Err(_)=>{return null_val;}
        }
    } else if int_type == 35{
        match data.read_u64::<BigEndian>(){
            Ok(v)=>{return gObjectValue::u64(v);},
            Err(_)=>{return null_val;}
        }
    } else if int_type == 36{
        match data.read_u128::<BigEndian>(){
            Ok(v)=>{return gObjectValue::u128(v);},
            Err(_)=>{return null_val;}
        }
    }

    else if int_type == 37{
        match data.read_f32::<BigEndian>(){
            Ok(v)=>{return gObjectValue::f32(v);},
            Err(_)=>{return null_val;}
        }
    } else if int_type == 38{
        match data.read_f64::<BigEndian>(){
            Ok(v)=>{return gObjectValue::f64(v);},
            Err(_)=>{return null_val;}
        }
    } 

    return null_val;

}

fn parse_f64(v:Vec<u8>) -> f64{
    let mut hold:[u8;8] = [0;8];
    for i in 0..v.len(){
        hold[i] = v[i];
    }
    f64::from_be_bytes(hold)
}
