use fluffl::{
    audio::*,
    console::*,
    io::*,
    // net::*,
    window::{event_util::*, glow::*, *},
    *,
    //playing music requires more than what the base library provides 
    //so here is my implementation of certain things like "text rendering" and music playing
    extras::{
        audio::{AudioBuffer}, 
        audio::music_player::*,
        audio::{ogg::*, *},
        hiero_pack::*,
        text_writer::*,
    }
};



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
    pub t: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub writer: TextWriter,
}

pub async fn fluffl_main() -> Result<(), FlufflError> {
    let ogg = {
        //load_file!("../../wasm_bins/resources/HipHopNoir_1.ogg")
        let file_bytes: Vec<u8> = load_file!("../../wasm_bins/resources/HipHopNoir_1.ogg").expect("ogg failed to load");
        ogg::OggFile::new()
            .with_data(file_bytes)
            .parse()
            .expect("parse failed")
    };

    //GlueWindow is configured with XML, the format is self-explanitory
    let raw_bytes = load_file!("../../wasm_bins/resources/config.xml").expect("config failed to load");
    let config_text = String::from_utf8(raw_bytes)?;
    let window = FlufflWindow::init(config_text.as_str())?;
    let gl = window.gl();

    unsafe {
        window.gl().clear_color(0.1, 0.1, 0.1, 1.);
        window
            .gl()
            .viewport(0, 0, window.width() as i32, window.height() as i32);
    }

    // setting up a device core doesn't actually do anything (no system calls)
    // think of it like filling out a form.
    let device_core: AudioDeviceCore<ShortDeviceCB, ShortMusicPlayer> = AudioDeviceCore::new()
        .with_specs(DesiredSpecs {
            sample_rate: ogg.sample_rate().map(|a| a as u32),
            channels: Some(2),
            buffer_size: None,
        })
        .with_state(MusicPlayer {
            ticks: 0,
            state: PlayState::Paused, //
            volume: 0.5,
            music_src: ogg.into(),
            repeat_track:true,
        })
        .with_callback(music_callback);

    // Creating a device context is where things really start to happen (new threads and memory are allocated for processing audio)
    let device = FlufflAudioDeviceContext::new(device_core, window.audio_context());

    let atlas_bytes = load_file!("../../wasm_bins/resources/font.bcode").expect("file not found");
    let atlas = HieroAtlas::deserialize(atlas_bytes).ok().expect("font parse failed");

    FlufflWindow::main_loop(
        window,
        MainState {
            dev_ptr: device,
            t: 0.,
            pos_x: 0.,
            pos_y: 0.,
            writer: TextWriter::new(&gl).with_atlas(atlas).build(),
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
                    device.modify_state(|state_opt| {
                        state_opt.map(|mp| {
                            mp.state = PlayState::RampUp(1000);
                            mp.music_src.seek_to_start();
                        });
                    })
                }
                if let KeyCode::PAGE_UP = code {
                    device.modify_state(|state_opt| {
                        state_opt.map(|mp| {
                            mp.volume = (mp.volume + 0.1).min(1.0).max(0.0);
                        });
                    })
                }
                if let KeyCode::PAGE_DOWN = code {
                    device.modify_state(|state_opt| {
                        state_opt.map(|mp| {
                            mp.volume = (mp.volume - 0.1).min(1.0).max(0.0);
                        });
                    })
                }
                if let KeyCode::SPACE = code {
                    device.modify_state(|state_opt| {
                        state_opt.map(|s| {
                            if let PlayState::Paused = s.state {
                                s.ticks = 0;
                                s.state = PlayState::RampUp(32000);
                            }
                        });
                    });
                    device.resume();
                }
                if let KeyCode::KEY_Y = code {
                    device.modify_state(|state_opt| {
                        state_opt.map(|s| {
                            if let PlayState::Playing = s.state {
                                s.ticks = 0;
                                s.state = PlayState::Paused;
                            }
                        });
                    });
                    device.resume();
                }
                console_log!("char = {}\n",code.key_val().unwrap());
            }
            EventKind::MouseMove { x, y, dx, dy } => {
                // console_log!("mouse move: [x:{},y:{},dx:{},dy:{}]\n", x, y, dx, dy);
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
    }

    main_state.borrow_mut().t += 0.01;
    let t = main_state.borrow().t;
    let x = main_state.borrow().pos_x;
    let y = main_state.borrow().pos_y;

    //draw text here 
    let caption_list = ["fluffl"];

    caption_list.iter().enumerate().for_each(|(k, caption)| {
        main_state.borrow_mut().writer.draw_text_line(
            caption,
            x,
            y + 64. * k as f32,
            (256. - 100.) * (t.sin() + 1.0) * 0.5 + 100.,
            Some((win_ptr.window().width(), win_ptr.window().height())),
        );
    });

    // main_state.borrow_mut().client_socket.listen();
}
