use super::{
    event_util::{constants::*, FlufflEvent},
    *,
};
use crate::FlufflState;

use glow::*;

use std::{cell::RefCell, mem, rc::Rc, sync::Arc};

use be_sdl2::{event::Event, keyboard::Scancode, mouse::MouseButton, video::FullscreenType};

///Global for touch tracker
static mut GLOBAL_TOUCH_TRACKER: Option<TouchTracker<i32>> = None;

impl TouchTracker<i32> {
    /// # Description
    /// Initalizes tracker. Tracker routines will panic if this function isn't called.
    pub fn init() {
        unsafe {
            GLOBAL_TOUCH_TRACKER = Some(TouchTracker::new());
        }
    }
    pub fn get_mut() -> &'static mut Self {
        unsafe {
            GLOBAL_TOUCH_TRACKER
                .as_mut()
                .expect("tracker not initalized")
        }
    }
}

/// # Description
/// A Custom SDL2 Window pointer type. This is needed because `RenderLoop<T>` takes the window provided by the
/// sdl2 wrapper and never gives it back.
struct CustomSDL2Window {
    #[allow(dead_code)]
    context: Rc<be_sdl2::video::WindowContext>,
}

impl CustomSDL2Window {
    fn new(context: Rc<be_sdl2::video::WindowContext>) -> Self {
        Self { context }
    }

    fn to_window(&mut self) -> &mut be_sdl2::video::Window {
        let mut_ref: &mut Self = self;
        unsafe { mem::transmute(mut_ref) }
    }
}

/// a very simple macro to shorten up text a little bit
macro_rules! push_event {
    ( $event_pump:ident , $event:expr  ) => {
        $event_pump.as_mut().unwrap().push_event($event)
    };
}

#[allow(dead_code)]
/// A cross-platform window handler, use it to set up opengl and listen to input devices in a platform
/// agnostic fashion
pub struct FlufflWindow {
    glue_event: Option<FlufflEvent>,
    sdl_state: be_sdl2::Sdl,
    sdl_gl_context: be_sdl2::video::GLContext,
    sdl_event_pump: be_sdl2::EventPump,
    gl: Arc<Box<Context>>,
    render_loop: Option<RenderLoop<be_sdl2::video::Window>>,
    audio_context: FlufflAudioContext,
    video_ss: be_sdl2::VideoSubsystem,
    window_width: u32,
    window_height: u32,
    window_pointer: CustomSDL2Window,
}

impl FlufflWindow {
    //moves renderloop out of glue_window and calls it
    fn get_render_loop(&mut self) -> impl glow::HasRenderLoop {
        self.render_loop.take().unwrap()
    }

    /// Loops infinitely unless caller mutates `running` to false. Just executes the user defined taske over and over while also \
    /// doing basic matenence for events and the window
    /// # Arguments
    /// * `fluffl_window` an initalized fluffl window
    /// * `app_state` is the variables and memory that is needed throughout the entire lifespan of the program
    /// * `core_loop` is the user-defined 'main_loop'
    pub fn run<Loop, LoopOut, State>(self, app_state: State, core_loop: Loop)
    where
        Loop: Fn(FlufflWindowPtr, FlufflRunning, FlufflState<State>) -> LoopOut + Copy + 'static,
        LoopOut: std::future::Future<Output = ()>,
        State: 'static,
    {
        let mut fluffl_window = self;
        let render_loop = fluffl_window.get_render_loop();

        let window_ptr = FlufflWindowPtr {
            ptr: Arc::new(RefCell::new(fluffl_window)),
        };

        let state_ptr = FlufflState::new(app_state);

        render_loop.run(move |running| {
            window_ptr.window_mut().collect_events();

            let unexecuted_iteration = core_loop(
                window_ptr.clone(),
                FlufflRunning::new(running),
                state_ptr.clone(),
            );

            //execute future
            futures::executor::block_on(unexecuted_iteration);
        });
    }
}

impl HasFlufflWindow for FlufflWindow {
    fn width(&self) -> u32 {
        self.window_width
    }

    fn height(&self) -> u32 {
        self.window_height
    }

    fn audio_context(&self) -> FlufflAudioContext {
        self.audio_context.clone()
    }

    fn gl(&self) -> Arc<Box<Context>> {
        self.gl.clone()
    }

