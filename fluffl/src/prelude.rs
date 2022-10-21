cfg_if::cfg_if! {
    if #[cfg(all(target_family = "wasm", not(target_os = "wasi")))] {
        pub use wasm_bindgen::prelude::*;
        pub use wasm_bindgen::*;
        pub use web_sys::{self,*};
        pub use wasm_bindgen_futures::*;
    }else{
        pub use tokio;
    }
}

pub use glow::{self,HasContext};
pub use fluffl_macros::*;
pub use hiero_pack::{self,*};
pub use crate::console::*; 
