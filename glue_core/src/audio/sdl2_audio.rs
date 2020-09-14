use super::{GlueAudioDeviceCore, IntoWithArg};

use std::cell::RefCell;
use std::fs::File;
use std::sync::{Arc, Mutex};

// use sdl2::audio;
use std::fs;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{BufRead, BufReader, Read};
use std::slice::*;

pub struct GlueAudioContext {
    pub audio_ss: sdl2::AudioSubsystem,
}


pub struct GlueAudioDeviceContext<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send,
    S: Send,
{
    glue_audio_device: Arc<Mutex<RefCell<GlueAudioDevice<F, S>>>>,
    sdl2_device: Arc<sdl2::audio::AudioDevice<GlueCallback<F, S>>>,
}
impl <F,S> Clone for GlueAudioDeviceContext<F,S>
where
F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send,
S: Send,
{
    fn clone(&self) -> Self {
        Self{
            glue_audio_device:  self.glue_audio_device.clone(),
            sdl2_device: self.sdl2_device.clone(),
        }
    }
}



impl<F, S> GlueAudioDeviceContext<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send,
    S: Send,
{
    pub fn new(
        core: GlueAudioDeviceCore<F, S>,
        audio_context: Arc<RefCell<GlueAudioContext>>,
    ) -> GlueAudioDeviceContext<F, S> {
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

        let audio_device = Arc::new(Mutex::new(RefCell::new(GlueAudioDevice { core })));

        let glue_callback = GlueCallback {
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
            glue_audio_device: audio_device,
            sdl2_device: Arc::new(sdl2_device),
        }
    }

    pub fn modify_state<CBF>(&self, mut cb: CBF)
    where
        CBF: FnMut(Option<&mut S>),
    {
        let lck = self.glue_audio_device.lock().unwrap();
        let device = &mut *lck.borrow_mut();
        let s = device.core.state.as_mut();
        cb(s)
    }

    pub fn resume(&self) {
        self.sdl2_device.resume();
    }

    pub fn pause(&self) {
        self.sdl2_device.pause();
    }
}

pub struct GlueAudioDevice<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send,
    S: Send,
{
    core: GlueAudioDeviceCore<F, S>,
}

impl<F, S> GlueAudioDevice<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send,
    S: Send,
{
    pub fn callback(&self) -> F {
        self.core.callback()
    }

    pub fn state(&mut self) -> Option<&mut S> {
        self.core.state.as_mut()
    }
}

pub struct GlueCallback<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send,
    S: Send,
{
    audio_device: Arc<Mutex<RefCell<GlueAudioDevice<F, S>>>>,
}

impl<F, S> sdl2::audio::AudioCallback for GlueCallback<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + Send + std::marker::Copy,
    S: Send,
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

impl<F, S> IntoWithArg<GlueAudioDeviceContext<F, S>, Arc<RefCell<GlueAudioContext>>>
    for GlueAudioDeviceCore<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + Send + std::marker::Copy,
    S: Send,
{
    fn into_with(self, arg: Arc<RefCell<GlueAudioContext>>) -> GlueAudioDeviceContext<F, S> {
        GlueAudioDeviceContext::new(self, arg)
    }
}
