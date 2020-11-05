use super::*;
use js_sys;
use wasm_bindgen;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;

pub use wasm_bindgen_futures::*;

pub async fn load_file(file_path: &str) -> Result<Vec<u8>, FlufflError> {
    let path = String::from(file_path);
    load_file_helper(path.as_str()).await
}

pub fn load_file_cb<F>(file_path: &str,mut cb:F)
where F : FnMut( Result<Vec<u8>,FlufflError>) + 'static 
{
    let path = String::from(file_path);
    spawn_local(async move {
        let result = load_file_helper(path.as_str()).await;
        cb(result)
    });
}

async fn load_file_helper(file_path: &str) -> Result<Vec<u8>, FlufflError> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    let request = Request::new_with_str_and_init(file_path, &opts)?;

    request
        .headers()
        .set("Accept", "application/octet-stream")?;

    let window = web_sys::window().unwrap();
    let resp_val = JsFuture::from(window.fetch_with_request(&request)).await?;

    assert!(resp_val.is_instance_of::<Response>());
    let resp: Response = resp_val.dyn_into().unwrap();

    if resp.status() != 200 {
        return Err(FlufflError::IOError(String::from(
            "Error: File not found!",  
        )));
    }
    
    let blob: Blob = JsFuture::from(resp.blob().unwrap())
        .await?
        .dyn_into()
        .unwrap();

    let array_buffer = JsFuture::from(blob.array_buffer()).await?;
    let byte_buffer = js_sys::Uint8Array::new(&array_buffer).to_vec();
    Ok(byte_buffer)
}

impl From<JsValue> for FlufflError {
    fn from(_js_error: JsValue) -> Self {
        FlufflError::IOError(String::new())
    }
}
