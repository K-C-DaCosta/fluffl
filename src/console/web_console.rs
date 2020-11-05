use wasm_bindgen;
use wasm_bindgen::prelude::*; 

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[allow(unused_unsafe)]
pub fn console_write(text: &str) {
    unsafe { log(text) }
}
