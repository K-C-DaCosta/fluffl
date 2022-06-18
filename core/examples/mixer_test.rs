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
    prelude::*,
    // net::*,
    window::{event_util::*, glow::*, *},
    *,
};
use std::f64::consts::PI;

fn wave_sin<const FREQ: u32>(t: f64) -> f64 {
    0.5 * ((FREQ as f64) * (2.0 * PI) * t).sin()
}

pub struct MixerState {
    pub mixer: Mixer,
    pub channels: u32,
    pub frequency: u32,
    pub t: f64,
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
    // for samp_idx in 0..output.len()/2 {
    //     let time_in_seconds = state.t/state.frequency as f64;
    //     let fx = wave_sin::<39>(time_in_seconds) as f32;
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
            t: 0.0,
            test_stream: Box::new(ImplicitWave::new(
                wave_sin::<440>,
                Interval::from((0, 5_000)),
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
                        if let KeyCode::KEY_A = code {
                            device.modify_state(|state_opt| {
                                let state = state_opt.unwrap();
                                let time = state.get_time();
                                let lo = time.elapsed_in_ms_u128();
                                state.add_track(Box::new(ImplicitWave::new(
                                    wave_sin::<262>,
                                    Interval::from((lo, lo+2000)),
                                    sample_rate,
                                )));
                            })
                        }
                        if let KeyCode::KEY_S = code {
                            device.modify_state(|state_opt| {
                                let state = state_opt.unwrap();
                                let time = state.get_time();
                                let lo = time.elapsed_in_ms_u128();
                                state.add_track(Box::new(ImplicitWave::new(
                                    wave_sin::<294>,
                                    Interval::from((lo, lo+1000)),
                                    sample_rate,
                                )));
                            })
                        }
                        if let KeyCode::KEY_D = code {
                            device.modify_state(|state_opt| {
                                let state = state_opt.unwrap();
                                let time = state.get_time();
                                let lo = time.elapsed_in_ms_u128();
                                state.add_track(Box::new(ImplicitWave::new(
                                    wave_sin::<330>,
                                    Interval::from((lo, lo+1000)),
                                    sample_rate,
                                )));
                            })
                        }
                        if let KeyCode::KEY_F = code {
                            device.modify_state(|state_opt| {
                                let state = state_opt.unwrap();
                                let time = state.get_time();
                                let lo = time.elapsed_in_ms_u128();
                                state.add_track(Box::new(ImplicitWave::new(
                                    wave_sin::<349>,
                                    Interval::from((lo+1, lo+1000)),
                                    sample_rate,
                                )));
                            })
                        }


                        if let KeyCode::PAGE_UP = code {
                            device.modify_state(|state_opt| {})
                        }
                        if let KeyCode::PAGE_DOWN = code {
                            device.modify_state(|state_opt| {})
                        }
                        if let KeyCode::SPACE = code {
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