    fn get_events(&mut self) -> &mut FlufflEvent {
        self.glue_event.as_mut().unwrap()
    }

    fn init(config: &str) -> Result<Self, FlufflError> {
        let settings = FlufflWindowConfigs::new().parser_config_file(config);

        // Create a context from a sdl2 window
        let sdl = be_sdl2::init()?;
        let audio = sdl.audio()?;
        let video = sdl.video()?;

        let gl_attr = video.gl_attr();

        gl_attr.set_context_profile(be_sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(settings.context_major, settings.context_minor);
        //set stencil buffer
        video.gl_attr().set_stencil_size(8);

        let win_builder = video.window(settings.title.as_str(), settings.width, settings.height);

        let build_window_according_to_settings = |mut builder: be_sdl2::video::WindowBuilder| {
            let mut builder_ref = builder.opengl();
            if settings.resizable {
                builder_ref = builder.resizable();
            }
            if settings.resizable == false && settings.fullscreen {
                builder_ref = builder.fullscreen();
            }

            builder_ref.build()
        };

        let window = build_window_according_to_settings(win_builder)?;

        // because be_sdl2::video::Window is a glorified smart pointer I can do this:
        let window_context_ref =
            unsafe { mem::transmute::<_, &Rc<be_sdl2::video::WindowContext>>(&window) };

        let gl_context = window.gl_create_context()?;

        let context = unsafe {
            glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _)
        };

        let render_loop = Some(glow::RenderLoop::<be_sdl2::video::Window>::from_sdl_window(
            window,
        ));
        let event_loop = sdl.event_pump()?;

        TouchTracker::init();

        // disable vsync
        // video.gl_set_swap_interval(0).expect("failed to disable vsync:");

        let fluffl_window = Self {
            sdl_gl_context: gl_context,
            sdl_event_pump: event_loop,
            glue_event: Some(FlufflEvent::new()),
            gl: Arc::new(Box::new(context)),
            render_loop,
            window_width: settings.width,
            window_height: settings.height,
            sdl_state: sdl,
            audio_context: FlufflAudioContext {
                audio_ss: Arc::new(RefCell::new(audio)),
            },
            video_ss: video,
            window_pointer: CustomSDL2Window::new(window_context_ref.clone()),
        };

        Ok(fluffl_window)
    }

    fn set_fullscreen(&mut self, go_fullscreen: bool) {
        let window = self.window_pointer.to_window();
        if go_fullscreen {
            if let Err(msg) = window.set_fullscreen(FullscreenType::True) {
                panic!("Error:{}", msg)
            }
        } else {
            if let Err(msg) = window.set_fullscreen(FullscreenType::Off) {
                panic!("Error:{}", msg)
            }
        }
    }
}

impl From<String> for FlufflError {
    fn from(string: String) -> Self {
        Self::WindowInitError(string)
    }
}

impl From<be_sdl2::video::WindowBuildError> for FlufflError {
    fn from(build_err: be_sdl2::video::WindowBuildError) -> Self {
        FlufflError::WindowInitError(build_err.to_string())
    }
}

