

use crate::gObject;
use crate::parser::{read_block,BlockType};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct gObjectReader{
    pub collection:Vec<u8>,
    pub started:bool,
    pub docs:Vec<gObject>,
    pub last_flag:u8,
    id_len_bytes:usize,
    id_len:usize,
    data_len_bytes:usize,
    data_len:usize
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum gObjectReaderStatus{
    Doc,
    Flush,
    Wait
}

impl gObjectReader{
    pub fn new()->gObjectReader{
        gObjectReader{
            collection:Vec::new(),
            started:false,
            last_flag:0,
            docs:Vec::new(),
            id_len_bytes:0,
            id_len:0,
            data_len_bytes:0,
            data_len:0
        }
    }
    pub fn with_capacity(c:usize)->gObjectReader{
        gObjectReader{
            collection:Vec::with_capacity(c),
            started:false,
            last_flag:0,
            docs:Vec::new(),
            id_len_bytes:0,
            id_len:0,
            data_len_bytes:0,
            data_len:0
        }
    }
    pub fn reset(&mut self){
        self.started = false;
        self.last_flag = 0;
        self.id_len_bytes = 0;
        self.id_len = 0;
        self.data_len_bytes = 0;
        self.data_len = 0;
    }
    pub fn pop(&mut self)->Option<gObject>{
        match self.docs.pop(){
            Some(v)=>{
                return Some(v);
            },
            None=>{
                None
            }
        }
    }
    pub fn pop_all(&mut self)->Vec<gObject>{
        let hold = self.docs.clone();
        self.docs = Vec::new();
        return hold;
    }
    pub fn push(&mut self,data:&Vec<u8>) -> gObjectReaderStatus{

        self.collection.extend_from_slice(data);

        if !self.started{
            match read_flag_pos(&self.collection,&1,0){
                Ok(cursor)=>{
                    if cursor > 6{
                        self.collection = self.collection.split_off(cursor-6);
                    }
                    self.started = true;
                    self.last_flag = 1;
                },
                Err(_)=>{
                    // self.collection = Vec::new();
                    return gObjectReaderStatus::Wait;
                }
            }
        }

        if self.last_flag == 1{
            if self.collection.len() >= 7+1+7{
                match find_flag_at(&self.collection,2,7+1){
                    Ok(_)=>{
                        self.last_flag = 2;
                        self.id_len_bytes = self.collection[7].clone() as usize;
                        // println!("flag 2 found");
                    },
                    Err(_)=>{
                        self.collection = Vec::new();
                        return gObjectReaderStatus::Flush;
                    }
                }
            }
        }
        if self.last_flag == 2{
            if self.collection.len() >= 7+1+7+self.id_len_bytes+7{
                match find_flag_at(&self.collection,3,7+1+7+self.id_len_bytes){
                    Ok(_)=>{
                        let id_len_bytes = get_data(&self.collection,self.id_len_bytes,7+1+7);
                        let id_len_kb_f64 = parse_f64(id_len_bytes);
                        let id_len_in_no_of_bytes = (id_len_kb_f64 * 1000.0) as usize;
                        self.id_len = id_len_in_no_of_bytes;
                        self.last_flag = 3;
                        // println!("flag 3 found");
                    },
                    Err(_)=>{
                        self.collection = Vec::new();
                        return gObjectReaderStatus::Flush;
                    }
                }
            }
        }
        if self.last_flag == 3{
            if self.collection.len() >= 7+1+7+self.id_len_bytes+7+self.id_len+7{
                match find_flag_at(&self.collection,4,7+1+7+self.id_len_bytes+7+self.id_len){
                    Ok(_)=>{
                        self.last_flag = 4;
                        // println!("flag 4 found");
                    },
                    Err(_)=>{
                        self.collection = Vec::new();
                        return gObjectReaderStatus::Flush;
                    }
                }
            }
        }
        if self.last_flag == 4{
            if self.collection.len() >= 7+1+7+self.id_len_bytes+7+self.id_len+7+1+7{
                match find_flag_at(&self.collection,5,7+1+7+self.id_len_bytes+7+self.id_len+7+1){
                    Ok(_)=>{
                        self.last_flag = 5;
                        // println!("flag 5 found");
                    },
                    Err(_)=>{
                        self.collection = Vec::new();
                        return gObjectReaderStatus::Flush;
                    }
                }
            }
        }
        if self.last_flag == 5{
            let limit = (7+1)+(7+self.id_len_bytes)+(7+self.id_len)+(7+1)+(7+1)+7;
            if self.collection.len() >= limit{
                match find_flag_at(&self.collection,6,limit-7){
                    Ok(_)=>{
                        self.last_flag = 6;
                        self.data_len_bytes = self.collection[limit-7-1] as usize;
                        // println!("flag 6 found : {:?}",self.data_len_bytes);
                    },
                    Err(_)=>{
                        self.collection = Vec::new();
                        return gObjectReaderStatus::Flush;
                    }
                }
            }
        }
        if self.last_flag == 6{
            let limit = (7+1)+(7+self.id_len_bytes)+(7+self.id_len)+(7+1)+(7+1)+(7+self.data_len_bytes)+7;
            if self.collection.len() >= limit{
                match find_flag_at(&self.collection,7,limit-7){
                    Ok(_)=>{
                        let data_len_bytes = get_data(&self.collection,self.data_len_bytes,limit-self.data_len_bytes-7);
                        let data_len_kb_f64 = parse_f64(data_len_bytes);
                        let data_len_in_no_of_bytes = (data_len_kb_f64 * 1000.0) as usize;
                        self.last_flag = 7;
                        self.data_len = data_len_in_no_of_bytes as usize;
                        // println!("flag 7 found");
                    },
                    Err(_)=>{
                        self.collection = Vec::new();
                        return gObjectReaderStatus::Flush;
                    }
                }
            }
        }
        if self.last_flag == 7{
            let limit = (7+1)+(7+self.id_len_bytes)+(7+self.id_len)+
                        (7+1)+(7+1)+(7+self.data_len_bytes)+
                        (7+self.data_len)+7;
            if self.collection.len() >= limit{
                match find_flag_at(&self.collection,8,limit-7){
                    Ok(_)=>{
                        self.last_flag = 8;
                        let (first,second) = self.collection.split_at(limit);
                        match read_block(&first.to_vec(),0,true){
                            Ok(parsed)=>{
                                match parsed{
                                    BlockType::document(value)=>{
                                        self.docs.push(value.value.unchecked_object());
                                        self.collection = second.to_vec();
                                        self.reset();
                                        loop{
                                            match self.push(&vec![]){
                                                gObjectReaderStatus::Doc=>{},
                                                _=>{break;}
                                            }
                                        }
                                        return gObjectReaderStatus::Doc;
                                    },
                                    _=>{}
                                }
                            },
                            Err(_)=>{}
                        }
                        // println!("flag 8 found");
                    },
                    Err(_)=>{
                        self.collection = Vec::new();
                        return gObjectReaderStatus::Flush;
                    }
                }
            }
        }

        return gObjectReaderStatus::Wait;

    }
}

fn get_data(data:&Vec<u8>,len:usize,cursor:usize)->Vec<u8>{
    // println!("data_len : {:?} read_len : {:?} c : {:?}",data.len(),len,cursor);
    let mut collect = Vec::with_capacity(len);
    for i in cursor..cursor+len{
        collect.push(data[i]);
    }
    collect
}

fn find_flag_at(data:&Vec<u8>,flag:u8,cursor:usize) -> Result<(),()>{
    if data.len() < cursor+6{
        return Err(());
    }
    if 
    data[cursor] == 0 && 
    data[cursor+1] == 0 && 
    data[cursor+2] == 0 && 

    data[cursor+3] == flag && 

    data[cursor+4] == 0 && 
    data[cursor+5] == 0 && 
    data[cursor+6] == 0{
        return Ok(());
    } else {
        println!("s : {:?} f : {:?} e : {:?}",data[cursor],data[cursor+3],data[cursor+6]);
        return Err(());
    }
}

fn read_flag_pos(data:&Vec<u8>,flag:&u8,start_cursor:usize) -> Result<usize,()>{
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
                        return Ok(n);
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
    return Err(());
}//fn read_flag_pos

fn parse_f64(v:Vec<u8>) -> f64{
    let mut hold:[u8;8] = [0;8];
    for i in 0..v.len(){
        hold[i] = v[i];
    }
    f64::from_be_bytes(hold)
}
