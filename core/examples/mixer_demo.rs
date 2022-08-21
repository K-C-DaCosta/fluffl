#![allow(unused_variables)]

use std::collections::{HashMap, HashSet};

use fluffl::{
    audio::{
        mixer::{
            protocol::{MixerRequest, MixerResponse, OffsetKind, TrackID},
            standard::MixerAudioDeviceContext,
            streams::{ExplicitWave, ImplicitWave, ScaleMode},
            HasAudioStream, Mixer, MixerProtocol, MutatedResult, SampleTime,
        },
        Interval,
    },
    console::*,
    //playing music files requires more than what the base library provides
    //so here is my implementation of certain things like "text rendering" and music playing
    extras::{hiero_pack::*, text_writer::*},
    gui::GuiManager,
    io::*,
    math::{WaveKind, FP64},
    prelude::*,
    // net::*,
    window::{event_util::*, *},
    *,
};

fn wave_sin<const FREQ: u32>(t: f64) -> f64 {
    use std::f64::consts::PI;
    ((FREQ as f64) * (2.0 * PI) * t).sin() * 0.2
}

pub struct MainState {
    // pub dev_ptr: ShortDeviceContext,
    pub mixer_device: MixerAudioDeviceContext,
    pub stream_queue: Vec<Option<Box<dyn HasAudioStream>>>,
    pub is_key_down: HashSet<KeyCode>,
    pub key_extend_list: Vec<(KeyCode, TrackID)>,
    pub key_frequency_table: HashMap<KeyCode, f32>,

    pub t: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub mixer_time: SampleTime,
    pub writer: TextWriter,
    pub init_route: bool,
    pub wave_type: WaveKind,
    pub gui_manager: GuiManager<FlufflState<MainState>>,
}

#[fluffl(Debug)]
pub async fn main() {
    math::waves::noise::init();

    //FlufflWindow is configured with XML, the format is self-explanitory
    let raw_bytes = load_file!("./wasm_bins/resources/config.xml").expect("config failed to load");
    let config_text = String::from_utf8(raw_bytes).expect("config file currupted");
    let window = FlufflWindow::init(config_text.as_str()).expect("failed to init window");
    let gl = window.gl();

    unsafe {
        window.gl().clear_color(0.1, 0.1, 0.1, 1.);
        window
            .gl()
            .viewport(0, 0, window.width() as i32, window.height() as i32);
    }

    let atlas_bytes = load_file!("./wasm_bins/resources/font.bcode").expect("file not found");
    let atlas = HieroAtlas::deserialize(atlas_bytes)
        .ok()
        .expect("font parse failed");

    let ctx = window.audio_context();

    let mixer_device = MixerAudioDeviceContext::new(ctx);
    mixer_device.resume();

    FlufflWindow::main_loop(
        window,
        MainState {
            gui_manager: GuiManager::new(gl.clone()),
            key_frequency_table: vec![
                (KeyCode::KEY_A, 262.0),
                (KeyCode::KEY_S, 294.0),
                (KeyCode::KEY_D, 330.0),
                (KeyCode::KEY_F, 349.0),
                (KeyCode::KEY_H, 10_000.0),
                (KeyCode::KEY_J, 100.0),
            ]
            .into_iter()
            .collect::<HashMap<_, _>>(),
            mixer_device,
            t: 0.,
            pos_x: 0.,
            pos_y: 0.,
            stream_queue: vec![],
            writer: TextWriter::new(&gl).with_atlas(atlas).build(),
            is_key_down: HashSet::new(),
            key_extend_list: Vec::new(),
            mixer_time: SampleTime::new(),
            init_route: false,
            wave_type: WaveKind::SQUARE,
        },
        main_loop,
    );
}

