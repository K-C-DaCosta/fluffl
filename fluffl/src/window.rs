use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    fmt,
    sync::Arc,
};

mod window_backends;

use super::parsers::xml::*;
use crate::{audio::FlufflAudioContext, FlufflError, GlowGL};

pub use event_util::FlufflEvent;
pub use window_backends::*;
pub mod event_util;
pub mod touch_tracker;

use touch_tracker::*;

#[derive(Clone, Copy)]
pub struct FlufflRunning {
    val: *mut bool,
}
impl FlufflRunning {
    pub fn new(running: &mut bool) -> Self {
        Self {
            val: running as *mut bool,
        }
    }

    pub fn get(&self) -> bool {
        unsafe { *self.val }
    }

    pub fn set(&mut self, val: bool) {
        unsafe {
            *self.val = val;
        }
    }
}

#[derive(Clone)]
pub struct FlufflWindowPtr {
    ptr: Arc<RefCell<FlufflWindow>>,
}

impl FlufflWindowPtr {
    /// Borrow window
    pub fn window(&self) -> Ref<FlufflWindow> {
        self.ptr.borrow()
    }

    /// Borrow the window mutably
    pub fn window_mut(&self) -> RefMut<FlufflWindow> {
        self.ptr.borrow_mut()
    }

    /// borrow the window through a callback
    pub fn window_cb<F>(&self, mut callback: F)
    where
        F: FnMut(&FlufflWindow),
    {
        let win_ref = &*self.ptr.borrow();
        callback(win_ref);
    }

    /// Attemps to borrow window mutably. If attempt is success then `callback` is executed ,and if not possible, it simply returns false
    pub fn window_mut_cb<F>(&self, mut callback: F) -> bool
    where
        F: FnMut(&mut FlufflWindow),
    {
        let win_ref_result = self.ptr.try_borrow_mut();
        let can_borrow = win_ref_result.is_ok();
        if let Ok(mut ptr) = win_ref_result {
            let win_ref = &mut *ptr;
            callback(win_ref);
        }
        can_borrow
    }
}

pub trait HasFlufflWindow: Sized {
    /// # Description
    /// initalizes window to `config`'s specifications
    /// # Parameters
    /// `config` - xml text that contains config paramaters
    /// # Returns
    /// returns an error if init fails
    /// # Comments
    /// `config` is of the format:
    /// ```xml
    /// <window>
    ///     <width>800</width>
    ///     <height>600</height>
    ///     <fullscreen>false</fullscreen>
    ///     ...
    ///     <canvas_id>fluffl</canvas_id>
    /// </window>
    /// ```
    /// Tags include:
    /// - `width`/`height`
    ///     - description:
    ///         - The desired window dimesnsions
    ///         - by default its assumed to be `800x600`
    ///     - value type: `u32`
    /// - `fullscreen`
    ///     - description:
    ///         - The desired windowing mode fluffl window
    ///         - By default this is assumed to be `false`
    ///     - value type: `bool`
    ///     - valid values are:
    ///         - `true`
    ///         - `false`
    /// - `context_major`/`context_minor`  
    ///     - description:
    ///         - The desired opengl version for desktop build
    ///         - By default we use opengl major=3 minor=0
    ///     - value type: `u32`
    ///     - possible values are:
    ///         - `0`,`2`,...,`100`,...
    /// - `wgl_version`
    ///     - description:
    ///         - The desired webgl version for browser build
    ///         - By default `webgl2` is assumed
    ///     - value type: `String`
    ///     - valid values are:
    ///         - `webgl1`
    ///         - `webgl2`
    /// - `resizeable`
    ///     - description:
    ///         - configures window to be resizable if `true` else the window stays fixed
    ///         - by default this setting is assumed to be false
    ///     - value type: `bool`
    fn init(config: &str) -> Result<Self, FlufflError>;
    /// returns the window event queue
    fn get_events(&mut self) -> &mut FlufflEvent;

    /// # Description
    /// Exposes the glow api to user
    /// # Comments
    /// - make sure you `use fluffl::{window::{ ... , glow::*, ... }};` in order to actually get access to the interface functions
    fn gl(&self) -> GlowGL;

    /// Returns a hook to audio functions
    fn audio_context(&self) -> FlufflAudioContext;

    /// returns current width of window
    fn width(&self) -> u32;

    /// returns current height of window
    fn height(&self) -> u32;

    /// # Description
    /// Used to enter/exit fullscreen mode
    /// # Parameters
    /// - `go_fullscreen`
    ///     - if set to `true` the window will enter fullscreen mode
    ///     - if set to `false` the window will exit fullscreen mode
    /// # Comments
    /// - If the window is already in the desired state the function will do nothing
    fn set_fullscreen(&mut self, go_fullscreen: bool);

