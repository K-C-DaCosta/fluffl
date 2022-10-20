#![allow(unused_variables, unused_imports)]

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use fluffl::{
    audio::{
        mixer::{
            protocol::{MixerEventKind, MixerRequest, MixerResponse, OffsetKind, TrackID},
            standard::MixerAudioDeviceContext,
            streams::{ExplicitWave, ImplicitWave, ScaleMode},
            HasAudioStream, Mixer, MixerProtocol, MutatedResult, SampleTime,
        },
        Interval,
    },
    collections::fixed_stack::FixedStack,
    console::*,
    //playing music files requires more than what the base library provides
    //so here is my implementation of certain things like "text rendering" and music playing
    extras::hiero_pack::*,
    gui::*,
    io::*,
    math::{Vec2, Vec4, WaveKind, FP32, FP64},
    prelude::*,
    text_writer::{self, HasTextWriterBuilder, TextWriter, UROOB},
    window::{event_util::*, *},
    *,
};

struct PathSegment {
    a: Vec2<f32>,
    b: Vec2<f32>,
    t_lo: f32,
    t_hi: f32,
}

impl PathSegment {
    pub fn new(a: Vec2<f32>, b: Vec2<f32>) -> Self {
        Self {
            a,
            b,
            t_lo: 0.0,
            t_hi: 0.0,
        }
    }

    pub fn with_time(mut self, lo: f32, hi: f32) -> Self {
        self.t_hi = hi;
        self.t_lo = lo;
        self
    }

    pub fn eval(&self, global_time: f32) -> Vec2<f32> {
        let local_time = global_time - self.t_lo;
        let duration = self.t_hi - self.t_lo;
        let when_gt_zero = (0.0 - local_time).to_bits() as i32 >> 31;
        let when_lt_durtation = (local_time - duration) as i32 >> 31;
        let mask =
            f32::from_bits((1.0f32.to_bits() as i32 & (when_gt_zero & when_lt_durtation)) as u32);
        let t = local_time / duration;
        ((self.a * (1.0 - t)) + (self.b * t)) * mask
    }
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
    pub wave_type: WaveKind,
    pub gui_manager: GuiManager<FlufflState<MainState>>,
    pub mutated_text: String,
    pub tracks_to_delete_table: HashSet<TrackID>,
    pub angle: FP32,
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
    let atlas = HieroAtlas::deserialize(crate::decoders::base64::decode(UROOB).unwrap())
        .ok()
        .expect("font parse failed");

    let ctx = window.audio_context();

    let mixer_device = MixerAudioDeviceContext::new(ctx);
    mixer_device.resume();

    let app_state = MainState {
        gui_manager: setup_test_gui(GuiManager::new(gl.clone())),
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
        wave_type: WaveKind::Square,
        mutated_text: String::new(),
        tracks_to_delete_table: HashSet::new(),
        angle: FP32::from_bits(20000),
    };

    FlufflWindow::run(window, app_state, main_loop);
}

async fn main_loop(
    win_ptr: FlufflWindowPtr,
    running: FlufflRunning,
    main_state: FlufflState<MainState>,
) {
    //these are all intended to be executed sequentially so lots of .await below
    process_events(&win_ptr, running, &main_state).await;
    draw_scene(&win_ptr, running, &main_state).await;
    handle_mixer_responses(&win_ptr, running, &main_state).await;
    execute_gui_mutation_requests(&win_ptr, running, &main_state).await;
}

