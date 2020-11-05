pub mod ogl; 
pub mod audio; 
pub use hiero_pack;

/// An auxiliary module that provides some basic utilities for music and text. 
/// This module is completely optional an I encourage users to come up with their 
/// own utilities.

#[derive(Debug)]
pub enum ErrorKind{
    WavParseError(String),
    Mp3ParseError(String),
    OggParseError(String),
}

pub type Result<T> = std::result::Result<T,ErrorKind>;