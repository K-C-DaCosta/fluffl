use super::*;
use std::fs::File;
use std::io::prelude::*;
/// # Description
/// Fetches a file in its entirety \
/// # Arguments
/// `file_path` - the location of the file. Example: "./foo.txt" \
/// # Comments
/// This function is only good for reading tiny files. \
/// On web targets, this just does a **GET** request using fetch API.  
/// On deksop targets, this is just a `std::fs::read(...)` or someting (could change)
/// In the future I could use http *HEAD* combined with *PARTIAL CONTENT*
/// in order to read parts of files on the HTTP side of things.  
pub async fn load_file(file_path: &str) -> Result<Vec<u8>, FlufflError> {
    load_file_helper(file_path)
}

/// # Description
/// fetches entire file, but instead user has to read contents through a callback
/// this is done to avoid blocking if one doesn't need it
/// this function only really does non-blocking reads in wasm target AFAICT
pub fn load_file_cb<F>(file_path: &str, mut cb: F)
where
    F: FnMut(Result<Vec<u8>, FlufflError>),
{
    let result = load_file_helper(file_path);
    cb(result)
}

fn load_file_helper(file_path: &str) -> Result<Vec<u8>, FlufflError> {
    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => {
            return Err(FlufflError::IOError(format!(
                "failed to open {}",
                file_path,
            )))
        }
    };

    let mut byte_buffer = Vec::new();
    if file.read_to_end(&mut byte_buffer).is_err() {
        return Err(FlufflError::IOError(format!(
            "failed to read {}",
            file_path
        )));
    }
    Ok(byte_buffer)
}