async fn process_events(
    win_ptr: &FlufflWindowPtr,
    mut running: FlufflRunning,
    main_state: &FlufflState<MainState>,
) {
    let main_state = &mut *main_state.borrow_mut();
    //split-borrow main_state
    let mixer_device = &mut main_state.mixer_device;
    let writer = &mut main_state.writer;
    let is_key_down = &mut main_state.is_key_down;
    let key_extend_list = &mut main_state.key_extend_list;
    let wave_type = &mut main_state.wave_type;
    let key_frequency_table = &mut main_state.key_frequency_table;
    let gui_manager = &mut main_state.gui_manager;

    let gl = win_ptr.window().gl();

    main_state.t += 0.01;
    let t = main_state.t;
    let x = main_state.pos_x;
    let y = main_state.pos_y;

    //draw seek time
    let max_seek_time_ms = 300_000.0;
    let seek_time = (x / 500.0) * max_seek_time_ms;

    for event in win_ptr.window_mut().get_events().flush_iter_mut() {
        //forward event to gui_manager
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

                // if let KeyCode::KEY_E = code {
                //     let file_pointer_to_music =
                //         std::fs::File::open("./wasm_bins/resources/taunt.adhoc")
                //             .expect("file galed to load");
                //     let parsed_music_file = adhoc_audio::AdhocCodec::load(file_pointer_to_music)
                //         .expect("failed to read music file");
                //     let id = mixer_device.gen_id();
                //     mixer_device.send_request(MixerRequest::AddTrack(
                //         id,
                //         OffsetKind::current(),
                //         Box::new(ExplicitWave::new(parsed_music_file, ScaleMode::Repeat)),
                //     ));
                // }
                // if let KeyCode::KEY_R = code {
                //     let file_pointer_to_music =
                //         std::fs::File::open("./wasm_bins/resources/hiphop.adhoc")
                //             .expect("file galed to load");
                //     let parsed_music_file = adhoc_audio::AdhocCodec::load(file_pointer_to_music)
                //         .expect("failed to read music file");

                //     let track_id = mixer_device.gen_id();

                //     mixer_device.send_request(MixerRequest::AddTrack(
                //         track_id,
                //         OffsetKind::current(),
                //         Box::new(ExplicitWave::new(parsed_music_file, ScaleMode::Repeat)),
                //     ));

                //     mixer_device.send_request(MixerRequest::MutateMixer(track_id, |tid, mixer| {
                //         let interval = mixer.track_get_interval(tid)?;
                //         let new_interval_that_loops_10_times =
                //             Interval::from_point_and_length(interval.lo, interval.distance() * 10);
                //         mixer.track_set_interval(tid, new_interval_that_loops_10_times)?;
                //         Ok(())
                //     }));
                // }
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
                if !is_key_down.contains(&code) {
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

                if let KeyCode::PAGE_U = code {}
                if let KeyCode::PAGE_D = code {}
                if let KeyCode::SPACE = code {
                    unsafe {
                        gl.clear(glow::COLOR_BUFFER_BIT);
                    }
                    // mixer_device.resume();
                }
                if let KeyCode::KEY_Y = code {
                    mixer_device.dump_recording();
                }

                //insert towards the end
                if !is_key_down.contains(&code) {
                    console_log!("char = {}\n", code.key_val().unwrap());
                    is_key_down.insert(code);
                }
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
}

async fn draw_scene(
    win_ptr: &FlufflWindowPtr,
    running: FlufflRunning,
    main_state: &FlufflState<MainState>,
) {
    let main_state = &mut *main_state.borrow_mut();
    let window_bounds = win_ptr.window_mut().get_bounds();

    //split-borrow main_state
    let mixer_device = &mut main_state.mixer_device;
    let writer = &mut main_state.writer;
    let is_key_down = &mut main_state.is_key_down;
    let key_extend_list = &mut main_state.key_extend_list;
    let wave_type = &mut main_state.wave_type;
    let key_frequency_table = &mut main_state.key_frequency_table;
    let gui_manager = &mut main_state.gui_manager;
    let temp_text = &mut main_state.mutated_text;

    let gl = win_ptr.window().gl();

    let t = main_state.t;
    let x = main_state.pos_x;
    let y = main_state.pos_y;

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
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::LEQUAL);
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

    // caption_list.iter().enumerate().for_each(|(k, caption)| {
    //     // let size = (256. - 100.) * (t.sin() + 1.0) * 0.5 + 100.;
    //     let size = 100.0;
    //     writer.draw_text_line_preserved(
    //         caption,
    //         0.,
    //         0. + 64. * k as f32,
    //         size,
    //         Some(win_ptr.window().get_bounds()),
    //     );
    // });

    let t = main_state.angle.as_f32().max(1.0).min(7980.9);
    let segments = [
        PathSegment::new(
            Vec2::from_array([100., 400.]),
            Vec2::from_array([600., 400.]),
        )
        .with_time(0., 2000.),
        PathSegment::new(
            Vec2::from_array([600., 400.]),
            Vec2::from_array([100., 100.]),
        )
        .with_time(2000., 4000.),
        PathSegment::new(
            Vec2::from_array([100., 100.]),
            Vec2::from_array([100., 400.]),
        )
        .with_time(4000., 8000.),
    ];
    let final_point: Vec2<f32> = segments.iter().map(|seg| seg.eval(t)).sum();

    gui_manager.render(writer, win_width, win_height);

    writer.draw_text_line(
        ".",
        final_point.x(),
        final_point.y(),
        32.0,
        Some(win_ptr.window().get_bounds()),
    );

    writer.draw_text_line(
        format!("angle = {} full ={}", t, main_state.angle.data).as_str(),
        0.,
        0.,
        32.0,
        Some(win_ptr.window().get_bounds()),
    );

    // writer.draw_text_line(
    //     &time_to_string(seek_time as i64),
    //     x + 10.0,
    //     y,
    //     32.0,
    //     Some(win_ptr.window().get_bounds()),
    // );

    // mixer_device.modify_state(|state| {
    //     let mixer_state = state?;
    //     mixer_state.set_mixer_speed(speed).ok()?;
    //     Some(())
    // });

    writer.draw_text_line(temp_text, 0.0, 200.0, 32.0, Some(window_bounds));
}

async fn handle_mixer_responses(
    win_ptr: &FlufflWindowPtr,
    running: FlufflRunning,
    main_state: &FlufflState<MainState>,
) {
    let ms_clone = main_state.clone();
    let main_state = &mut *main_state.borrow_mut();
    let window_bounds = win_ptr.window_mut().get_bounds();

    //split-borrow main_state
    let mixer_device = &mut main_state.mixer_device;
    let writer = &mut main_state.writer;
    let is_key_down = &mut main_state.is_key_down;
    let key_extend_list = &mut main_state.key_extend_list;
    let wave_type = &mut main_state.wave_type;
    let key_frequency_table = &mut main_state.key_frequency_table;
    let gui_manager = &mut main_state.gui_manager;
    let tracks_to_delete_table = &mut main_state.tracks_to_delete_table;

    mixer_device.send_request(MixerRequest::FetchMixerTime);
    let responses_iter = mixer_device.recieve_responses();

    let mut tracks_to_delete = FixedStack::<32, TrackID>::new();

    for resp in responses_iter {
        match resp {
            MixerResponse::MixerTime(t) => {
                main_state.mixer_time = t;
            }
            MixerResponse::MixerEvent(MixerEventKind::TrackStopped(tid)) => {
                if tracks_to_delete_table.contains(&tid) {
                    tracks_to_delete.push(tid);
                    tracks_to_delete_table.remove(&tid);
                }
            }
            _ => (),
        }
    }
    while let Some(tid) = tracks_to_delete.pop() {
        mixer_device.send_request(MixerRequest::RemoveTrack(tid));
    }
}

async fn execute_gui_mutation_requests(
    win_ptr: &FlufflWindowPtr,
    running: FlufflRunning,
    main_state: &FlufflState<MainState>,
) {
    GuiManager::execute_mutation_requests(main_state, |state| {
        state.borrow_mut().gui_manager.poll_mutation_requsts()
    })
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

fn setup_test_gui(
    mut manager: GuiManager<FlufflState<MainState>>,
) -> GuiManager<FlufflState<MainState>> {
    let origin = manager.add_component(GuiComponentKey::default(), Box::new(OriginState::new()));

    let pink_frame = manager
        .builder_frame()
        .with_parent(origin)
        .with_bounds([400.0 + 0.0, 200.0 + 100.0])
        .with_roundness([1., 1., 10.0, 10.0])
        .with_position([64.0, 32.0])
        .with_edge_thickness(0.05)
        .with_scrollbars(true)
        // .with_drag(true)
        .build();

    let alt_frame = manager
        .builder_frame()
        .with_parent(origin)
        .with_bounds([200.0, 100.0])
        .with_roundness([0.0, 0.0, 10.0, 10.])
        .with_position([64.0, 400.0])
        .with_drag(true)
        .build();

    let red_frame = manager
        .builder_frame()
        .with_parent(pink_frame)
        .with_bounds([400., 45.])
        .with_color([0.7, 0.2, 0., 1.0])
        .with_position([0.0, -45.0])
        .with_roundness([15.0, 15.0, 0.0, 0.0])
        .with_flags(component_flags::TITLEBAR | component_flags::OVERFLOWABLE)
        .with_drag_highest(true)
        .with_edge_thickness(0.00)
        .build();

    let red_child = manager
        .builder_frame()
        .with_parent(red_frame)
        .with_bounds([32., 32.])
        .with_roundness([16.0, 16.0, 0.0, 0.0])
        .with_color(Vec4::rgb_u32(0x277BC0))
        .with_position([2.0, 6.0])
        .with_drag(true)
        .build();

    let orange_frame = manager
        .builder_frame()
        .with_parent(pink_frame)
        .with_bounds([256., 128.])
        .with_color(Vec4::rgb_u32(0xFF7F3F))
        .with_roundness(Vec4::from([1., 1., 30., 30.]))
        .with_edge_color([0., 0., 0., 1.0])
        .with_position([128.0, 64.0])
        .with_drag(true)
        .with_visibility(false)
        .build();

    let slider_frame = manager
        .builder_slider()
        .with_parent(pink_frame)
        .with_position([4.0, 64.0])
        .with_bounds([400.0, 32.0])
        .with_color(Vec4::rgb_u32(0x554994))
        .with_edge_color(Vec4::rgb_u32(0xFFCCB3))
        .with_roundness([8.0; 4])
        .with_drag(false)
        .with_listener(GuiEventKind::OnFocusIn, |state, _, _| {
            state.slider_frame.edge_color = Vec4::rgb_u32(0xff0000);
        })
        .with_listener(GuiEventKind::OnFocusOut, |state, _, _| {
            state.slider_frame.edge_color = Vec4::rgb_u32(0xFFCCB3);
        })
        .with_button_bounds([32.0, 120.0])
        .with_button_color(Vec4::rgb_u32(0x332255))
        .with_button_edge_color(Vec4::rgb_u32(0xF29393))
        .with_button_roundness([8.0; 4])
        .with_button_listener(GuiEventKind::OnHoverIn, |f, _, _| {
            f.color *= 9. / 10.;
        })
        .with_button_listener(GuiEventKind::OnHoverOut, |f, _, _| {
            f.color *= 10. / 9.;
        })
        .with_button_listener(GuiEventKind::OnMouseDown, |f, _, _| {
            f.color = Vec4::from([1.0; 4]) - f.color;
        })
        .with_button_listener(GuiEventKind::OnMouseRelease, |f, _, _| {
            f.color = Vec4::from([1.0; 4]) - f.color;
        })
        .with_button_listener_advanced(GuiEventKind::OnDrag, |info| {
            let slider_button_key = info.key;
            let gui_comp_tree = info.gui_comp_tree;
            let slider_frame_key = gui_comp_tree
                .get_parent_id(slider_button_key)
                .expect("slider button should have parent");

            let percentage = gui_comp_tree
                .get(slider_frame_key)
                .and_then(|comp| comp.as_any().downcast_ref::<SliderState>())
                .map(|slider_frame| slider_frame.percentage)
                .unwrap_or_default();

            let disp = info.event.disp();

            let new_angle = FP32::from_bits(((i32::MAX) as f64 * percentage as f64) as i32);

            let can_beep = (percentage * 10000.0).fract() < 0.001
                && percentage > 0.05
                && percentage < 0.95
                && disp.y().abs() < 0.001;
            if can_beep {
                info.mutation_queue.enqueue(Box::new(move |state| {
                    let state = &mut *state.borrow_mut();
                    let mixer_device = &mut state.mixer_device;
                    let removal_table = &mut state.tracks_to_delete_table;
                    let new_track_id = mixer_device.gen_id();
                    state.angle = new_angle;

                    //mark track for deletion when it stops playing
                    removal_table.insert(new_track_id);
                    mixer_device.send_request(MixerRequest::AddTrack(
                        new_track_id,
                        OffsetKind::current(),
                        Box::new(ImplicitWave::new(
                            WaveKind::SawTooth.as_fn(),
                            Interval::from_length(FP64::from(64)),
                            1200.0,
                        )),
                    ))
                }));
            } else {
                // let t = FP32::from(32767.0 * percentage);
                // let a = FP32::from(-20000);
                // let b = FP32::from(12768);
                // let s = t.div_exact(FP32::from(32767));
                // let mix = (b - a) * s + a;
                // println!("mix = {mix}");
                info.mutation_queue.enqueue(Box::new(move |state| {
                    let state = &mut *state.borrow_mut();
                    state.angle = new_angle;
                }));
            }
        })
        .build();

    for k in 0..20 {
        let row = k / 7;
        let col = k % 7;
        let color = Vec4::<f32>::rgb_u32(0x277BC0);
        let _blue_button = manager
            .builder_frame()
            .with_name(format!("{}", (k as u8 + b'a') as char))
            .with_parent(orange_frame)
            .with_bounds([32., 32.])
            .with_color(color)
            .with_roundness(Vec4::from([1., 1., 1., 1.]))
            .with_edge_color([0., 0., 0., 1.0])
            .with_position([7.0 + 35.0 * (col as f32), 5.0 + 33.0 * (row as f32)])
            .with_listener(GuiEventKind::OnHoverIn, |frame, _state, _| {
                frame.color *= 0.5;
                frame.color[3] = 1.0;
            })
            .with_listener(GuiEventKind::OnHoverOut, |frame, _state, _| {
                frame.color *= 2.0;
                frame.color[3] = 1.0;
            })
            .with_listener(GuiEventKind::OnMouseDown, |frame, _, mrq| {
                frame.color = Vec4::rgb_u32(!0);
                let name = frame.name().to_string();
                if name == "a" {
                    mrq.enqueue(Box::new(|state| {
                        state.borrow_mut().mutated_text.clear();
                    }));
                } else {
                    mrq.enqueue(Box::new(move |state| {
                        state.borrow_mut().mutated_text.push_str(name.as_str());
                    }));
                }
            })
            .with_listener(GuiEventKind::OnMouseRelease, move |frame, _, _| {
                frame.color = color * 0.5;
                frame.color[3] = 1.0;
            })
            .with_drag(false)
            .build();
    }

    let textbox_key = manager
        .builder_textbox()
        .with_parent(pink_frame)
        .with_bounds([1000.0, 64.0])
        .with_position([4.0, 200.0 - 64.0])
        .with_color(Vec4::rgb_u32(0))
        .with_roundness([0.0, 0.0, 32.0, 32.0])
        .with_font_size(32.0)
        .with_alignment([TextAlignment::Left, TextAlignment::Center])
        .with_listener(GuiEventKind::OnFocusIn, |comp, _, _| {
            comp.frame.edge_color = Vec4::rgb_u32(0xff0000);
        })
        .with_listener(GuiEventKind::OnFocusOut, |comp, _, _| {
            comp.frame.edge_color = Vec4::rgb_u32(0x89CFFD);
        })
        .build();

    let _title_label = manager
        .builder_label()
        .with_parent(red_frame)
        .with_bounds([400.0, 45.0])
        .with_position([0.0, 0.0])
        .with_caption("its fucking over :-(")
        .build();

    // println!("origin={}", origin);
    // println!("pink_frame={}", prink_frame);
    // println!("orange_frame={}", orange_frame);
    // println!("blue_button={}", blue_button);
    // println!("slider_frame={}", slider_frame);
    // println!("slider_button={}", slider_button);
    // manager.gui_component_tree.print_by_ids();
    // let parent = manager.gui_component_tree.get_parent_id(NodeID(4)).unwrap();
    // println!("parent of 4 is = {:?}", parent);
    manager
}
