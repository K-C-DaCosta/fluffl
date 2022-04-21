#[cfg(not(all(target_family = "wasm", not(target_os = "wasi"))))]
#[path = "./time/desktop_time.rs"]
pub mod time_util;

#[cfg(all(target_family = "wasm", not(target_os = "wasi")))]
#[path = "./time/web_time.rs"]
pub mod time_util;


pub use time_util::*;