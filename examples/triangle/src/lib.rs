// #![allow(warnings)]

use fluffr::{
    audio::*,
    console::*,
    io::*,
    net::*,
    window_util::{event_util::*, glow::*, *},
    *,
};

use fluffr_util::{
    audio::music_player::*,
    audio::{ogg::*, *},
};

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use console_error_panic_hook;
use std::panic;

#[wasm_bindgen(start)]
pub fn wasm_entry_point() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    spawn_local(async move {
        let _ = fluffr_main().await;
    });
}

type ShortBufferType = OggBuffer;
type ShortMusicPlayer = MusicPlayer<ShortBufferType>;
type ShortDeviceCB = DeviceCB<ShortMusicPlayer>;
type ShortDeviceContext = FluffrAudioDeviceContext<ShortDeviceCB, ShortMusicPlayer>;

pub struct MainState {
    pub device_list: Vec<ShortDeviceContext>,
    pub current_device: usize,
    pub vao: VertexArray,
    pub prog: Program,
    pub socket: Box<dyn HasWebSocketClient>,
}
impl MainState {
    pub fn next_device(&mut self) {
        let len = self.device_list.len();
        self.current_device = (self.current_device + 1) % len;
    }
    pub fn prev_device(&mut self) {
        let len = self.device_list.len();
        self.current_device = (self.current_device + len - 1) % len;
    }
    pub fn get_current_device_depricated(&self) -> ShortDeviceContext {
        self.device_list[self.current_device].clone()
    }

    pub fn get_current_device<F>(&self, mut cb: F)
    where
        F: FnMut(&ShortDeviceContext),
    {
        cb(&self.device_list[self.current_device])
    }

    pub fn delete_current_track(&mut self) {
        if self.device_list.len() > 0 {
            let device_id = self.current_device;
            self.device_list.remove(device_id);
            //clamp device index
            self.current_device = self.current_device.max(self.device_len()).min(0);
        }
    }

    pub fn device_len(&self) -> usize {
        self.device_list.len()
    }

    pub async fn load_music(
        &mut self,
        path: &str,
        audio_context: Arc<RefCell<FluffrAudioContext>>,
    ) -> Result<(), FluffrError> {
        let wav = {
            let wav_data: Vec<u8> = load_file!(path)?;
            ogg::OggFile::new()
                .with_data(wav_data)
                .parse()
                .ok()
                .unwrap()
        };
        let music_core: AudioDeviceCore<ShortDeviceCB, _> = AudioDeviceCore::new()
            .with_specs(DesiredSpecs {
                sample_rate: wav.sample_rate(),
                channels: Some(2),
                buffer_size: Some(4096),
            })
            .with_state(MusicPlayer {
                ticks: 0,
                state: PlayState::RampUp(48000),
                volume: 0.7,
                music_src: wav.into(),
            })
            .with_callback(music_callback);
        let music_device = FluffrAudioDeviceContext::new(music_core, audio_context);
        self.device_list.push(music_device);
        Ok(())
    }
}

pub async fn fluffr_main() -> Result<(), FluffrError> {
    let config_text = "
        <window>
            <width>800</width>
            <height>600</height>
            <title>my_app</title>
        </window>";
    let window = FluffrWindow::init(config_text).unwrap();

    unsafe {
        window.gl().clear_color(0.1, 0.1, 0.1, 1.0);
        window.gl().viewport(0, 0, 512, 512);
    }

    let (vao, prog) = unsafe {
        let vert_source = String::from_utf8(load_file!("./resources/vert.glsl")?).unwrap();
        let frag_source = String::from_utf8(load_file!("./resources/frag.glsl")?).unwrap();
        let geo_data = vec![1.0f32, 0., 0., 1., 0., 1., 0., 1., -1., 0., 0., 1.];
        let color_data = vec![1.0f32, 0., 0., 1., 0., 1., 0., 1., 0., 0., 1., 1.];
        let gl = window.gl();

        let vert_shader: glow::Shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
        gl.shader_source(vert_shader, vert_source.as_str());
        gl.compile_shader(vert_shader);
        if gl.get_shader_compile_status(vert_shader) == false {
            console_log!(
                "Vert shader Failed!\n Reason:\n{}\n",
                gl.get_shader_info_log(vert_shader)
            );
        }

        let frag_shader: glow::Shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
        gl.shader_source(frag_shader, frag_source.as_str());
        gl.compile_shader(frag_shader);
        if gl.get_shader_compile_status(frag_shader) == false {
            console_log!(
                "Frag shader Failed!\n Reason:\n{}\n",
                gl.get_shader_info_log(frag_shader)
            );
        }

        let program: glow::Program = gl.create_program().unwrap();
        gl.use_program(Some(program));
        gl.attach_shader(program, vert_shader);
        gl.attach_shader(program, frag_shader);
        gl.link_program(program);

        if gl.get_program_link_status(program) == false {
            console_log!(
                "Program failed to link!\n Reason:\n{}\n",
                gl.get_program_info_log(program)
            );
        }

        let vao: glow::VertexArray = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        let buffer: glow::Buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));

        //upload geo data gpu
        let data = &geo_data[..];
        let data_as_bytes: &[u8] =
            std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4);
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data_as_bytes, glow::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(1);

        let buffer: glow::Buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));

        //upload vert color data to gpu
        let data = &color_data[..];
        let data_as_bytes: &[u8] =
            std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4);
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data_as_bytes, glow::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(2, 4, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(2);

        (vao, program)
    };

    FluffrWindow::main_loop(
        window,
        MainState {
            device_list: Vec::new(),
            current_device: 0,
            vao,
            prog,
            socket: WsClient::new(())
                .with_on_close_cb(|_state| console_log!("socket closed!\n"))
                .with_on_error_cb(|_state| console_log!("Socket error occured"))
                .with_on_message_cb(|socket, _state, message| {
                    console_log!(
                        "socket is  {}\n",
                        if socket.is_closed() { "closed" } else { "open" }
                    );

                    console_log!("message revieved");
                    let _ = String::from_utf8(message.to_vec()).map(|string_message| {
                        console_log!("recieved a message:{}\n", string_message);
                    });
                })
                .connect("ws://localhost:9001")
                .ok()
                .expect("error occured! connection failed")
                .into(),
        },
        core_loop,
    );
    Ok(())
}

