use super::AudioDeviceCore;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

pub struct FlufflAudioContext {
    //has platform specifitc pointers and stuff in here 
}

pub struct FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send + 'static,
    State: Send + 'static,
{
    fluffl_audio_device: Arc<Mutex<FlufflAudioDevice<Callback, State>>>,
}

impl<Callback, State> Clone for FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send,
    State: Send,
{
    fn clone(&self) -> Self {
        Self {
            fluffl_audio_device: self.fluffl_audio_device.clone(),
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
        let audio_device = Arc::new(Mutex::new(FlufflAudioDevice { core }));

        let glue_callback = FlufflCallback {
            audio_device: audio_device.clone(),
        };

        Self {
            fluffl_audio_device: audio_device,
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
        unimplemented!("resume(..)")
    }

    /// pauses the device
    pub fn pause(&self) {
        unimplemented!("pause(..) unimplemented ")
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
