cfg_if::cfg_if! {
    if #[cfg(feature="web")] {
        pub use wasm_bindgen::prelude::*;
        pub use wasm_bindgen::*;
        pub use web_sys::{self,*};
        pub use wasm_bindgen_futures::*;
    }
}

pub use macros::*;