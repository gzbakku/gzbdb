# Keyval

this is a file based key value store, it can add delete and update key value pairs in a file, you have to ensure all transactions to the Map are atomic i suggest you use mutex to get atomic handle on file before updating the map.

## KeyVal Structure

this is a binary structre of a data frame, data frame is a key value pair in u8 binary.

```
start_flag - [0,1,0];
key_Size_int = [0,8];
key_Size = [0,0,0,0,0,0,0,0,3];
key = [0,111,101,89];
data_type_int = [0,3];
Value_Size_int = [0,8];
value_Size = [0,0,0,0,0,0,0,0,3];
value = [0,112,113,114];
end_flag = [0,2,0];

full_frame = [
    0,1,0,                  //start_flag
    0,8,                    //key_Size_int
    0,0,0,0,0,0,0,0,3,      //key_Size
    0,111,101,89,           //key
    0,3,                    //data_type_int
    0,8,                    //Value_Size_int
    0,0,0,0,0,0,0,0,3,      //value_Size
    0,112,113,114           //value
    0,2,0                   //end_flag
];
```

## Corruption Handling

data corruption is handled via frame spaces and flags cases where data can be curropted are if flags are folllowed by a [0,1] pair or if a transaction is blocked when new frame is written and old frame is not started to get removed from file space. In case of transaction halt if data was being written previous data frame will remain valid or if the new frame was written and old frame was being removed new frame will be enabled and old frame will become corropt.

## Example
```rust

//open map
let mut map1:Map;
match Map::ensure(dir,file_name){
    Ok(v)=>{map1 = v;},
    Err(e)=>{
        println!("map ensure failed => {:?}",e);
        return;
    }
}

//add
map1.add("name",Value::String("KING Akku".to_string()),false).unwrap();
//update
map1.add("name",Value::String("Emperor Akku".to_string()),false).unwrap();
//read
println!("\nname : {:?}",map1.read("name"));
//delete
map1.delete("name").unwrap();

```