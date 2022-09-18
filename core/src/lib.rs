// / Module common crates that you'll probably need access to
/// Module for playing sounds
pub mod audio;
/// Module for writing text to consoles
pub mod console;
/// Module for loading files
pub mod io;
/// Module for opening websocket clients
pub mod net;
pub mod prelude;
/// Module for timing functions
pub mod time;
/// Module for creating a an opengl window
pub mod window;

/// private custom datastructures
pub mod collections;
/// private decodes
pub mod decoders;
/// private custom parsers
mod parsers;
mod slice;

/// unsafe memory stuff
mod mem;

pub mod gui;

/// private custom iterators
mod iterators;

/// math utilities
pub mod math;

/// Extras module has music playback and text-rendering routines
/// This module is totally optional, and not really considered a part of the library
#[cfg(feature = "extras")]
pub mod extras;

use glow::Context;
use std::{cell::RefCell, ops::Deref, rc::Rc, sync::Arc};

/// A pointer to GLOW state. All variables with this type should be named: `gl`
pub type GlowGL = Arc<Box<Context>>;


pub struct FlufflState<T> {
    inner: Rc<RefCell<T>>,
}
impl<T> Clone for FlufflState<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<T> FlufflState<T> {
    pub fn new(state: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(state)),
        }
    }
}

impl<T> Deref for FlufflState<T> {
    type Target = Rc<RefCell<T>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

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
