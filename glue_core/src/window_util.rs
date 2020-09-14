use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::sync::Arc;

use super::parsers::xml::*;
use crate::GlueError;
pub use glow::*;

use crate::audio::GlueAudioContext;

pub mod event_util;

#[cfg(feature = "desktop")]
#[path = "./window_util/sdl2_window.rs"]
pub mod window;

#[cfg(feature = "web")]
#[path = "./window_util/web_window.rs"]
pub mod window;

pub use event_util::GlueEvent;
pub use window::*;

#[derive(Clone)]
pub struct GlueWindowPtr {
    ptr: Arc<RefCell<GlueWindow>>,
}

impl GlueWindowPtr {
    pub fn window(&self) -> Ref<GlueWindow> {
        self.ptr.borrow()
    }

    pub fn window_mut(&self) -> RefMut<GlueWindow> {
        self.ptr.borrow_mut()
    }

    pub fn window_cb<F>(&self, mut callback: F)
    where
        F: FnMut(&GlueWindow),
    {
        let win_ref = &*self.ptr.borrow();
        callback(win_ref);
    }

    /// Attemps to borrow window mutably. If attempt is success then `callback` is executed ,and if not possible, it simply returns false 
    pub fn window_mut_cb<F>(&self, mut callback: F) ->bool 
    where
        F: FnMut(&mut GlueWindow),
    {
        let win_ref_result  = self.ptr.try_borrow_mut();
        if let  Ok(mut ptr) = win_ref_result{
            let win_ref = &mut *ptr; 
            callback(win_ref);
            true
        }else{
            false
        }
    }
}

pub trait WindowManager: Sized {
    /// initalizes window\
    /// `config` - xml text that contains config paramaters
    /// returns an error if init fails
    fn init(config: &str) -> Result<Self, GlueError>;
    fn get_events(&mut self) -> &mut GlueEvent;
    fn clear_window(&mut self);
    fn gl(&self) -> Arc<Box<Context>>;
    fn audio_context(&self) -> Arc<RefCell<GlueAudioContext>>;
}

pub trait HasEventCollection {
    /// Populates the event queue. This is a platform specific operation
    fn collect_events(&mut self);
}

pub trait HasDimensions {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}

impl fmt::Display for GlueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::GenericError(err_str) => write!(f, "Generic Error: {}", err_str),
            Self::WindowInitError(err_str) => write!(f, "Window Init error: {}", err_str),
            Self::IOError(err_str) => write!(f, "File IO error: {}", err_str),
            _ => write!(f, "unimplemented display! [look in window_util.rs]"),
        }
    }
}

fn extract_optional_paramaters(config: &str) -> (u32, u32, String, u8, u8) {
    let mut width = 800;
    let mut height = 600;
    let mut title = String::from("glue window");
    let context_major: u8 = 3;
    let context_minor: u8 = 1;

    let parser = XMLParser::new().parse(&String::from(config)).unwrap();

    parser
        .search("width", parser.ast.root_list[0])
        .map(|node_ptr| {
            parser.get_child_tokens(node_ptr, |data, _| {
                data.map(|token| {
                    token.content.parse().map_or((), |num| {
                        width = num;
                    });
                });
                false
            });
        });

    parser
        .search("height", parser.ast.root_list[0])
        .map(|node_ptr| {
            parser.get_child_tokens(node_ptr, |data, _| {
                data.map(|token| {
                    token.content.parse().map_or((), |num| {
                        height = num;
                    });
                });
                false
            });
        });

    parser
        .search("title", parser.ast.root_list[0])
        .map(|node_ptr| {
            parser.get_child_tokens(node_ptr, |data, _| {
                data.map(|token| {
                    title = token.content.clone();
                });
                false
            });
        });

    (width, height, title, context_major, context_minor)
}
