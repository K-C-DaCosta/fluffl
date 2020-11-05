use super::{AudioDeviceCore};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

/// The platform specific  Audio context for the desktop
/// Here I use sdl2 as the backend
pub struct FlufflAudioContext {
    pub audio_ss: sdl2::AudioSubsystem,
}

/// A pointer to SDL2's audio device (which is uses some multithreading) 
/// This is where we can actually start playing music(on desktop )
pub struct FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + std::marker::Copy + Send + 'static,
    State: Send + 'static,
{
    fluffl_audio_device: Arc<Mutex<RefCell<FlufflAudioDevice<Callback, State>>>>,
    sdl2_device: Arc<sdl2::audio::AudioDevice<FlufflCallback<Callback, State>>>,
}

impl<Callback, State> Clone for FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + std::marker::Copy + Send,
    State: Send,
{
    fn clone(&self) -> Self {
        Self {
            fluffl_audio_device: self.fluffl_audio_device.clone(),
            sdl2_device: self.sdl2_device.clone(),
        }
    }
}

impl<Callback, State> FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send,
    State: Send,
{
    /// creates a platform-agnostic FlufflAudioDefice
    pub fn new(
        core: AudioDeviceCore<Callback, State>,
        audio_context: Arc<RefCell<FlufflAudioContext>>,
    ) -> FlufflAudioDeviceContext<Callback, State> {
        println!("new music context");
        let desired_spec = sdl2::audio::AudioSpecDesired {
            freq: core.desired_specs.sample_rate.clone().map(|a| {
                println!("freq = {}", a);
                a as i32
            }),
            channels: core.desired_specs.channels.clone().map(|a| {
                println!("channels = {}", a);
                a as u8
            }),
            samples: core.desired_specs.buffer_size.clone().map(|a| {
                println!("buffer_size = {}", a);
                a as u16
            }),
        };

        let audio_device = Arc::new(Mutex::new(RefCell::new(FlufflAudioDevice { core })));

        let glue_callback = FlufflCallback {
            audio_device: audio_device.clone(),
        };

        let sdl2_device = audio_context
            .borrow()
            .audio_ss
            .open_playback(None, &desired_spec, |_spec| {
                // initialize the audio callback
                glue_callback
            })
            .unwrap();

        Self {
            fluffl_audio_device: audio_device,
            sdl2_device: Arc::new(sdl2_device),
        }
    }
    /// Allows the user to modify state through a callback
    pub fn modify_state<ModifyCallback>(&self, mut cb: ModifyCallback)
    where
        ModifyCallback: FnMut(Option<&mut State>),
    {
        let lck = self.fluffl_audio_device.lock().unwrap();
        let device = &mut *lck.borrow_mut();
        let s = device.core.state.as_mut();
        cb(s)
    }
    /// resumes the device 
    pub fn resume(&self) {
        self.sdl2_device.resume();
    }

    /// pauses the device 
    pub fn pause(&self) {
        self.sdl2_device.pause();
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

pub struct FlufflCallback<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + Copy + Send + 'static,
    S: Send,
{
    audio_device: Arc<Mutex<RefCell<FlufflAudioDevice<F, S>>>>,
}

impl<Callback, State> sdl2::audio::AudioCallback for FlufflCallback<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Send + Copy,
    State: Send + 'static,
{
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let mut glue_cb = self.audio_device.lock().unwrap().borrow().callback();
        let device_lock = self.audio_device.lock().unwrap();
        let device = &mut *device_lock.borrow_mut();
        device.state().map(|state| {
            glue_cb(state, out);
        });
    }
}

// impl<F, S> IntoWithArg<FlufflAudioDeviceContext<F, S>, Arc<RefCell<FlufflAudioContext>>>
//     for AudioDeviceCore<F, S>
// where
//     F: FnMut(&mut S, &mut [f32]) + Send + std::marker::Copy,
//     S: Send,
// {
//     fn into_with(self, arg: Arc<RefCell<FlufflAudioContext>>) -> FlufflAudioDeviceContext<F, S> {
//         FlufflAudioDeviceContext::new(self, arg)
//     }
// }
