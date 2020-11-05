
use fluffl::{
    audio::*,
    console::*,
    io::*,
    net::*,
    window::{event_util::*, glow::*, *},
    *,
};

//playing music requires more than what the base library provides 
//so here is my implementation of certain things like "text rendering" and music playign
use fluffl::extras::{
    audio::music_player::*,
    audio::{ogg::*, *},
    hiero_pack::*,
    ogl::text_writer::*,
    ogl::{array::*, buffer::*, program::*, texture::*, *},
};

//This is the entry point for the web target (not default )
#[cfg(feature="web")]
#[wasm_bindgen(start)]
pub fn wasm_entry_point() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    spawn_local(async move {
        let _ = fluffr_main().await;
    });
}






// use glue_core:
#[derive(Default)]
pub struct ClientState {
    pub foo: i32,
}

use std::cell::{Cell, RefCell};
use std::rc::Rc;

type ShortBufferType = OggBuffer;
type ShortMusicPlayer = MusicPlayer<ShortBufferType>;
type ShortDeviceCB = DeviceCB<MusicPlayer<ShortBufferType>>;
type ShortDeviceContext = FlufflAudioDeviceContext<ShortDeviceCB, ShortMusicPlayer>;

pub struct MainState {
    pub dev_ptr: ShortDeviceContext,
    pub vao: OglArray,
    pub progbox: OglProg,
    pub t: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub tex: Texture,
    pub noise: OglTexture,
    pub writer: TextWriter,
    pub client_socket: Box<dyn HasWebSocketClient>,
}

