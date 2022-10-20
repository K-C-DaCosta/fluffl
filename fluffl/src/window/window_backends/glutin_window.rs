use super::{
    event_util::{constants::*, FlufflEvent},
    *,
};

use crate::FlufflState;

use be_glutin::{
    self,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        DeviceId, ElementState, Event, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
    },
    event_loop::ControlFlow,
    ContextWrapper,
};

///Global for touch tracker
static mut GLOBAL_TOUCH_TRACKER: Option<TouchTracker<DeviceId>> = None;

impl TouchTracker<DeviceId> {
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

pub struct FlufflWindow {
    gl: GlowGL,
    events: FlufflEvent,
    render_loop: Option<be_glutin::event_loop::EventLoop<()>>,
    window: ContextWrapper<be_glutin::PossiblyCurrent, be_glutin::window::Window>,
}

impl FlufflWindow {
    //moves renderloop out of glue_window and calls it
    fn get_render_loop(&mut self) -> be_glutin::event_loop::EventLoop<()> {
        self.render_loop
            .take()
            .expect("render loop not set.  Maybe FlufflHasWindow::init(..) contains a bug?  ")
    }
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

        render_loop.run(move |event, _, control_flow| {
            let mut is_running = true;
            match event {
                Event::MainEventsCleared | Event::RedrawRequested(_) => {
                    window_ptr
                        .window()
                        .window
                        .swap_buffers()
                        .expect("failed to swap buffers");
                    //execute future
                    futures::executor::block_on(core_loop(
                        window_ptr.clone(),
                        FlufflRunning::new(&mut is_running),
                        state_ptr.clone(),
                    ));
                    if !is_running {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                _ => {
                    window_ptr
                        .window_mut()
                        .convert_glutin_event_to_fluffl_event(event);
                }
            }
        });
    }

    fn convert_glutin_event_to_fluffl_event(&mut self, glutin_event: be_glutin::event::Event<()>) {
        match glutin_event {
            Event::Resumed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    self.events.push_event(EventKind::Quit);
                }
                WindowEvent::KeyboardInput {
                    input,
                    is_synthetic: _,
                    ..
                } => match (input.state, input.virtual_keycode) {
                    (ElementState::Pressed, Some(code)) => {
                        let code = map_virtual_keycode_to_fluffl(code).expect("couldnt map key");
                        self.events.push_event(EventKind::KeyDown { code })
                    }
                    (ElementState::Released, Some(code)) => {
                        let code = map_virtual_keycode_to_fluffl(code).expect("couldnt map key");
                        self.events.push_event(EventKind::KeyUp { code })
                    }
                    _ => (),
                },
                WindowEvent::MouseInput {
                    device_id,
                    state,
                    button,
                    ..
                } => {
                    let button_code = mouse_button_to_mouse_code(button)
                        .expect("glutin::MouseButton could not be translated");

                    let device_state = TouchTracker::get_mut()
                        .get(&device_id)
                        .copied()
                        .expect("device_id not found");

                    let fluffl_event = match state {
                        ElementState::Pressed => EventKind::MouseDown {
                            button_code,
                            x: device_state.prev_pos[0] as i32,
                            y: device_state.prev_pos[1] as i32,
                        },
                        ElementState::Released => {
                            // it appears that I dont have to remove on glutin
                            // TouchTracker::get_mut().remove(&device_id);
                            EventKind::MouseUp {
                                button_code,
                                x: device_state.prev_pos[0] as i32,
                                y: device_state.prev_pos[1] as i32,
                            }
                        }
                    };

                    self.events.push_event(fluffl_event);
                }
                WindowEvent::CursorMoved {
                    device_id,
                    position: PhysicalPosition { x, y },
                    ..
                } => {
                    let device_displacement = TouchTracker::get_mut()
                        .get_touch_displacement(device_id, [x as f32, y as f32]);

                    self.events.push_event(EventKind::MouseMove {
                        x: x as i32,
                        y: y as i32,
                        dx: device_displacement[0] as i32,
                        dy: device_displacement[1] as i32,
                    })
                }

                WindowEvent::MouseWheel { delta, .. } => {
                    self.events.push_event(EventKind::MouseWheel {
                        button_code: MouseCode::WHEEL {
                            direction: match delta {
                                MouseScrollDelta::LineDelta(_x, y) => y.signum() as i32,
                                _ => 0,
                            },
                        },
                    })
                }
                WindowEvent::Resized(PhysicalSize { width, height }) => {
                    self.events.push_event(EventKind::Resize {
                        width: width as i32,
                        height: height as i32,
                    })
                }
                _ => (),
            },
            _ => (),
        }
    }
}
impl HasFlufflWindow for FlufflWindow {
    fn init(config: &str) -> Result<Self, FlufflError> {
        let settings = FlufflWindowConfigs::new().parser_config_file(config);

        let event_loop = be_glutin::event_loop::EventLoop::new();
        let mut window_builder = be_glutin::window::WindowBuilder::new();

        window_builder = window_builder.with_title(settings.title);
        window_builder = window_builder.with_inner_size(be_glutin::dpi::LogicalSize::new(
            settings.width,
            settings.height,
        ));

        let window = unsafe {
            be_glutin::ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
                .expect("failed to build window")
                .make_current()
                .expect("failed to make_current(..)")
        };
        let gl = unsafe {
            glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _)
        };