async fn core_loop(
    win_ptr: FluffrWindowPtr,
    _running: Rc<Cell<bool>>,
    main_state: Rc<RefCell<MainState>>,
) {
    let audio_ctx = win_ptr.window().audio_context().clone();
    let gl = win_ptr.window().gl();

    unsafe {
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        gl.use_program(Some(main_state.borrow().prog.clone()));
        gl.bind_vertex_array(Some(main_state.borrow().vao.clone()));
        gl.draw_arrays(glow::TRIANGLES, 0, 3);
        gl.use_program(None);
    }

    for event in win_ptr.window_mut().get_events().iter_mut() {
        match event {
            EventKind::Quit => (),
            EventKind::KeyDown { code } => {
                if let KeyCode::NUM_1 = code {
                    let _ = main_state
                        .borrow_mut()
                        .load_music("./resources/st3.ogg", audio_ctx.clone())
                        .await;
                }
                if let KeyCode::NUM_2 = code {
                    // main_state
                    //     .borrow_mut()
                    //     .load_music("./resources/out.wav", audio_ctx.clone())
                    //     .await;
                }
                if let KeyCode::KEY_A = code {
                    main_state
                        .borrow_mut()
                        .socket
                        .send("hello world from wasm client!\n".as_bytes())
                        .ok()
                        .unwrap();
                }

                if let KeyCode::KEY_R = code {
                    main_state.borrow().get_current_device(|device| {
                        device.modify_state(|state| {
                            state.map(|mp| {
                                mp.state = PlayState::RampUp(12000);
                                mp.ticks = 0;
                                mp.music_src.seek_to_start();
                            });
                        })
                    });
                }

                if let KeyCode::PAGE_UP = code {
                    main_state.borrow().get_current_device(|device| {
                        device.modify_state(|state| {
                            state.map(|mp| {
                                mp.volume = (mp.volume + 0.1).min(1.0).max(0.0);
                            });
                        })
                    })
                }

                if let KeyCode::PAGE_DOWN = code {
                    main_state.borrow().get_current_device(|device| {
                        device.modify_state(|state| {
                            state.map(|mp| {
                                mp.volume = (mp.volume - 0.1).min(1.0).max(0.0);
                            });
                        })
                    });
                }

                if let KeyCode::SPACE = code {
                    if main_state.borrow().device_len() > 0 {
                        main_state.borrow().get_current_device(|dev| {
                            dev.modify_state(|state| {
                                state.map(|s| {
                                    if let PlayState::Paused = s.state {
                                        s.ticks = 0;
                                        s.state = PlayState::RampUp(12000);
                                    }
                                });
                            });
                            dev.resume();
                        });
                    }
                }

                if let KeyCode::DELETE = code {
                    main_state.borrow_mut().delete_current_track();
                }

                if let KeyCode::KEY_Y = code {
                    main_state.borrow().get_current_device(|device| {
                        device.modify_state(|state| {
                            state.map(|s| {
                                if let PlayState::Playing = s.state {
                                    s.ticks = 0;
                                    s.state = PlayState::RampDown(12000);
                                }
                            });
                        });
                        device.resume();
                    });
                }

                if let KeyCode::ARROW_RIGHT = code {
                    main_state.borrow_mut().next_device();
                }
                if let KeyCode::ARROW_LEFT = code {
                    main_state.borrow_mut().prev_device();
                }

                let ncode: i128 = code.into();
                console_log!("char = {}\n", (ncode as u8 as char).to_lowercase());
                // if (ncode >= KeyCode::KEY_A.into()) && (ncode <= KeyCode::KEY_Z.into()) {
                // }
            }
            EventKind::MouseMove { x, y, dx, dy } => {
                console_log!("mouse move: [x:{},y:{},dx:{},dy:{}\n", x, y, dx, dy);
            }
            EventKind::MouseUp { button_code, x, y } => {
                console_log!("mouse up at: [x:{},y:{}]\n", x, y);
                console_log!("{}\n", button_code);
            }
            EventKind::MouseDown { button_code, x, y } => {
                console_log!("mouse down at: [x:{},y:{}]\n", x, y);
                console_log!("{}\n", button_code);
            }
            EventKind::MouseWheel { button_code } => {
                console_log!("{}\n", button_code);
            }
            _ => (),
        }
    }
}