pub async fn fluffl_main() -> Result<(), FlufflError> {
    
    let ogg = {
        let wav_data: Vec<u8> = load_file!("./resources/st2.ogg")?;
        ogg::OggFile::new()
            .with_data(wav_data)
            .parse()
            .expect("parse failed")
    };

    //GlueWindow is configured with XML, the format is self-explanitory
    let raw_bytes = load_file!("./resources/config.xml")?;
    let config_text = String::from_utf8(raw_bytes)?;
    let window = FlufflWindow::init(config_text.as_str())?;
    let gl = window.gl();

    unsafe {
        window.gl().clear_color(0.1, 0.1, 0.1, 1.);
        window
            .gl()
            .viewport(0, 0, window.width() as i32, window.height() as i32);
    }
    console_log!("widht = {}, height = {}\n", window.width(), window.height());

    let device_core: AudioDeviceCore<ShortDeviceCB, ShortMusicPlayer> = AudioDeviceCore::new()
        .with_specs(DesiredSpecs {
            sample_rate: ogg.sample_rate().map(|a| a as u32),
            channels: Some(2),
            buffer_size: None,
        })
        .with_state(MusicPlayer {
            ticks: 0,
            state: PlayState::Paused,
            volume: 0.7,
            music_src: ogg.into(),
        })
        .with_callback(music_callback);

    let device = FlufflAudioDeviceContext::new(device_core, window.audio_context());

    let atlas_bin = load_file!("font.bcode").unwrap();
    let atlas = HieroAtlas::deserialize(atlas_bin).ok().unwrap();
    let page = atlas.try_unpack_page(0).ok().unwrap();

    let (vao, prog, texture, noise) = {
        let geo_data: Vec<f32> = vec![
            0.5, -0.5, 0., 1., -0.5, 0.5, 0., 1., -0.5, -0.5, 0., 1., 0.5, -0.5, 0., 1., 0.5, 0.5,
            0., 1., -0.5, 0.5, 0., 1.,
        ];
        let _color_data: Vec<f32> = vec![1., 0., 0., 1., 0., 1., 0., 1., 0., 0., 1., 1.];
        let uvs: Vec<f32> = vec![1., 1., 0., 0., 0., 1., 1., 1., 1., 0., 0., 0.];

        let shader_source =
            String::from_utf8(load_file!("./resources/phong.glsl").unwrap()).unwrap();

        let program = match OglProg::compile_program(&gl, shader_source.as_str()) {
            Ok(p) => p,
            Err(err) => match err {
                program::CompilationError::ShaderError {
                    ogl_error,
                    faulty_source,
                } => panic!("error:\n{}\nsource:\n{}\n", ogl_error, faulty_source),
                program::CompilationError::LinkError {
                    ogl_error,
                    faulty_source,
                } => panic!("error:\n{}\nsource:\n{}\n", ogl_error, faulty_source),
            },
        };

        let vao = OglArray::new(&gl).init(vec![
            BufferPair::new(
                "verts",
                OglBuf::new(&gl)
                    .with_target(glow::ARRAY_BUFFER)
                    .with_data(geo_data)
                    .with_index(1)
                    .with_num_comps(4)
                    .with_usage(glow::STATIC_DRAW)
                    .build()
                    .into(),
            ),
            BufferPair::new(
                "uv",
                OglBuf::new(&gl)
                    .with_target(glow::ARRAY_BUFFER)
                    .with_index(2)
                    .with_num_comps(2)
                    .with_usage(glow::STATIC_DRAW)
                    .with_data(uvs)
                    .build()
                    .into(),
            ),
        ]);

        //checkerboard texture for debugging purposes
        let width = 128;
        let height = 128;
        let mut checker_board: Vec<u8> = (0..3 * width * height).map(|_| 0).collect();
        for y in 0..height {
            for x in 0..width {
                let pixel_start = (y * width + x) * 3;
                let val = (((y as u8 / 16) % 2) ^ ((x as u8 / 16) % 2)) * 255;
                checker_board[pixel_start + 0] = val;
                checker_board[pixel_start + 1] = val;
                checker_board[pixel_start + 2] = val;
            }
        }

        let texture: Texture = unsafe {
            let to = gl.create_texture().unwrap();
            gl.bind_texture(TEXTURE_2D, Some(to));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                Some(&checker_board[..]),
            );
            gl.tex_parameter_i32(TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.tex_parameter_i32(TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            to
        };

        println!(
            "page width = {},page_height={}",
            page.info().width,
            page.info().height
        );

        let noise: OglTexture = TextureObj::<u8>::new(&gl)
            .with_format(glow::RGBA)
            .with_width(page.info().width)
            .with_height(page.info().height)
            .with_pixels_slice(&page.pixels()[..])
            .build()
            .into();

        // noise.copy_image(512,512,);
        (vao, program, texture, noise)
    };

    FlufflWindow::main_loop(
        window,
        MainState {
            dev_ptr: device.clone(),
            vao,
            progbox: prog,
            t: 0.,
            pos_x: 0.,
            pos_y: 0.,
            tex: texture,
            noise,
            writer: TextWriter::new(&gl).with_atlas(atlas).build(),
            client_socket: WsClient::new(ClientState::default())
                .with_on_message_cb(|_websocket, _state, message| {
                    match String::from_utf8(message.to_vec()) {
                        Ok(text) => console_log!("recieved: {}\n", text),
                        Err(_) => console_log!("unexpected input!\n"),
                    }
                })
                .with_on_close_cb(|_state| {
                    console_log!("socket has been closed!\n");
                })
                .with_on_error_cb(|_state| {
                    console_log!("An Error has occoured!\n");
                })
                .connect("ws://localhost:9001")
                .ok()
                .expect("Socket failed to connect, server may be offline")
                .into(),
        },
        core_loop,
    );

    Ok(())
}

pub async fn core_loop(
    win_ptr: FlufflWindowPtr,
    running: Rc<Cell<bool>>,
    main_state: Rc<RefCell<MainState>>,
) {
    let gl = win_ptr.window().gl();

    for event in win_ptr.window_mut().get_events().iter_mut() {
        let ms = &mut *main_state.borrow_mut();
        let device = &ms.dev_ptr;
        match event {
            EventKind::Quit => running.set(false),
            EventKind::Resize { width, height } => {
                console_log!("resized: [{}x{}]\n", width, height);
                unsafe {
                    gl.viewport(0, 0, width, height);
                }
            }
            EventKind::KeyDown { code } => {
                if let KeyCode::KEY_R = code {
                    device.modify_state(|state| {
                        state.map(|mp| {
                            mp.state = PlayState::RampUp(1000);
                            mp.music_src.seek_to_start();
                        });
                    })
                }
                if let KeyCode::PAGE_UP = code {
                    device.modify_state(|state| {
                        state.map(|mp| {
                            mp.volume = (mp.volume + 0.1).min(1.0).max(0.0);
                        });
                    })
                }
                if let KeyCode::PAGE_DOWN = code {
                    device.modify_state(|state| {
                        state.map(|mp| {
                            mp.volume = (mp.volume - 0.1).min(1.0).max(0.0);
                        });
                    })
                }
                if let KeyCode::SPACE = code {
                    device.modify_state(|state| {
                        state.map(|s| {
                            if let PlayState::Paused = s.state {
                                s.ticks = 0;
                                s.state = PlayState::RampUp(12000);
                            }
                        });
                    });
                    device.resume();
                }
                if let KeyCode::KEY_Y = code {
                    device.modify_state(|state| {
                        state.map(|s| {
                            if let PlayState::Playing = s.state {
                                s.ticks = 0;
                                s.state = PlayState::RampDown(12000);
                            }
                        });
                    });
                    device.resume();
                }
                if let KeyCode::KEY_B = code {
                    let message = "Hello Server!";
                    match ms.client_socket.send(message.as_bytes()) {
                        Err(_err) => {
                            console_log!("Woah! Send error!\n");
                        }
                        Ok(_) => (),
                    }
                }

                let code: i128 = code.into();
                if (code > KeyCode::KEY_A.into()) || (code < KeyCode::KEY_Z.into()) {
                    console_log!("char = {}\n", (code as u8 as char).to_lowercase());
                }
            }
            EventKind::MouseMove { x, y, dx, dy } => {
                console_log!("mouse move: [x:{},y:{},dx:{},dy:{}]\n", x, y, dx, dy);
                ms.pos_x = x as f32;
                ms.pos_y = y as f32;
            }
            EventKind::MouseUp { button_code, x, y } => {
                console_log!("mouse down at: [x:{},y:{}]\n", x, y);
                console_log!("{}\n", button_code);
            }
            EventKind::MouseWheel { button_code } => {
                console_log!("{}\n", button_code);
            }
            _ => (),
        }
    }

    unsafe {
        gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);

        main_state.borrow_mut().t += 0.01;
        let t = main_state.borrow().t;

        // let prog = main_state.borrow().progbox.prog();
        // gl.use_program(Some(prog));
        // let layer1_loc = gl.get_uniform_location(prog, "layer1");
        // let layer2_loc = gl.get_uniform_location(prog, "layer2");
        // let time_loc = gl.get_uniform_location(prog, "iTime");
        // main_state.borrow().vao.bind(true);

        // main_state
        //     .borrow()
        //     .noise
        //     .bind(TEXTURE1, layer2_loc.as_ref());

        // gl.uniform_1_f32(time_loc.as_ref(), t);
        // gl.draw_arrays(glow::TRIANGLES, 0, 6);
        // gl.use_program(None);

        let x = main_state.borrow().pos_x;
        let y = main_state.borrow().pos_y;
        let caption_list = ["Hello World!"];

        caption_list.iter().enumerate().for_each(|(k, caption)| {
            main_state.borrow_mut().writer.draw_text_line(
                caption,
                x,
                y + 64. * k as f32,
                (256. - 100.) * (t.sin() + 1.0) * 0.5 + 100.,
                Some((win_ptr.window().width(), win_ptr.window().height())),
            );
        });
    }

    main_state.borrow_mut().client_socket.listen();
}
