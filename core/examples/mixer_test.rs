use fluffl::{
    audio::{
        mixer::{streams::ImplicitWave, HasAudioStream, Mixer},
        Interval, PCMSlice, *,
    },
    console::*,
    //playing music files requires more than what the base library provides
    //so here is my implementation of certain things like "text rendering" and music playing
    extras::{hiero_pack::*, text_writer::*},
    io::*,
    math::FixedPoint,
    prelude::*,
    // net::*,
    window::{event_util::*, glow::*, *},
    *,
};
use std::collections::VecDeque;

fn wave_sin<const FREQ: u32>(t: f64) -> f64 {
    use std::f64::consts::PI;
    ((FREQ as f64) * (2.0 * PI) * t).sin() * 0.2
}

fn sinf32(t: f32) -> f32 {
    (2.0 * 3.14159 * t).sin() * 0.2
}

pub struct SoundWave {
    frequency: f32,
    wave: fn(f32) -> f32,
    interval: (f32, f32),
    attack_time: f32,
    release_time: f32,
}

impl SoundWave {
    pub fn new(wave: fn(f32) -> f32, frequency: f32, interval: (f32, f32)) -> Self {
        let smoothing_length = (interval.0 - interval.1).abs();
        Self {
            frequency,
            wave,
            interval,
            attack_time: smoothing_length * 0.05,
            release_time: smoothing_length * 0.02,
        }
    }

    /// `time` is in seconds
    pub fn evaluate(&self, time: f32) -> f32 {
        //divide everything by 1000 to make sure were in seconds
        let to_seconds = 1.0 / 1000.0;

        let interval_lo = self.interval.0 * to_seconds;
        let interval_hi = self.interval.1 * to_seconds;
        let attack_dt = self.attack_time * to_seconds;
        let release_dt = self.release_time * to_seconds;

        let frequency = self.frequency;
        let wave = self.wave;

        let linear_t = |x: f32, e0: f32, e1: f32| -> f32 { ((x - e0) / (e1 - e0)).clamp(0.0, 1.0) };
        let attack_t = linear_t(time, interval_lo, interval_lo + attack_dt);
        let release_t = linear_t(time, interval_lo - release_dt, interval_hi);
        let attack_coef = 1.0 - (1.0 - attack_t).powf(2.0);
        let release_coef = 1.0 - (release_t * release_t);

        attack_coef * wave(frequency * time) * release_coef
    }
}

pub struct MixerState {
    pub mixer: Mixer,
    pub channels: u32,
    pub frequency: u32,
    pub t: f64,
    pub amplitude: f32,
    pub sound_waves: VecDeque<SoundWave>,
    pub test_stream: Box<dyn HasAudioStream>,
}
impl std::ops::Deref for MixerState {
    type Target = Mixer;
    fn deref(&self) -> &Self::Target {
        &self.mixer
    }
}
impl std::ops::DerefMut for MixerState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mixer
    }
}

fn mixer_state_cb(state: &mut MixerState, output: &mut [f32]) {
    // Naive way of adding multiple implicit waves together (mixing but only for implicit waves )
    // upsides:
    // - easy to implement
    // downsides:
    // - practical only for implicit waves where you can compute f(t) easily, computing f(t) for sampled audio requires seeking (can be slow)
    // - can't add sampled audio like mp3's or whatever
    // - expensive, spatial datastructure can shorten innter loop by O(log(number_of_waves)) rather than  O(number_of_waves)
    // ------------------------------------------------------------------------------------------------------------------------
    
    // let conversion_factor_sec = 1.0 / state.frequency as f64;
    // let conversion_factor_ms = 1000.0 * conversion_factor_sec;

    // for samp_idx in 0..output.len() / 2 {
    //     let mut dst = 0.0;

    //     let time_in_ms = (state.t * conversion_factor_ms) as f32;
    //     let time_in_seconds = state.t * conversion_factor_sec;

    //     let is_in_bounds = |wave: &&SoundWave| {
    //         time_in_ms > (wave.interval.0 + -1.0) && time_in_ms < (wave.interval.1 + 1.0)
    //     };
    //     let mut count = 0; 
    //     for wave in state.sound_waves.iter().filter(is_in_bounds) {
    //         let old = dst;
    //         let new = wave.evaluate(time_in_seconds as f32);
    //         dst = old + new;
    //         count+=1;
    //     }
    //     // println!("count = {}",count);

    //     output[2 * samp_idx + 0] = dst;
    //     output[2 * samp_idx + 1] = dst;
    //     state.t += 1.0;
    // }

    // for samp_idx in 0..output.len()/2 {
    //     let time_in_seconds = state.t/state.frequency as f64;
    //     let fx = wave_sin::<441>(time_in_seconds) as f32;
    //     output[2*samp_idx + 0] = fx;
    //     output[2*samp_idx + 1] = fx;
    //     state.t+=1.0;
    // }

    state.mixer.mix_audio(PCMSlice::new(
        output,
        state.frequency as u32,
        state.channels as u32,
    ))

    // //set everything to zero
    // output.iter_mut().for_each(|e| *e = 0.0);
    // //write sine wave into output
    // state.test_stream.pull_samples(PCMSlice::new(
    //     output,
    //     state.frequency as u32,
    //     state.channels as u32,
    // ));
}
type ShortState = MixerState;
type ShortDeviceCB = fn(&mut ShortState, &mut [f32]);
type ShortDeviceContext = FlufflAudioDeviceContext<ShortDeviceCB, ShortState>;