impl HasEventCollection for FlufflWindow {
    //Basically passes events from SDL's event queue into a more Generic Queue
    fn collect_events(&mut self) {
        let mut gevent = self.glue_event.take();
        let mut width_update = None;
        let mut height_update = None;

        let (cur_width, cur_height) = self.get_bounds_f32();

        self.sdl_event_pump
            .poll_iter()
            .for_each(|event| match event {
                Event::Quit { .. } => {
                    push_event!(gevent, EventKind::Quit);
                }

                Event::FingerDown {
                    finger_id, x, y, ..
                } => {
                    let id = finger_id as i32;
                    let mouse_pos = [cur_width * x, cur_height * y];
                    let _ = TouchTracker::get_mut().get_touch_displacement(id, mouse_pos);
                    push_event!(
                        gevent,
                        EventKind::TouchDown {
                            finger_id: id,
                            x: mouse_pos[0],
                            y: mouse_pos[1],
                            dx: 0.,
                            dy: 0.,
                        }
                    );
                }

                Event::FingerMotion {
                    finger_id, x, y, ..
                } => {
                    let id = finger_id as i32;
                    let mouse_pos = [cur_width * x, cur_height * y];
                    let [dx, dy] = TouchTracker::get_mut().get_touch_displacement(id, mouse_pos);
                    push_event!(
                        gevent,
                        EventKind::TouchMove {
                            finger_id: id,
                            x: mouse_pos[0],
                            y: mouse_pos[1],
                            dx,
                            dy,
                        }
                    );
                }

                Event::FingerUp {
                    finger_id, x, y, ..
                } => {
                    let id = finger_id as i32;
                    let mouse_pos = [cur_width * x, cur_height * y];
                    let _ = TouchTracker::get_mut().get_touch_displacement(id, mouse_pos);
                    push_event!(
                        gevent,
                        EventKind::TouchUp {
                            finger_id: id,
                            x: mouse_pos[0],
                            y: mouse_pos[1],
                            dx: 0.,
                            dy: 0.,
                        }
                    );
                    // Its important to remove info associated with id when finger is released
                    // because SDL2 will assign a new id for every FingerDown
                    // and we only want to track unique fingers detected by the touchscreen
                    TouchTracker::get_mut().remove(&id);
                }

                Event::Window { win_event, .. } => match win_event {
                    be_sdl2::event::WindowEvent::Resized(width, height) => {
                        push_event!(gevent, EventKind::Resize { width, height });
                        width_update = Some(width as u32);
                        height_update = Some(height as u32);
                    }
                    _ => (),
                },

                Event::KeyUp { scancode, .. } => {
                    scancode.map(|sc| {
                        let code = map_scancode(sc);
                        push_event!(gevent, EventKind::KeyUp { code })
                    });
                }
                Event::KeyDown { scancode, .. } => {
                    scancode.map(|sc| {
                        let code = map_scancode(sc);
                        push_event!(gevent, EventKind::KeyDown { code })
                    });
                }
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => match mouse_btn {
                    MouseButton::Left => push_event!(
                        gevent,
                        EventKind::MouseDown {
                            button_code: MouseCode::LEFT_BUTTON,
                            x,
                            y
                        }
                    ),
                    MouseButton::Right => push_event!(
                        gevent,
                        EventKind::MouseDown {
                            button_code: MouseCode::RIGHT_BUTTON,
                            x,
                            y
                        }
                    ),
                    MouseButton::Middle => push_event!(
                        gevent,
                        EventKind::MouseDown {
                            button_code: MouseCode::WHEEL { direction: 0 },
                            x,
                            y
                        }
                    ),
                    _ => (),
                },
                Event::MouseButtonUp {
                    mouse_btn, x, y, ..
                } => match mouse_btn {
                    MouseButton::Left => push_event!(
                        gevent,
                        EventKind::MouseUp {
                            button_code: MouseCode::LEFT_BUTTON,
                            x,
                            y
                        }
                    ),
                    MouseButton::Right => push_event!(
                        gevent,
                        EventKind::MouseUp {
                            button_code: MouseCode::RIGHT_BUTTON,
                            x,
                            y
                        }
                    ),
                    MouseButton::Middle => push_event!(
                        gevent,
                        EventKind::MouseUp {
                            button_code: MouseCode::WHEEL { direction: 0 },
                            x,
                            y
                        }
                    ),
                    _ => (),
                },
                Event::MouseMotion {
                    x, y, xrel, yrel, ..
                } => {
                    push_event!(
                        gevent,
                        EventKind::MouseMove {
                            x,
                            y,
                            dx: xrel,
                            dy: yrel
                        }
                    );
                }
                Event::MouseWheel { y, .. } => {
                    push_event!(
                        gevent,
                        EventKind::MouseWheel {
                            button_code: MouseCode::WHEEL { direction: y }
                        }
                    );
                }
                _ => (),
            });

        width_update.map(|width| self.window_width = width);
        height_update.map(|height| self.window_height = height);
        //make sure to give the pollevent back
        self.glue_event = gevent;
    }
}

