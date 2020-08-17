use glutin;
use super::{WindowManager,GlueError};
pub struct GlueWindow{
    stuff:i32, 
    stuff2:i32, 
}
impl GlueWindow{
    pub fn new()->GlueWindow{
        GlueWindow{
            stuff:0,
            stuff2:1,
        }
    }
}

impl WindowManager for GlueWindow{
    fn init(&mut self,config:&str)->Option<GlueError>{
        println!("This is the wasm window implementations\n");
        None
    }
}