        TouchTracker::init();

        Ok(Self {
            window,
            events: FlufflEvent::new(),
            render_loop: Some(event_loop),
            gl: Arc::new(Box::new(gl)),
        })
    }

    fn get_events(&mut self) -> &mut FlufflEvent {
        &mut self.events
    }

    fn gl(&self) -> GlowGL {
        self.gl.clone()
    }

    fn audio_context(&self) -> FlufflAudioContext {
        FlufflAudioContext::default()
    }

    fn width(&self) -> u32 {
        self.window.window().inner_size().width
    }

    fn height(&self) -> u32 {
        self.window.window().inner_size().height
    }

    fn set_fullscreen(&mut self, go_fullscreen: bool) {
        let window = self.window.window();
        let monitor_handle = window.current_monitor();
        window.set_fullscreen(
            go_fullscreen.then_some(be_glutin::window::Fullscreen::Borderless(monitor_handle)),
        );
    }
}

pub fn mouse_button_to_mouse_code(mb: MouseButton) -> Option<MouseCode> {
    let translation = match mb {
        MouseButton::Left => MouseCode::LEFT_BUTTON,
        MouseButton::Right => MouseCode::RIGHT_BUTTON,
        MouseButton::Middle => MouseCode::WHEEL { direction: 0 },
        _ => return None,
    };
    Some(translation)
}