async fn main_loop(
    win_ptr: FlufflWindowPtr,
    running: FlufflRunning,
    main_state: FlufflState<MainState>,
) {
    let ms_clone = main_state.clone();

    let main_state = &mut *main_state.inner.borrow_mut();
    let mixer_device = &mut main_state.mixer_device;
    let writer = &mut main_state.writer;
    let is_key_down = &mut main_state.is_key_down;
    let key_extend_list = &mut main_state.key_extend_list;
    let init_route = &mut main_state.init_route;
    let wave_type = &mut main_state.wave_type;
    let key_frequency_table = &mut main_state.key_frequency_table;
    let gui_manager = &mut main_state.gui_manager;

    let gl = win_ptr.window().gl();

    gui_manager.init_state(ms_clone);

    mixer_device.send_request(MixerRequest::FetchMixerTime);

    main_state.t += 0.01;
    let t = main_state.t;
    let x = main_state.pos_x;
    let y = main_state.pos_y;

    // if *init_route == false {
    //     let file_pointer_to_music = std::fs::File::open("./wasm_bins/resources/fuck_jannies.adhoc")
    //         .expect("file galed to load");
    //     let parsed_music_file = adhoc_audio::AdhocCodec::load(file_pointer_to_music)
    //         .expect("failed to read music file");
    //     let id = mixer_device.gen_id();
    //     mixer_device.send_request(MixerRequest::AddTrack(
    //         id,
    //         OffsetKind::current(),
    //         Box::new(ExplicitWave::new(parsed_music_file, ScaleMode::Repeat)),
    //     ));
    //     *init_route = true;
    // }

    //draw seek time
    let max_seek_time_ms = 300_000.0;
    let seek_time = (x / 500.0) * max_seek_time_ms;

    for event in win_ptr.window_mut().get_events().flush_iter_mut() {
        gui_manager.push_event(event);
        match event {
            EventKind::Quit => running.set(false),
            EventKind::Resize { width, height } => {
                console_log!("resized: [{}x{}]\n", width, height);
                unsafe {
                    gl.viewport(0, 0, width, height);
                }
            }
            EventKind::KeyDown { code } => {
                let code_char = code.key_val().unwrap_or_default();

                if let '0'..='9' = code_char {
                    let offset = code_char as u8 as usize - b'0' as usize;
                    let new_type = WaveKind::from(offset);
                    *wave_type = new_type;
                }

                if let KeyCode::KEY_E = code {
                    let file_pointer_to_music =
                        std::fs::File::open("./wasm_bins/resources/taunt.adhoc")
                            .expect("file galed to load");
                    let parsed_music_file = adhoc_audio::AdhocCodec::load(file_pointer_to_music)
                        .expect("failed to read music file");
                    let id = mixer_device.gen_id();
                    mixer_device.send_request(MixerRequest::AddTrack(
                        id,
                        OffsetKind::current(),
                        Box::new(ExplicitWave::new(parsed_music_file, ScaleMode::Repeat)),
                    ));
                }
                if let KeyCode::KEY_R = code {
                    let file_pointer_to_music =
                        std::fs::File::open("./wasm_bins/resources/hiphop.adhoc")
                            .expect("file galed to load");
                    let parsed_music_file = adhoc_audio::AdhocCodec::load(file_pointer_to_music)
                        .expect("failed to read music file");

                    let track_id = mixer_device.gen_id();

                    mixer_device.send_request(MixerRequest::AddTrack(
                        track_id,
                        OffsetKind::current(),
                        Box::new(ExplicitWave::new(parsed_music_file, ScaleMode::Repeat)),
                    ));

                    mixer_device.send_request(MixerRequest::MutateMixer(track_id, |tid, mixer| {
                        let interval = mixer.track_get_interval(tid)?;
                        let new_interval_that_loops_10_times =
                            Interval::from_point_and_length(interval.lo, interval.distance() * 10);
                        mixer.track_set_interval(tid, new_interval_that_loops_10_times)?;
                        Ok(())
                    }));
                }
                if let KeyCode::KEY_T = code {
                    let file_pointer_to_music =
                        std::fs::File::open("./wasm_bins/resources/fuck_jannies.adhoc")
                            .expect("file galed to load");
                    let parsed_music_file = adhoc_audio::AdhocCodec::load(file_pointer_to_music)
                        .expect("failed to read music file");
                    let id = mixer_device.gen_id();
                    mixer_device.send_request(MixerRequest::AddTrack(
                        id,
                        OffsetKind::current(),
                        Box::new(ExplicitWave::new(parsed_music_file, ScaleMode::Repeat)),
                    ));
                }

                //handle keyboard notes
                if is_key_down.contains(&code) == false {
                    if let Some(&wave_frequency) = key_frequency_table.get(&code) {
                        let wave_frequency = wave_frequency as f64;
                        let id = mixer_device.gen_id();

                        mixer_device.send_request(MixerRequest::AddTrack(
                            id,
                            OffsetKind::current(),
                            Box::new(ImplicitWave::new(
                                wave_type.as_fn(),
                                Interval::from_length(FP64::from(1000)),
                                wave_frequency,
                            )),
                        ));

                        key_extend_list.push((code, id));
                    }
                }

                if let KeyCode::KEY_V = code {
                    mixer_device
                        .send_request(MixerRequest::MutateMixer(TrackID::null(), |_, mixer| {
                            mixer.print_tree()
                        }));
                }

                if let KeyCode::PAGE_UP = code {}
                if let KeyCode::PAGE_DOWN = code {}
                if let KeyCode::SPACE = code {
                    mixer_device.resume();
                }
                if let KeyCode::KEY_Y = code {
                    mixer_device.dump_recording();
                }

                //insert towards the end

                is_key_down.insert(code);

                console_log!("char = {}\n", code.key_val().unwrap());
            }
            EventKind::KeyUp { code } => {
                is_key_down.remove(&code);

                let if_code_is_in_extend_list = key_extend_list
                    .iter()
                    .enumerate()
                    .find(|(k, &(c, _))| c == code);

                if let Some((key_idx, _)) = if_code_is_in_extend_list {
                    key_extend_list.remove(key_idx);
                }
            }
            EventKind::MouseMove { x, y, .. } => {
                // console_log!("mouse move: [x:{},y:{},dx:{},dy:{}]\n", x, y, dx, dy);
                main_state.pos_x = x as f32;
                main_state.pos_y = y as f32;
            }
            EventKind::MouseDown {
                button_code: MouseCode::LEFT_BUTTON,
                ..
            } => {
                // console_log!("mouse down at: [x:{},y:{}]\n", x, y);
                // console_log!("{}\n", button_code);

                println!("seeked at:{}", time_to_string(seek_time as i64));
                mixer_device.send_request(MixerRequest::Seek(OffsetKind::Start {
                    offset: seek_time as u64,
                }));
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

    //extend key  here
    // for &(c, id) in &key_extend_list[..] {
    //     if is_key_down.contains(&c) {
    //         mixer_device.send_request(MixerRequest::MutateMixer(id, |tid, mixer| {
    //             let mut interval = mixer.track_get_interval(tid)?;
    //             //extend the hi part of the interval by 16 ms (because this callback is assumed to be called every 16 ms)
    //             interval.hi = interval.hi + 16;
    //             mixer.track_set_interval(tid, interval)
    //         }));
    //     }
    // }

    unsafe {
        gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
    }

    let (win_width, win_height) = win_ptr.window().get_bounds_f32();
    let speed_t = ((FP64::from(x) - 0) / 200).clamp(FP64::from(0), FP64::from(1));
    let speed = speed_t * 3;

    //draw text here
    let caption_list = [format!(
        "{time:}x[{speed:.2}]",
        time = time_to_string(main_state.mixer_time.elapsed_in_ms_fp().as_i64()),
        speed = speed.as_f64()
    )];
    caption_list.iter().enumerate().for_each(|(k, caption)| {
        // let size = (256. - 100.) * (t.sin() + 1.0) * 0.5 + 100.;
        let size = 100.0;
        writer.draw_text_line(
            caption,
            0.,
            0. + 64. * k as f32,
            size,
            Some(win_ptr.window().get_bounds()),
        );
    });

    writer.draw_text_line(
        &time_to_string(seek_time as i64),
        x + 10.0,
        y,
        32.0,
        Some(win_ptr.window().get_bounds()),
    );

    gui_manager.render(win_width, win_height);

    // mixer_device.modify_state(|state| {
    //     let mixer_state = state?;
    //     mixer_state.set_mixer_speed(speed).ok()?;
    //     Some(())
    // });

    let responses_iter = mixer_device.recieve_responses();
    for resp in responses_iter {
        match resp {
            MixerResponse::MixerTime(t) => {
                main_state.mixer_time = t;
            }
            _ => (),
        }
    }
}

fn time_to_string(elapsed_time_ms: i64) -> String {
    let total_seconds = elapsed_time_ms / 1000;
    let total_minutes = total_seconds / 60;
    let total_hours = total_minutes / 60;
    if total_seconds < 60 {
        format!("{}s", total_seconds % 60)
    } else if total_seconds < 3600 {
        format!("{}m:{}s", total_minutes % 60, total_seconds % 60)
    } else {
        format!(
            "{}h:{}m:{}s",
            total_hours,
            total_minutes % 60,
            total_seconds % 60
        )
    }
}
