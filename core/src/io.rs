#[cfg(not(all(target_family = "wasm", not(target_os = "wasi"))))]
#[path = "./io/desktop_io.rs"]
pub mod io_util;

#[cfg(all(target_family = "wasm", not(target_os = "wasi")))]
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