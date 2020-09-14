use glue_core::audio::*;
use glue_core::io::*;
use glue_core::window_util::{event_util::*, *};
use glue_core::*;

use std::cell::{Cell, RefCell};
use std::fs::File;
use std::rc::Rc;

use std::fs;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{BufRead, BufReader, Read};
use std::slice::*;

#[derive(Clone, Copy)]
enum PlayState {
    RampUp(usize),
    RampDown(usize),
    Playing,
    Paused,
}

struct MusicPlayer {
    ticks: usize,
    state: PlayState,
    volume: f32,
    music_src: AudioBuffer<f32>,
    channels: usize,
}
/// This callback assumes that samples are interleaved and works for two channels ONLY
fn music_player_callback(mp: &mut MusicPlayer, out: &mut [f32]) {
    if let PlayState::Paused = mp.state {
        out.iter_mut().for_each(|e| *e = 0.);
        return;
    }
    let num_channels = mp.channels;
    let samples = out.len();
    let mut input_samples = Vec::new();
    input_samples.resize(samples / num_channels, audio::AudioSample::from([0f32; 2]));
   

    let vol = mp.volume;

    mp.state = match mp.state {
        PlayState::RampUp(max_ticks) => {
            if mp.ticks > max_ticks {
                PlayState::Playing
            } else {
                mp.state
            }
        }
        PlayState::RampDown(max_ticks) => {
            if mp.ticks > max_ticks {
                PlayState::Paused
            } else {
                mp.state
            }
        }
        _ => mp.state,
    };

    let samples_read = mp.music_src.read(&mut input_samples[..]);
    let inv_out_len = 1.0 / (out.len() as f32);
    let play_state = mp.state; 

    if samples_read == 0 {
        return;
    }

    for k in (0..out.len()).step_by(num_channels) {
        let j = (k * samples_read) as f32 * inv_out_len;
        let t = j.fract();

        let j0 = j as usize;
        let j1 = (j0 + 1).min((samples_read - 1).max(0));

        let samp0 = input_samples[j0];
        let samp1 = input_samples[j1];
        

        let exec = |samp0: AudioSample<_>, samp1: AudioSample<_>, channel_index| {
            let f0 = samp0.channel[channel_index];
            let f1 = samp1.channel[channel_index];
            //do some linear interpolation here
            let lerp = (f1 - f0) * t + f0;

            match play_state {
                PlayState::RampUp(max_ticks) => {
                    let t = (mp.ticks as f32 / max_ticks as f32).min(1.0).max(0.0);
                    lerp * vol * (t * t)
                }
                PlayState::RampDown(max_ticks) => {
                    let t = (mp.ticks as f32 / max_ticks as f32).min(1.0).max(0.0);
                    let linear_down = 1. - t;
                    lerp * vol * linear_down * linear_down
                }
                PlayState::Paused => 0.0,
                _ => lerp * vol,
            }
        };
        //write 'samples' into the output buffer
        //In this callback samples are assumed to be INTERLEAVED , not planar. 
        for j in 0..num_channels{
            out[k+j] = exec(samp0, samp1,1-j);
        }

        mp.ticks += 1;
    }
}
pub struct MainState<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send,
    S: Send,
{
    pub dev_ptr: GlueAudioDeviceContext<F, S>,
}

fn main() -> Result<(), GlueError> {
    // glue_core::io::load_file_cb("resources/out.wav", move |wav_fetch|{
    //     glue_core::io::load_file_cb("./resources/config.xml", move |xml_fetch|{
    //         if let (Ok(wav_data),Ok(xml_data)) = (wav_fetch.as_ref(),xml_fetch.as_ref()){

    //         }
    //     });
    // });

    let wav = {
        let wav_data: Vec<u8> = fs::read("./resources/sound_test.wav").unwrap();
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

async fn core_loop<F>(
    win_ptr: GlueWindowPtr,
    running: Rc<Cell<bool>>,
    main_state: Rc<RefCell<MainState<F, MusicPlayer>>>,
) where
    F: FnMut(&mut MusicPlayer, &mut [f32]) + std::marker::Copy + Send,
{
    let gl = win_ptr.window().gl();
    for event in win_ptr.window_mut().get_events().iter_mut() {
        let ms = &*main_state.borrow();
        let device = &ms.dev_ptr;
        match event {
            EventKind::Quit => running.set(false),
            EventKind::KeyDown { code } => {
                if let KeyCode::KEY_R = code {
                    device.modify_state(|state|{
                        state.map(|mp|{
                            mp.state = PlayState::RampUp(12000);
                            mp.ticks = 0; 
                            mp.music_src.sample_index = 0; 
                        });
                    })
                }
                if let KeyCode::PAGE_UP = code {
                    device.modify_state(|state|{
                        state.map(|mp|{
                            mp.volume = (mp.volume+0.1).min(1.0).max(0.0);
                        });
                    })
                }
                if let KeyCode::PAGE_DOWN = code {
                    device.modify_state(|state|{
                        state.map(|mp|{
                            mp.volume = (mp.volume-0.1).min(1.0).max(0.0);
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
