use glue_core::audio::{music_player::*, *};
use glue_core::console::*;
use glue_core::io::*;
use glue_core::window_util::{event_util::*, *};
use glue_core::*;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct MainState<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send,
    S: Send,
{
    pub dev_ptr: GlueAudioDeviceContext<F, S>,
}

pub fn glue_main() -> Result<(), GlueError> {
    let wav = {
        let wav_data: Vec<u8> = load_file!("./resources/sound_test.wav")?;
        wav::WavFile::new().with_data(wav_data).parse()?
    };

    //GlueWindow is configured with XML, the format is self-explanitory
    let raw_bytes = load_file!("./resources/config.xml")?;
    let config_text = String::from_utf8(raw_bytes)?;
    let window = GlueWindow::init(config_text.as_str())?;

    unsafe {
        window.gl().clear_color(1., 0.5, 0., 1.);
        window.gl().viewport(0, 0, 800, 600);
    }

    let device: GlueAudioDeviceContext<_, _> = GlueAudioDeviceCore::new()
        .with_specs(GlueDesiredSpecs {
            sample_rate: wav.header().map(|hdr| hdr.sample_rate),
            channels: Some(2),
            buffer_size: None,
        })
        .with_state(MusicPlayer {
            ticks: 0,
            state: PlayState::Paused,
            volume: 1.0,
            music_src: wav.samples().into(),
            channels: 2,
        })
        .with_callback(music_player_callback)
        .into_with(window.audio_context());

    window.main_loop(
        MainState {
            dev_ptr: device.clone(),
        },
        core_loop,
    );

    Ok(())
}

pub async fn core_loop<F>(
    win_ptr: GlueWindowPtr,
    running: Rc<Cell<bool>>,
    main_state: Rc<RefCell<MainState<F, MusicPlayer>>>,
) where
    F: FnMut(&mut MusicPlayer, &mut [f32]) + Copy + Send + 'static,
{
    let gl = win_ptr.window().gl();
    for event in win_ptr.window_mut().get_events().iter_mut() {
        let ms = &*main_state.borrow();
        let device = &ms.dev_ptr;
        match event {
            EventKind::Quit => running.set(false),
            EventKind::KeyDown { code } => {
                if let KeyCode::KEY_R = code {
                    device.modify_state(|state| {
                        state.map(|mp| {
                            mp.state = PlayState::RampUp(12000);
                            mp.ticks = 0;
                            mp.music_src.sample_index = 0;
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
                let code: i128 = code.into();
                if (code > KeyCode::KEY_A.into()) || (code < KeyCode::KEY_Z.into()) {
                    console_log!("char = {}\n", (code as u8 as char).to_lowercase());
                }
            }
            EventKind::MouseMove { x, y, dx, dy } => {
                console_log!("mouse move: [x:{},y:{},dx:{},dy:{}]\n", x, y, dx, dy);
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
        gl.clear(COLOR_BUFFER_BIT);
    }
}