pub struct MainState {
    pub dev_ptr: ShortDeviceContext,
    pub stream_queue: Vec<Box<dyn HasAudioStream>>,
    pub t: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub writer: TextWriter,
}

#[fluffl(Debug)]
pub async fn main() {
    //GlueWindow is configured with XML, the format is self-explanitory
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
    let sample_rate = 44_100;
    let channels = 2;
    let buffer_size = 2048;
    // setting up a device core doesn't actually do anything (no system calls)
    // think of it like filling out a form.
    let device_core: AudioDeviceCore<ShortDeviceCB, MixerState> = AudioDeviceCore::new()
        .with_specs(DesiredSpecs {
            sample_rate: Some(sample_rate),
            channels: Some(channels),
            buffer_size: Some(buffer_size),
        })
        .with_state(MixerState {
            mixer: Mixer::new(sample_rate, channels),
            channels,
            frequency: sample_rate,
            amplitude: 1.0,
            t: 0.0,
            sound_waves: vec![SoundWave::new(sinf32, 440.0, (0.0, 1000.0))]
                .into_iter()
                .collect::<VecDeque<_>>(),
            test_stream: Box::new(ImplicitWave::new(
                wave_sin::<440>,
                Interval::from((0, 1_000i32)),
                sample_rate,
            )),
        })
        .with_callback(mixer_state_cb);

    // Creating a device context is where things really start to happen (new threads and memory are allocated for processing audio)
    let device = FlufflAudioDeviceContext::new(device_core, window.audio_context());

    let atlas_bytes = load_file!("./wasm_bins/resources/font.bcode").expect("file not found");
    let atlas = HieroAtlas::deserialize(atlas_bytes)
        .ok()
        .expect("font parse failed");

    FlufflWindow::main_loop(
        window,
        MainState {
            dev_ptr: device,
            t: 0.,
            pos_x: 0.,
            pos_y: 0.,
            stream_queue: vec![],
            writer: TextWriter::new(&gl).with_atlas(atlas).build(),
        },
        move |win_ptr, running, main_state| async move {
            let gl = win_ptr.window().gl();

            for event in win_ptr.window_mut().get_events().flush_iter_mut() {
                let ms = &mut *main_state.inner.borrow_mut();
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
                        if let KeyCode::KEY_G = code {
                            device.modify_state(|state_opt| {
                                let state: &mut MixerState = state_opt.unwrap();
                                let time = state.get_time();
                                let lo = time.elapsed_in_ms_fp() + FixedPoint::from(1);
                                let interval =
                                    Interval::from_point_and_length(lo, FixedPoint::from(10_000));

                                state.add_track(Box::new(ImplicitWave::new(
                                    wave_sin::<39>,
                                    interval,
                                    sample_rate,
                                )));
                            })
                        }
                        if let KeyCode::KEY_A = code {
                            device.modify_state(|state_opt| {
                                let state = state_opt.unwrap();
                                let time = state.get_time();
                                let lo = time.elapsed_in_ms_fp() + FixedPoint::from(1);
                                let interval =
                                    Interval::from_point_and_length(lo, FixedPoint::from(1000));

                                state.add_track(Box::new(ImplicitWave::new(
                                    wave_sin::<262>,
                                    interval,
                                    sample_rate,
                                )));

                                let time = ((state.t / state.frequency as f64) * 1000.0) as f32;
                                state.sound_waves.push_back(SoundWave::new(
                                    sinf32,
                                    440.0,
                                    (time, time + 1000.0),
                                ));
                                // if state.sound_waves.len() > 10 {
                                //     state.sound_waves.pop_front();
                                // }
                            })
                        }
                        if let KeyCode::KEY_S = code {
                            device.modify_state(|state_opt| {
                                let state = state_opt.unwrap();
                                let time = state.get_time();
                                let lo = time.elapsed_in_ms_fp() + FixedPoint::from(1);
                                let interval =
                                    Interval::from_point_and_length(lo, FixedPoint::from(1000));

                                state.add_track(Box::new(ImplicitWave::new(
                                    wave_sin::<294>,
                                    interval,
                                    sample_rate,
                                )));

                                let time = ((state.t / state.frequency as f64) * 1000.0) as f32;
                                state.sound_waves.push_back(SoundWave::new(
                                    sinf32,
                                    294.,
                                    (time, time + 1000.0),
                                ));
                                // if state.sound_waves.len() > 10 {
                                //     state.sound_waves.pop_front();
                                // }
                            })
                        }
                        if let KeyCode::KEY_D = code {
                            device.modify_state(|state_opt| {
                                let state = state_opt.unwrap();
                                let time = state.get_time();
                                let lo = time.elapsed_in_ms_u64();
                                state.add_track(Box::new(ImplicitWave::new(
                                    wave_sin::<330>,
                                    Interval::from((lo, lo + 1000)),
                                    sample_rate,
                                )));

                                let time = ((state.t / state.frequency as f64) * 1000.0) as f32;
                                state.sound_waves.push_back(SoundWave::new(
                                    sinf32,
                                    330.0,
                                    (time, time + 1000.0),
                                ));
                                // if state.sound_waves.len() > 10 {
                                //     state.sound_waves.pop_front();
                                // }
                            });
                        }
                        if let KeyCode::KEY_F = code {
                            device.modify_state(|state_opt| {
                                let state = state_opt.unwrap();
                                let time = state.get_time();
                                let lo = time.elapsed_in_ms_fp();
                                state.add_track(Box::new(ImplicitWave::new(
                                    wave_sin::<349>,
                                    Interval::from_point_and_length(lo, FixedPoint::from(1000)),
                                    sample_rate,
                                )));

                                let time = ((state.t / state.frequency as f64) * 1000.0) as f32;
                                state.sound_waves.push_back(SoundWave::new(
                                    sinf32,
                                    349.0,
                                    (time, time + 1000.0),
                                ));

                                // if state.sound_waves.len() > 10 {
                                //     state.sound_waves.pop_front();
                                // }
                            })
                        }

                        if let KeyCode::PAGE_UP = code {
                            device.modify_state(|state_opt| {})
                        }
                        if let KeyCode::PAGE_DOWN = code {
                            device.modify_state(|state_opt| {})
                        }
                        if let KeyCode::SPACE = code {
                            device.modify_state(|state_opt| {
                                let state = state_opt.unwrap();
                                let time = state.get_time().elapsed_in_ms_fp();

                                let add_track = |state: &mut MixerState, time| {
                                    state.add_track(Box::new(ImplicitWave::new(
                                        wave_sin::<349>,
                                        Interval::from_point_and_length(
                                            time,
                                            FixedPoint::from(2000),
                                        ),
                                        sample_rate,
                                    )));
                                };

                                state.t = 0.0;
                                // add_track(state, time+ 0);
                                // add_track(state, time+100);
                                // add_track(state, time+200);
                                // add_track(state, time+300);
                            });
                            device.resume();
                        }
                        if let KeyCode::KEY_Y = code {}
                        console_log!("char = {}\n", code.key_val().unwrap());
                    }
                    EventKind::MouseMove { x, y, .. } => {
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

            main_state.inner.borrow_mut().t += 0.01;
            let t = main_state.inner.borrow().t;
            let x = main_state.inner.borrow().pos_x;
            let y = main_state.inner.borrow().pos_y;

            //draw text here
            let caption_list = ["fluffl"];
            caption_list.iter().enumerate().for_each(|(k, caption)| {
                main_state.inner.borrow_mut().writer.draw_text_line(
                    caption,
                    x,
                    y + 64. * k as f32,
                    (256. - 100.) * (t.sin() + 1.0) * 0.5 + 100.,
                    Some(win_ptr.window().get_bounds()),
                );
            });
        },
    );
}
