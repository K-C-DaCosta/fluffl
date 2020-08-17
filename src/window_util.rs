use std::fmt;
pub mod event_util;

#[cfg(feature = "desktop-backend")]
#[path = "./window_util/sdl2_window.rs"]
mod window;


#[cfg(feature = "browser-backend")]
#[path = "./window_util/web_window.rs"]
mod window;

pub use window::GlueWindow;


pub use event_util::GlueEvent; 
#[derive(Debug)]
pub enum GlueError {
    GenericError(String),
    WindowInitError(String),
}


pub trait WindowManager : Sized  {
    /// initalizes window\
    /// `config` - xml text that contains config paramaters
    /// returns an error if init fails
    fn init(config: &str) -> Result<Self,GlueError>;
    fn get_events(&mut self)->&mut GlueEvent;
    fn clear_window(&mut self);
    /// Populates the event queue. This is a platform specific operation
    fn collect_events(&mut self);
}

impl fmt::Display for GlueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::GenericError(err_str) => write!(f, "Generic Error: {}", err_str),
            Self::WindowInitError(err_str) => write!(f, "Window Init error: {}", err_str),
        }
    }
}
