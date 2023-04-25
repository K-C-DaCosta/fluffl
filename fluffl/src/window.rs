use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    fmt,
    sync::Arc,
};

pub mod event_util;
pub mod touch_tracker;
mod window_backends;

use crate::{audio::FlufflAudioContext, Error, GlowGL};
use serde_json::{Map, Value};

pub use event_util::FlufflEvent;
use touch_tracker::*;
pub use window_backends::*;

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
    fn init(config: &str) -> Result<Self, Error>;
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::GenericError(err_str) => write!(f, "Generic Error: {}", err_str),
            Self::WindowInitError(err_str) => write!(f, "Window Init error: {}", err_str),
            Self::IOError(err_str) => write!(f, "File IO error: {}", err_str),
            _ => write!(f, "unimplemented display! [look in window_util.rs]"),
        }
    }
}
#[derive()]
pub enum IconSetting {
    Base64(String),
    Path(String),
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
    pub icon: Option<IconSetting>,
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
            icon: None,
        }
    }

    /// parses config text setting the struct to values specified in thext
    pub fn parse_config_file(mut self, config: &str) -> Result<Self, serde_json::Error> {
        let obj = serde_json::from_str::<Map<String, Value>>(config)?;

        if let Some(val) = obj.get("width").and_then(|num| num.as_u64()) {
            self.width = val as u32;
        }

        if let Some(val) = obj.get("height").and_then(|num| num.as_u64()) {
            self.height = val as u32;
        }

        if let Some(val) = obj.get("context_major").and_then(|num| num.as_u64()) {
            self.context_major = val as u8;
        }

        if let Some(val) = obj.get("context_minor").and_then(|num| num.as_u64()) {
            self.context_minor = val as u8;
        }

        if let Some(val) = obj.get("title").and_then(|num| num.as_str()) {
            self.title = String::from(val);
        }

        if let Some(val) = obj.get("canvas_id").and_then(|num| num.as_str()) {
            self.canvas_id = String::from(val);
        }

        if let Some(val) = obj.get("wgl_version").and_then(|num| num.as_str()) {
            self.webgl_version = String::from(val);
        }

        if let Some(val) = obj.get("resizable").and_then(|num| num.as_bool()) {
            self.resizable = val;
        }
        if let Some(val) = obj.get("fullscreen").and_then(|num| num.as_bool()) {
            self.fullscreen = val;
        }

        if let Some(icon_obj) = obj.get("icon").and_then(|val| val.as_object()) {
            if let Some(path) = icon_obj.get("path").and_then(|val| val.as_str()) {
                self.icon = Some(IconSetting::Path(String::from(path)));
            }
            if let Some(path) = icon_obj.get("base64").and_then(|val| val.as_str()) {
                self.icon = Some(IconSetting::Base64(
                    path.chars().filter(|c| !c.is_whitespace()).collect(),
                ));
            }
        }

        Ok(self)
    }
}

impl Default for FlufflWindowConfigs {
    fn default() -> Self {
        Self::new()
    }
}
