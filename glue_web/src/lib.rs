#![allow(warnings)]
use futures::executor;
use futures::future::TryFutureExt;

use glue_core::console::*; 
use glue_core::io;
use glue_core::*;
use glue_core::audio::music_player::*;
use glue_core::audio::wav;
use glue_core::audio::*;
use glue_core::io::*;
use glue_core::window_util::{event_util::*, *};


use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use std::io::prelude::*;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::slice;

use std::collections::VecDeque;

use console_error_panic_hook;
use std::panic;



#[wasm_bindgen(start)]
pub fn glue_entry_point() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    spawn_local(async move {
        glue_main().await;
    });
}



pub struct MainState<F, S> {
    device: GlueAudioDeviceContext<F, S>,
}

pub async fn glue_main() -> Result<(), GlueError> {
    let config_text = "
        <window>
            <width>800</width>
            <height>600</height>
            <title>my_app</title>
        </window>";
    let window = GlueWindow::init(config_text).unwrap();

    let wav = {
        let wav_data: Vec<u8> = load_file!("./resources/sound_test.wav")?;
        wav::WavFile::new().with_data(wav_data).parse()?
    };

    let music_device: GlueAudioDeviceContext<_, _> = GlueAudioDeviceCore::new()
        .with_specs(GlueDesiredSpecs {
            sample_rate: wav.header().map(|hdr| hdr.sample_rate),
            channels: Some(2),
            buffer_size: None,
        })
        .with_state(MusicPlayer {
            ticks: 0,
            state: PlayState::RampUp(48000),
            volume: 1.0,
            music_src: wav.samples().into(),
            channels: 2,
        })
        .with_callback(music_player_callback)
        .into_with(window.audio_context());

    unsafe {
        window.gl().clear_color(1.0, 0.5, 0.0, 1.0);
        window.gl().viewport(0, 0, 512, 512);
    }

    window.main_loop(
        MainState {
            device: music_device.clone(),
        },
        core_loop,
    );
    Ok(())
}

async fn core_loop<F>(
    win_ptr: GlueWindowPtr,
    running: Rc<Cell<bool>>,
    ms: Rc<RefCell<MainState<F, MusicPlayer>>>,
) where
    F: FnMut(&mut MusicPlayer, &mut [f32]) + Copy + Send + 'static,
{
    let audio_ctx = win_ptr.window().audio_context().clone();

    for event in win_ptr.window_mut().get_events().iter_mut() {
        match event {
            EventKind::Quit => (),
            EventKind::KeyDown { code } => {
                let device = ms.borrow().device.clone();

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

                let ncode: i128 = code.into();
                if (ncode > KeyCode::KEY_A.into()) || (ncode < KeyCode::KEY_Z.into()) {
                    console_log!("char = {}\n", (ncode as u8 as char).to_lowercase());
                }
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
    // write_line(format!("tick = {}",unsafe{counter}).as_str());
    unsafe {
        win_ptr.window().gl().clear(COLOR_BUFFER_BIT);
    }
}



// pub struct TinyAudioSample {
//     pub channel: [u8; 2],
// }
// static mut CALLBACK: Option<js_sys::Function> = None;


// pub fn run_music(audio_out: &Arc<RefCell<GlueAudioContext>>, fetch: Result<Vec<u8>, GlueError>) {
//     let ctx_out = audio_out.clone();

//     let ctx = ctx_out.clone();
//     let temp_ctx = ctx.clone();

//     let mut pcm_samples: Vec<u8> = Vec::new();
//     let mut samples: Vec<f32> = Vec::new();
//     let max_samples = 1024;
//     let pump_list: Rc<RefCell<VecDeque<js_sys::Function>>> = Rc::new(RefCell::new(VecDeque::new()));

//     if let Ok(data) = fetch {
//         let mut play_time = ctx.borrow().ctx.current_time();

//         let data = Rc::new(RefCell::new(data));

//         let header = &data.borrow()[0..45];
//         //I keep an explicit pointer that I update evertime I read the music data
//         let mut byte_index = 0;

//         let temp_data = data.clone();
//         let process_raw_pcm = move || {
//             samples.resize(max_samples, 0.0);
//             pcm_samples.resize(max_samples * 2, 0);

//             let mut raw_pcm = &temp_data.borrow()[byte_index..];
//             // console_log!("play time = {}", play_time);

//             while play_time - ctx.borrow().ctx.current_time() < 1. {
//                 let buffer = ctx
//                     .borrow()
//                     .ctx
//                     .create_buffer(1, max_samples as u32, 22000.0f32)
//                     .unwrap();
//                 let mut samples = buffer.get_channel_data(0).unwrap();

//                 raw_pcm.read(&mut pcm_samples).map(|bytes_read| {
//                     let samples_read = bytes_read >> 1;
//                     byte_index += bytes_read;
//                     let pcm_array: &[TinyAudioSample] = unsafe {
//                         slice::from_raw_parts(
//                             pcm_samples.as_ptr() as *mut TinyAudioSample,
//                             max_samples,
//                         )
//                     };
//                     for i in 0..max_samples {
//                         unsafe {
//                             *samples.get_unchecked_mut(i) =
//                                 pcm_array.get_unchecked(i).channel[0] as f32 / 255.0;
//                         }
//                     }
//                 });

//                 buffer.copy_to_channel(&mut samples[..], 0).unwrap();

//                 let bsn = ctx.borrow().ctx.create_buffer_source().unwrap();
//                 bsn.set_buffer(Some(&buffer));

//                 //when this buffer finished playing, continue buffering
//                 let tpl = pump_list.clone();
//                 let continue_buffering = move || {
//                     let cb = unsafe { CALLBACK.as_ref() };
//                     let f = cb.unwrap();
//                     let pump_list = tpl;
//                     f.call0(&JsValue::null());
//                     // because we know 'onended' had to have fired  we can remove the oldest buffer.
//                     // The popped buffer is likely to be garbage collected by the JS interpreter
//                     pump_list.borrow_mut().pop_front();
//                     // console_log!(
//                     //     "callback triggered, pl size = {}",
//                     //     pump_list.borrow().len()
//                     // );
//                 };

//                 //wrap continue_buffering in boxed closure and convert to a Js Function
//                 let cb = Closure::once_into_js(continue_buffering)
//                     .dyn_into::<js_sys::Function>()
//                     .unwrap();

//                 pump_list.borrow_mut().push_back(cb);
//                 bsn.set_onended(pump_list.borrow().back());

//                 bsn.start_with_when(play_time).unwrap();

//                 play_time += max_samples as f64 / 22000.0;

//                 let node: AudioNode = bsn.dyn_into::<AudioNode>().unwrap();
//                 node.connect_with_audio_node(&ctx.borrow().ctx.destination().dyn_into().unwrap());
//                 // console_log!("play time = {}", ctx.borrow().ctx.current_time() );
//             }
//         };

//         let process_raw_pcm_closure = Closure::wrap(Box::new(process_raw_pcm) as Box<dyn FnMut()>)
//             .into_js_value()
//             .dyn_into::<js_sys::Function>()
//             .unwrap();

//         unsafe {
//             //soud callback to global variable
//             CALLBACK = Some(process_raw_pcm_closure);
//             //execute sound thingy
//             CALLBACK.as_ref().unwrap().call0(&JsValue::null());
//         }
//     }
// }
