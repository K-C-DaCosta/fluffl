use super::{AudioDeviceCore, ConcreteSpecs};
use crate::{console::*, *};
use std::sync::{Arc, Mutex};

use std::{
    thread,
    time::{Duration, Instant},
};

use alsa::{
    pcm::{Access, Format, HwParams, State, PCM},
    Output, ValueOr,
};

#[derive(Clone, Default)]
pub struct FlufflAudioContext {
    /*
    This struct is not really needed for ALSA
    but for SDL2 and WEBAUDIO a struct like this
    is needed
    */
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum DeviceState {
    Playing,
    Paused,
}

pub struct FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send + 'static,
    State: Send + 'static,
{
    fluffl_audio_device: Arc<Mutex<FlufflAudioDevice<Callback, State>>>,
    pcm: Arc<Mutex<alsa::pcm::PCM>>,
    state: Arc<Mutex<DeviceState>>,
}

impl<Callback, State> Clone for FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send,
    State: Send,
{
    fn clone(&self) -> Self {
        Self {
            fluffl_audio_device: self.fluffl_audio_device.clone(),
            pcm: self.pcm.clone(),
            state: self.state.clone(),
        }
    }
}

impl<Callback, State> FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send,
    State: Send,
{
    /// creates a platform-agnostic FlufflAudioDevice
    pub fn new(core: AudioDeviceCore<Callback, State>, _actx: FlufflAudioContext) -> Self {
        let audio_device = Arc::new(Mutex::new(FlufflAudioDevice { core }));
        //select the default audio device
        let pcm = alsa::pcm::PCM::new("default", alsa::Direction::Playback, true)
            .expect("alsa: default device failed");
        Self {
            fluffl_audio_device: audio_device,
            pcm: Arc::new(Mutex::new(pcm)),
            state: Arc::new(Mutex::new(DeviceState::Paused)),
        }
    }

    /// ## Description
    /// Allows the user to modify state through a callback
    /// ### Comments
    /// If I can't easily return the value to code higher up in the stack,
    /// the next best thing is pass a callback to the value  
    pub fn modify_state<ModifyCallback>(&self, mut cb: ModifyCallback)
    where
        ModifyCallback: FnMut(Option<&mut State>) -> Option<()>,
    {
        let mut lck = self.fluffl_audio_device.lock().unwrap();
        let device = &mut *lck;
        let s = device.core.state.as_mut();
        let _ = cb(s);
    }
    /// resumes the device
    pub fn resume(&self) {
        let ctx = self.clone();
        let audio_device = self.fluffl_audio_device.clone();

        if *ctx.state.lock().unwrap() == DeviceState::Playing{
            println!("already playing!");
            return; 
        }


        //mark state as "playing"
        *ctx.state.lock().unwrap() = DeviceState::Playing;


        //spawn a dedicated thread to pump PCM to ALSA
        thread::spawn(move || {
            let ConcreteSpecs {
                sample_rate,
                channels,
                buffer_size,
            } = audio_device
                .lock()
                .unwrap()
                .core
                .desired_specs
                .make_concrete();

            //get the pcm struct ptr then drop the guard
            let pcm_ptr = {&*ctx.pcm.lock().unwrap()} as *const _;

            let pcm = unsafe{&*pcm_ptr};
            let hwp = alsa::pcm::HwParams::any(pcm).expect("hw params failed");
            hwp.set_channels(channels as u32)
                .expect("set_channels(..) failed");
            hwp.set_rate(sample_rate as u32, ValueOr::Nearest)
                .expect("set_format(..) failed");
            hwp.set_format(Format::float())
                .expect("set_format(..) failed");
            hwp.set_access(Access::RWInterleaved)
                .expect("set_access(..) failed");

            pcm.hw_params(&hwp).unwrap();
            let hwp = pcm.hw_params_current().unwrap();
            let swp = pcm.sw_params_current().unwrap();
            swp.set_start_threshold(hwp.get_buffer_size().unwrap())
                .unwrap();
            pcm.sw_params(&swp).unwrap();

            console_log!(
                "PCM status: {:?}, {:?}",
                pcm.state(),
                pcm.hw_params_current().unwrap()
            );

            let mut outp = Output::buffer_open().unwrap();
            pcm.dump(&mut outp).unwrap();
            console_log!("== PCM dump ==\n{}", outp);

            let mut buffer = vec![0.0f32; channels * buffer_size];
            let audio_device = audio_device;
            let mut frames_written = 0u128;

            const NANOS_IN_ONE_SEC: u128 = 1_000_000_000;
            let frequency = sample_rate as u128;
            let buffer_size = buffer_size as u128;

            let calc_elasped_time_nanos = |frames, frequency|  { 
                // NOTE:
                // frames_written[frames](1/freq)[secs/frames](10^9ns/secs) = time_in_nanos   
                (frames * NANOS_IN_ONE_SEC) / frequency
            };

            // its okay to lock and hold this throughout the entire duration of the threads lifespan
            // because no other threads should be fighting for it
            let io = pcm.io_f32().unwrap();
            let real_time = Instant::now();

            if pcm.state() != alsa::pcm::State::Running {
                pcm.start().unwrap();
            }

            /// buffer 500 milliseconds ahead
            const BUFFER_DELTA_IN_NANOS:u128 = NANOS_IN_ONE_SEC/2;


            loop {
                // check if state changed then break
                if let Ok(DeviceState::Paused) = ctx.state.try_lock().map(|a| *a) {
                    break;
                }

      
                if let Ok(mut dev) = audio_device.try_lock() {
                    let mut callback = dev.core.callback();
                    let state = dev.state().expect("state not initalized");
                    
                    let written_time_nanos = calc_elasped_time_nanos(frames_written, frequency);
                    let real_time_nanos = real_time.elapsed().as_nanos();

                   
                    if written_time_nanos > (  BUFFER_DELTA_IN_NANOS + real_time_nanos) {
                        //if the buffer is ahead by BUFFER_DELTA do nothing (busy-wait)
                        continue;
                    }

                    // call user-defined callback
                    callback(state, &mut buffer[..]);

                    // send ALL samples retrieved from state to ALSA
                    let mut frames_pending = buffer_size as isize;
                    while let Ok(frames_written) = io.writei(&buffer[..]) {
                        frames_pending -= frames_written as isize;
                        if frames_pending <= 0 {
                            break;
                        }
                    }

                    //update frames written
                    frames_written += buffer_size;
                }
            }
            pcm.drain().unwrap();
        });
    }

    /// pauses the device
    pub fn pause(&self) {
        // this simply just signals the thread to stop playing and clean up after itself
        *self.state.lock().unwrap() = DeviceState::Paused;
    }
}

pub struct FlufflAudioDevice<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send + 'static,
    S: Send,
{
    core: AudioDeviceCore<F, S>,
}

impl<F, S> FlufflAudioDevice<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send + 'static,
    S: Send + 'static,
{
    pub fn callback(&self) -> F {
        self.core.callback()
    }

    pub fn state(&mut self) -> Option<&mut S> {
        self.core.state.as_mut()
    }
}

pub struct FlufflCallback<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send + 'static,
    State: Send,
{
    audio_device: Arc<Mutex<FlufflAudioDevice<Callback, State>>>,
}
