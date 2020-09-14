use super::*;
use std::fs::File;
use std::io::prelude::*;


pub fn load_file(file_path: &str) -> Result<Vec<u8>, GlueError>
{
    load_file_helper(file_path)
}

pub fn load_file_cb<F>(file_path: &str,mut cb:F)
where F : FnMut( Result<Vec<u8>,GlueError>)
{
    let result = load_file_helper(file_path);
    cb(result)
}

fn load_file_helper(file_path: &str) -> Result<Vec<u8>, GlueError> {
    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => {
            return Err(GlueError::IOError(String::from(
                "failed to open file [read mode]",
            )))
        }
    };

    let mut byte_buffer = Vec::new();
    match file.read_to_end(&mut byte_buffer) {
        Err(_) => return Err(GlueError::IOError(String::from("failed to read file"))),
        _ => (),
    }

    Ok(byte_buffer)
}
