use super::{AudioDeviceCore};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

/// The platform specific  Audio context for the desktop
/// Here I use sdl2 as the backend
#[derive(Clone)]
pub struct FlufflAudioContext {
    pub audio_ss: Arc<RefCell<be_sdl2::AudioSubsystem>>,
}

/// # Description 
/// You use this to actually start playing the sound.
/// This struct is just a generic 'handler'/'pointer' to the audio backend, and to the state that 
/// was defined in the core
/// # Desktop Comments
/// A pointer to SDL2's audio device (which is uses some multithreading). Nothing really interesting happens on this side. 
/// Its literally just a wrapper, for sdl2's interface.
/// # Wasm/Web Comments
/// The implementation for this side of things was much more complicated than I could've
/// imagined. With a lot of help from these sources: 
/// - https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API
/// - https://blog.mecheye.net/2017/09/i-dont-know-who-the-web-audio-api-is-designed-for/
/// - https://rustwasm.github.io/docs/wasm-bindgen/" \
/// I was able to cobble  something together that actually seems to work okay , well, for now anyway.
/// A new chrome or firefox update could break this code further.
/// As of writing this Chrome appearently reduced timer resolutions which slightly break the wasm implemention 
/// causing noticable pops and cracks in the audio playback(Firefox works great though).
/// Look is the `web_audio.rs` module for a peek at the  wasm implementation
pub struct FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send + 'static,
    State: Send + 'static,
{
    fluffl_audio_device: Arc<Mutex<FlufflAudioDevice<Callback, State>>>,
    sdl2_device: Arc<be_sdl2::audio::AudioDevice<FlufflCallback<Callback, State>>>,
}

impl<Callback, State> Clone for FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send,
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
    /// creates a platform-agnostic FlufflAudioDevice
    pub fn new(
        core: AudioDeviceCore<Callback, State>,
        audio_context: FlufflAudioContext,
    ) -> FlufflAudioDeviceContext<Callback, State> {
        // println!("new music context");
        let desired_spec = be_sdl2::audio::AudioSpecDesired {
            freq: core.desired_specs.sample_rate.clone().map(|a| {
                // println!("freq = {}", a);
                a as i32
            }),
            channels: core.desired_specs.channels.clone().map(|a| {
                // println!("channels = {}", a);
                a as u8
            }),
            samples: core.desired_specs.buffer_size.clone().map(|a| {
                // println!("buffer_size = {}", a);
                a as u16
            }),
        };

        let audio_device = Arc::new(Mutex::new(FlufflAudioDevice { core }));

        let glue_callback = FlufflCallback {
            audio_device: audio_device.clone(),
        };

        let sdl2_device = audio_context
            .audio_ss
            .borrow_mut()
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
    
    /// ## Description
    /// Allows the user to modify state through a callback
    /// ### Comments
    /// If I can't easily return the value to code higher up in the stack, 
    /// the next best thing is pass a callback to the value  
    pub fn modify_state<ModifyCallback>(&self, mut cb: ModifyCallback)
    where
        ModifyCallback: FnMut(Option<&mut State>)->Option<()>,
    {
        let mut lck = self.fluffl_audio_device.lock().unwrap();
        let device = &mut *lck;
        let s = device.core.state.as_mut();
        let _ = cb(s);
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

pub struct FlufflCallback<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send + 'static,
    State: Send,
{
    audio_device: Arc<Mutex<FlufflAudioDevice<Callback, State>>>,
}

impl<Callback, State> be_sdl2::audio::AudioCallback for FlufflCallback<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Send + Copy,
    State: Send + 'static,
{
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let mut callback = self.audio_device.lock().unwrap().callback();
        let mut device_lock = self.audio_device.lock().unwrap();
        let device = &mut *device_lock;
        device.state().map(|state| {
            callback(state, out);
        });
    }
}