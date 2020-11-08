// / Module common crates that you'll probably need access to
pub mod prelude; 

/// Module for playing sounds
pub mod audio;
/// Module for writing text to consoles 
pub mod console;
/// Module for loading files 
pub mod io;
/// Module for opening websocket clients
pub mod net;
/// Module for creating a an opengl window
pub mod window;


/// private custom datastructures
mod collections;
/// private custom parsers
mod parsers;

/// Extras module has music playback and text-rendering routines
/// This module is totally optional, and not really considered a part of the library
#[cfg(feature = "extras")]
pub mod extras;

use glow::Context;
use std::sync::Arc;

/// A pointer to GLOW state. All variables with this type should be named: `gl`
pub type GlState = Arc<Box<Context>>;

#[derive(Debug)]
/// A collection of Common errors that possibly could arise
pub enum FlufflError {
    GenericError(String),
    FromUtf8ParseError(String),
    WindowInitError(String),
    IOError(String),
}

impl From<std::io::Error> for FlufflError {
    fn from(err: std::io::Error) -> Self {
        FlufflError::IOError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for FlufflError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::FromUtf8ParseError(err.to_string())
    }
}