pub fn map_virtual_keycode_to_fluffl(code: VirtualKeyCode) -> Option<KeyCode> {
    let key_code = match code {
        VirtualKeyCode::Key1 => KeyCode::NUM_1,
        VirtualKeyCode::Key2 => KeyCode::NUM_2,
        VirtualKeyCode::Key3 => KeyCode::NUM_3,
        VirtualKeyCode::Key4 => KeyCode::NUM_4,
        VirtualKeyCode::Key5 => KeyCode::NUM_5,
        VirtualKeyCode::Key6 => KeyCode::NUM_6,
        VirtualKeyCode::Key7 => KeyCode::NUM_7,
        VirtualKeyCode::Key8 => KeyCode::NUM_8,
        VirtualKeyCode::Key9 => KeyCode::NUM_9,
        VirtualKeyCode::Key0 => KeyCode::NUM_0,
        VirtualKeyCode::A => KeyCode::KEY_A,
        VirtualKeyCode::B => KeyCode::KEY_B,
        VirtualKeyCode::C => KeyCode::KEY_C,
        VirtualKeyCode::D => KeyCode::KEY_D,
        VirtualKeyCode::E => KeyCode::KEY_E,
        VirtualKeyCode::F => KeyCode::KEY_F,
        VirtualKeyCode::G => KeyCode::KEY_G,
        VirtualKeyCode::H => KeyCode::KEY_H,
        VirtualKeyCode::I => KeyCode::KEY_I,
        VirtualKeyCode::J => KeyCode::KEY_J,
        VirtualKeyCode::K => KeyCode::KEY_K,
        VirtualKeyCode::L => KeyCode::KEY_L,
        VirtualKeyCode::M => KeyCode::KEY_M,
        VirtualKeyCode::N => KeyCode::KEY_N,
        VirtualKeyCode::O => KeyCode::KEY_O,
        VirtualKeyCode::P => KeyCode::KEY_P,
        VirtualKeyCode::Q => KeyCode::KEY_Q,
        VirtualKeyCode::R => KeyCode::KEY_R,
        VirtualKeyCode::S => KeyCode::KEY_S,
        VirtualKeyCode::T => KeyCode::KEY_T,
        VirtualKeyCode::U => KeyCode::KEY_U,
        VirtualKeyCode::V => KeyCode::KEY_V,
        VirtualKeyCode::W => KeyCode::KEY_W,
        VirtualKeyCode::X => KeyCode::KEY_X,
        VirtualKeyCode::Y => KeyCode::KEY_Y,
        VirtualKeyCode::Z => KeyCode::KEY_Z,
        VirtualKeyCode::Escape => KeyCode::ESC,
        VirtualKeyCode::F1 => KeyCode::F1,
        VirtualKeyCode::F2 => KeyCode::F2,
        VirtualKeyCode::F3 => KeyCode::F3,
        VirtualKeyCode::F4 => KeyCode::F4,
        VirtualKeyCode::F5 => KeyCode::F5,
        VirtualKeyCode::F6 => KeyCode::F6,
        VirtualKeyCode::F7 => KeyCode::F7,
        VirtualKeyCode::F8 => KeyCode::F8,
        VirtualKeyCode::F9 => KeyCode::F9,
        VirtualKeyCode::F10 => KeyCode::F10,
        VirtualKeyCode::F11 => KeyCode::F11,
        VirtualKeyCode::F12 => KeyCode::F12,
        VirtualKeyCode::Scroll => KeyCode::SCROLL_LOCK,
        VirtualKeyCode::Pause => KeyCode::PAUSE,
        VirtualKeyCode::Insert => KeyCode::INSERT,
        VirtualKeyCode::Home => KeyCode::HOME,
        VirtualKeyCode::Delete => KeyCode::DELETE,
        VirtualKeyCode::End => KeyCode::END,
        VirtualKeyCode::PageDown => KeyCode::PAGE_D,
        VirtualKeyCode::PageUp => KeyCode::PAGE_U,
        VirtualKeyCode::Left => KeyCode::ARROW_L,
        VirtualKeyCode::Up => KeyCode::ARROW_U,
        VirtualKeyCode::Right => KeyCode::ARROW_R,
        VirtualKeyCode::Down => KeyCode::ARROW_D,
        VirtualKeyCode::Back => KeyCode::BACKSPACE,
        VirtualKeyCode::Return => KeyCode::ENTER,
        VirtualKeyCode::Space => KeyCode::SPACE,
        VirtualKeyCode::Numlock => KeyCode::NUMLOCK,
        VirtualKeyCode::Numpad0 => KeyCode::KP_0,
        VirtualKeyCode::Numpad1 => KeyCode::KP_1,
        VirtualKeyCode::Numpad2 => KeyCode::KP_2,
        VirtualKeyCode::Numpad3 => KeyCode::KP_3,
        VirtualKeyCode::Numpad4 => KeyCode::KP_4,
        VirtualKeyCode::Numpad5 => KeyCode::KP_5,
        VirtualKeyCode::Numpad6 => KeyCode::KP_6,
        VirtualKeyCode::Numpad7 => KeyCode::KP_7,
        VirtualKeyCode::Numpad8 => KeyCode::KP_8,
        VirtualKeyCode::Numpad9 => KeyCode::KP_9,
        VirtualKeyCode::NumpadAdd => KeyCode::KP_PLUS,
        VirtualKeyCode::NumpadDivide => KeyCode::KP_MINUS,
        VirtualKeyCode::NumpadDecimal => KeyCode::KP_DECIMAL,
        VirtualKeyCode::NumpadEnter => KeyCode::KP_ENTER,
        VirtualKeyCode::NumpadSubtract => KeyCode::KP_MINUS,
        VirtualKeyCode::Apostrophe => KeyCode::QUOTE,
        VirtualKeyCode::Backslash => KeyCode::BACKSLASH,
        VirtualKeyCode::Comma => KeyCode::COMMA,
        VirtualKeyCode::Equals => KeyCode::EQUALS,
        VirtualKeyCode::Grave => KeyCode::BACK_QUOTE,
        VirtualKeyCode::Minus => KeyCode::MINUS,
        VirtualKeyCode::Period => KeyCode::PERIOD,
        VirtualKeyCode::RAlt => KeyCode::ALT_R,
        VirtualKeyCode::RBracket => KeyCode::BRACKET_R,
        VirtualKeyCode::RControl => KeyCode::CTRL_R,
        VirtualKeyCode::RShift => KeyCode::SHIFT_R,
        VirtualKeyCode::RWin => KeyCode::SUPER_R,
        VirtualKeyCode::LAlt => KeyCode::ALT_L,
        VirtualKeyCode::LBracket => KeyCode::BRACKET_L,
        VirtualKeyCode::LControl => KeyCode::CTRL_L,
        VirtualKeyCode::LShift => KeyCode::SHIFT_L,
        VirtualKeyCode::LWin => KeyCode::SUPER_L,
        VirtualKeyCode::Semicolon => KeyCode::COLON,
        VirtualKeyCode::Slash => KeyCode::FORDSLASH,
        VirtualKeyCode::Sleep => KeyCode::SLEEP,
        VirtualKeyCode::Tab => KeyCode::TAB,
        VirtualKeyCode::Underline => KeyCode::MINUS,
        _ => return None,
    };

    Some(key_code)
}