fn map_scancode(scancode: be_sdl2::keyboard::Scancode) -> KeyCode {
    match scancode {
        Scancode::A => KeyCode::KEY_A,
        Scancode::B => KeyCode::KEY_B,
        Scancode::C => KeyCode::KEY_C,
        Scancode::D => KeyCode::KEY_D,
        Scancode::E => KeyCode::KEY_E,
        Scancode::F => KeyCode::KEY_F,
        Scancode::G => KeyCode::KEY_G,
        Scancode::H => KeyCode::KEY_H,
        Scancode::I => KeyCode::KEY_I,
        Scancode::J => KeyCode::KEY_J,
        Scancode::K => KeyCode::KEY_K,
        Scancode::L => KeyCode::KEY_L,
        Scancode::M => KeyCode::KEY_M,
        Scancode::N => KeyCode::KEY_N,
        Scancode::O => KeyCode::KEY_O,
        Scancode::P => KeyCode::KEY_P,
        Scancode::Q => KeyCode::KEY_Q,
        Scancode::R => KeyCode::KEY_R,
        Scancode::S => KeyCode::KEY_S,
        Scancode::T => KeyCode::KEY_T,
        Scancode::U => KeyCode::KEY_U,
        Scancode::V => KeyCode::KEY_V,
        Scancode::W => KeyCode::KEY_W,
        Scancode::X => KeyCode::KEY_X,
        Scancode::Y => KeyCode::KEY_Y,
        Scancode::Z => KeyCode::KEY_Z,
        Scancode::Num0 => KeyCode::NUM_0,
        Scancode::Num1 => KeyCode::NUM_1,
        Scancode::Num2 => KeyCode::NUM_2,
        Scancode::Num3 => KeyCode::NUM_3,
        Scancode::Num4 => KeyCode::NUM_4,
        Scancode::Num5 => KeyCode::NUM_5,
        Scancode::Num6 => KeyCode::NUM_6,
        Scancode::Num7 => KeyCode::NUM_7,
        Scancode::Num8 => KeyCode::NUM_8,
        Scancode::Num9 => KeyCode::NUM_9,
        Scancode::Return => KeyCode::ENTER,
        Scancode::Escape => KeyCode::ESC,
        Scancode::Tab => KeyCode::TAB,
        Scancode::LShift => KeyCode::SHIFT_L,
        Scancode::RShift => KeyCode::SHIFT_R,
        Scancode::Space => KeyCode::SPACE,
        Scancode::Minus => KeyCode::MINUS,
        Scancode::Equals => KeyCode::EQUALS,
        Scancode::LeftBracket => KeyCode::BRACKET_L,
        Scancode::RightBracket => KeyCode::BRACKET_R,
        Scancode::Backslash => KeyCode::BACKSLASH,
        Scancode::NonUsHash => KeyCode::NUM_2,
        Scancode::Semicolon => KeyCode::COLON,
        Scancode::Apostrophe => KeyCode::QUOTE,
        Scancode::Grave => KeyCode::BACK_QUOTE,
        Scancode::Comma => KeyCode::COMMA,
        Scancode::Period => KeyCode::PERIOD,
        Scancode::Slash => KeyCode::FORDSLASH,
        Scancode::CapsLock => KeyCode::CAPSLOCK,
        Scancode::LCtrl => KeyCode::CTRL_L,
        Scancode::RCtrl => KeyCode::CTRL_R,
        Scancode::LAlt => KeyCode::ALT_L,
        Scancode::RAlt => KeyCode::ALT_R,
        Scancode::F1 => KeyCode::F1,
        Scancode::F2 => KeyCode::F2,
        Scancode::F3 => KeyCode::F3,
        Scancode::F4 => KeyCode::F4,
        Scancode::F5 => KeyCode::F5,
        Scancode::F6 => KeyCode::F6,
        Scancode::F7 => KeyCode::F7,
        Scancode::F8 => KeyCode::F8,
        Scancode::F9 => KeyCode::F9,
        Scancode::F10 => KeyCode::F10,
        Scancode::F11 => KeyCode::F11,
        Scancode::F12 => KeyCode::F12,
        Scancode::PrintScreen => KeyCode::PRINT_SCREEN,
        Scancode::ScrollLock => KeyCode::SCROLL_LOCK,
        Scancode::Pause => KeyCode::PAUSE,
        Scancode::Delete => KeyCode::DELETE,
        Scancode::End => KeyCode::END,
        Scancode::PageDown => KeyCode::PAGE_D,
        Scancode::PageUp => KeyCode::PAGE_U,
        Scancode::Left => KeyCode::ARROW_L,
        Scancode::Right => KeyCode::ARROW_R,
        Scancode::Up => KeyCode::ARROW_U,
        Scancode::Down => KeyCode::ARROW_D,
        Scancode::Backspace => KeyCode::BACKSPACE,
        _ => KeyCode::UNKNOWN,
    }
}