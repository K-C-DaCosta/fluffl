/// This module has a bunch of code to play ogg,mp3 and wav formats.
/// It was not inteneded to be used by others and my still have bugs.
pub mod audio;

/// Lets you draw anti-aliased rectangles, circles, etc
pub mod shapes;

// A very simple packing/upacking library used to load SDF fonts.
pub use hiero_pack;

pub mod math_util;

#[derive(Debug)]
pub enum ErrorKind {
    WavParseError(String),
    Mp3ParseError(String),
    OggParseError(String),
}

pub type Result<T> = std::result::Result<T, ErrorKind>;
