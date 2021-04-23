
#[derive(Debug)]
pub struct Error{
    pub val:String
}

#[macro_export]
macro_rules! Error{
    ($val:expr)=>{{
        Error{
            val:$val.to_string()
        }
    }};

    ( $current:expr => $chain:expr )=>{{
        Error{
            val:format!("{} => {}",$current.to_string(),$chain.val)
        }
    }};

}
