use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell; 


#[cfg(feature = "desktop")]
#[path = "./io/desktop_io.rs"]
pub mod io_util;

#[cfg(feature = "web")]
#[path = "./io/web_io.rs"]
pub mod io_util;

use crate::GlueError;
pub use io_util::*;


#[cfg(feature = "web")]
#[macro_export]
macro_rules! load_file {
    ( $argument:expr ) => {
        load_file($argument).await
    };
    ( $argument:ident ) => {
        load_file($argument).await
    };
}

#[cfg(feature = "desktop")]
#[macro_export]
macro_rules! load_file {
    ( $argument:expr ) => {
        load_file($argument)
    };
    ( $argument:ident ) => {
        load_file($argument)
    };
}


static mut BINARY_RESOURCES: Option<HashMap<String, Vec<u8>>> = None;

pub fn register_resource<'a>(resource_name: String, resource: Vec<u8>) {
    get_resource_table_mut().map(|htable| htable.insert(resource_name, resource));
}

pub fn get_resource<'a>(resource_name: String) -> Option<&'a Vec<u8>> {
    get_resource_table_mut()
        .map(|htable| htable.get(&resource_name))
        .flatten()
}

pub fn get_resource_mut<'a>(resource_name: String) -> Option<&'a mut Vec<u8>> {
    get_resource_table_mut()
        .map(|htable| htable.get_mut(&resource_name))
        .flatten()
}

fn get_resource_table_mut<'a>() -> Option<&'a mut HashMap<String, Vec<u8>>> {
    unsafe { BINARY_RESOURCES.as_mut().map(|a| a) }
}
