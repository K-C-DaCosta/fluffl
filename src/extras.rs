/// This module basically a bunch of helper code to make dealing with opengl easier.
/// It was not inteneded to be used by others.
pub mod ogl; 

/// This module has a bunch of code to play ogg,mp3 and wav formats.
/// It was not inteneded to be used by others and my still have bugs.
pub mod audio; 

// A very simple packing/upacking library used to load SDF fonts.
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