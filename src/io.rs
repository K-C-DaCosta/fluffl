#[cfg(feature = "desktop")]
#[path = "./io/desktop_io.rs"]
pub mod io_util;

#[cfg(feature = "web")]
#[path = "./io/web_io.rs"]
pub mod io_util;

use crate::FlufflError;
pub use io_util::*;

#[macro_export]
macro_rules! load_file {
    ( $argument:expr ) => {
        load_file($argument).await
    };
    ( $argument:ident ) => {
        load_file($argument).await
    };
}