
/// Open a Map File
/// map file will be created if it doesnt exists or else it will open the map file
/// ```rust
/// let dir = "D://workstation/expo/rust/gzbdb/test/keyVal".to_string();
/// let file_name = format!("test1");
/// let mut map1:Map;
/// match Map::ensure(dir,file_name){
///     Ok(v)=>{map1 = v;},
///     Err(e)=>{
///         println!("map ensure failed => {:?}",e);
///         return;
///     }
/// }
/// 
/// //add keyval
/// map1.add("name",Value::String("KING Akku".to_string()),false).unwrap();
/// //update keyval
/// map1.add("name",Value::String("Emperor Akku".to_string()),false).unwrap();
/// //read keyval
/// println!("\nname : {:?}",map1.read("name"));
/// //delete keyval
/// map1.delete("name").unwrap();
/// ```


mod myio;
mod map;
mod parse;
mod mapify;
mod common;

pub use common::Error;
pub use map::{Map,Value};
pub use mapify::{
    DataFrame,
    DataFrameType,
    MapStructure,
    MapStructureStatus,
    Frame,
    Pointer
};