    fn get_bounds(&self) -> (u32, u32) {
        (self.width(), self.height())
    }
    fn get_bounds_f32(&self) -> (f32, f32) {
        (self.width() as f32, self.height() as f32)
    }
}
/// This should NOT be PUBLIC
trait HasEventCollection {
    /// Populates the event queue. This is a platform specific operation
    fn collect_events(&mut self);
}

impl fmt::Display for FlufflError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::GenericError(err_str) => write!(f, "Generic Error: {}", err_str),
            Self::WindowInitError(err_str) => write!(f, "Window Init error: {}", err_str),
            Self::IOError(err_str) => write!(f, "File IO error: {}", err_str),
            _ => write!(f, "unimplemented display! [look in window_util.rs]"),
        }
    }
}
/// Parses Config setting from xml to be used in window building on execution
pub struct FlufflWindowConfigs {
    ///window width
    pub width: u32,
    ///window height
    pub height: u32,
    ///window title ( doesn't apply on wasm )
    pub title: String,
    /// Specifies the id of the canvas (doesn't apply on desktop)
    pub canvas_id: String,
    /// Specifies wasm version either 'webgl2' or 'webgl1'
    pub webgl_version: String,
    /// Specifies dekstop opengl major version
    pub context_major: u8,
    /// Specifies desktop opengl minor version
    pub context_minor: u8,
    /// Specifies if window is resizable
    pub resizable: bool,
    /// Specifies if window is fullscreen
    pub fullscreen: bool,
}

impl FlufflWindowConfigs {
    /// Creates a config POD with relatively sane defaults
    pub fn new() -> Self {
        //These are the default settings
        Self {
            width: 800,
            height: 600,
            title: String::from("fluffl app"),
            canvas_id: String::from("fluffl_canvas"),
            webgl_version: String::from("webgl2"),
            context_major: 3,
            context_minor: 0,
            resizable: true,
            fullscreen: false,
        }
    }

    /// parses config text setting the struct to values specified in thext
    pub fn parser_config_file(mut self, config: &str) -> Self {
        let parser = XMLParser::new().parse(&String::from(config)).unwrap();

        Self::search_numeric(&parser, "width", |num| self.width = num);
        Self::search_numeric(&parser, "height", |num| self.height = num);
        Self::search_numeric(&parser, "context_major", |num| {
            self.context_major = num as u8
        });
        Self::search_numeric(&parser, "context_minor", |num| {
            self.context_minor = num as u8
        });

        Self::search_string(&parser, "title", |text| self.title = text.clone());
        Self::search_string(&parser, "canvas_id", |text| self.canvas_id = text.clone());
        Self::search_string(&parser, "wgl_version", |text| {
            self.webgl_version = text.clone()
        });

        Self::search_bool(&parser, "resizable", |val| self.resizable = val);
        Self::search_bool(&parser, "fullscreen", |val| self.fullscreen = val);
        self
    }

    fn search_bool<Callback>(parser: &XMLParser, tag_name: &str, mut closure: Callback)
    where
        Callback: FnMut(bool),
    {
        if let Some(node_ptr) = parser.search(tag_name, parser.ast.root_list[0]) {
            for (_, data) in parser.get_child_tokens(node_ptr) {
                if let Some(token) = data {
                    let content_text = token.content.trim().to_lowercase();
                    let is_true = content_text == "true";
                    let is_false = content_text == "false";
                    let is_valid_text_boolean = is_true || is_false;
                    if is_valid_text_boolean {
                        closure(is_true);
                    }
                    break;
                }
            }
        }
    }

    fn search_string<Callback>(parser: &XMLParser, tag_name: &str, mut closure: Callback)
    where
        Callback: FnMut(&String),
    {
        if let Some(node_ptr) = parser.search(tag_name, parser.ast.root_list[0]) {
            for (_, data) in parser.get_child_tokens(node_ptr) {
                if let Some(token) = data {
                    closure(&token.content);
                    break;
                }
            }
        }
    }

    fn search_numeric<Callback>(parser: &XMLParser, tag_name: &str, mut closure: Callback)
    where
        Callback: FnMut(u32),
    {
        if let Some(node_ptr) = parser.search(tag_name, parser.ast.root_list[0]) {
            for (_, data) in parser.get_child_tokens(node_ptr) {
                if let Some(token) = data {
                    if let Ok(num) = token.content.parse() {
                        closure(num);
                        break;
                    };
                };
            }
        }
    }
}

impl Default for FlufflWindowConfigs{
    fn default()->Self{
        Self::new()
    }
}