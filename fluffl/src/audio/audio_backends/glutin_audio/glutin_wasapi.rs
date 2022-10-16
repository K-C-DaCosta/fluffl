use super::*;
use crate::{console::*, *};
use std::sync::{Arc, Mutex};
use std::{thread, time::{Instant,Duration}};


use windows::{
    core::*,
    Win32::{
        UI::Shell::PropertiesSystem::PROPERTYKEY,
        Devices::FunctionDiscovery::*,
        Media::{
            Audio::{eRender, IMMDeviceEnumerator, MMDeviceEnumerator, WAVEFORMATEX, *},
            KernelStreaming::*,
            Multimedia::*,
        },
        System::Com::*,
    },
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

        Self {
            fluffl_audio_device: audio_device,
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
        // unimplemented!("wasapi not implemented")
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
