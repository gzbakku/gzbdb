
#[derive(Debug)]
pub struct Error{
    pub val:String
}

#[macro_export]
macro_rules! Error{

    ( $current:expr => $chain:expr )=>{{
        Error{
            val:format!("{} => {}",$current.to_string(),$chain.val)
        }
    }};

    ($val:expr)=>{{
        Error{
            val:$val.to_string()
        }
    }};

}

impl Error{
    pub fn is(self,e:String) -> bool{
        if self.val == e{
            true
        } else {
            false
        }
    }
}
