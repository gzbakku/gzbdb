
#[derive(Debug)]
pub struct Config{
    pub cwd:String
}

impl Config{
    pub fn new()->Config{
        Config{
            cwd:crate::myio::cwd()
        }
    }
}