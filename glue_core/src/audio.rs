use crate::io::*;
use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{BufRead, BufReader};
use std::rc::Rc;
use std::slice::*;
use std::sync::Arc;

#[cfg(feature = "desktop")]
#[path = "./audio/sdl2_audio.rs"]
pub mod audio_util;

#[cfg(feature = "web")]
#[path = "./audio/web_audio.rs"]
pub mod audio_util;

pub mod wav;

pub use audio_util::*;

pub trait PcmConverter<T> {
    fn samples(&self) -> Vec<AudioSample<T>>;
}

#[derive(Copy, Clone)]
pub struct AudioSample<T> {
    pub channel: [T; 2],
}

pub struct AudioBuffer<T> {
    pub samples: Vec<AudioSample<T>>,
    pub sample_index: usize,
}

impl<T> AudioBuffer<T>
where
    T: Copy,
{
    pub fn read(&mut self, out: &mut [AudioSample<T>]) -> usize {
        if self.sample_index >= self.samples.len() {
            return 0;
        }

        let buff = &self.samples[self.sample_index..];
        let out_len = out.len();
        let buff_len = buff.len();
        let mut samples_read = 0;

        buff.iter()
            .enumerate()
            .take_while(|&(i, _)| i < out_len.min(buff_len))
            .for_each(|(i, &s)| {
                out[i] = s;
                samples_read += 1;
            });

        self.sample_index += samples_read;

        samples_read
    }
}

impl<T> From<[T; 2]> for AudioSample<T> {
    fn from(list: [T; 2]) -> Self {
        Self { channel: list }
    }
}

pub struct GlueDesiredSpecs {
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub buffer_size: Option<u32>,
}
impl GlueDesiredSpecs {
    fn get_specs(&self) -> (u32, usize, usize) {
        (
            self.sample_rate.unwrap_or(48000),
            self.channels.unwrap_or(2) as usize,
            self.buffer_size.unwrap_or(1024) as usize,
        )
    }
}

pub trait IntoWithArg<T, Arg> {
    fn into_with(self, arg: Arg) -> T;
}

//has common resources both implementations need
pub struct GlueAudioDeviceCore<F, S> {
    cb: Option<F>,
    state: Option<S>,
    desired_specs: GlueDesiredSpecs,
}

impl<F, S> GlueAudioDeviceCore<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + Copy,
{
    pub fn new() -> Self {
        Self {
            cb: None,
            state: None,
            desired_specs: GlueDesiredSpecs {
                sample_rate: None,
                channels: None,
                buffer_size: None,
            },
        }
    }

    pub fn with_callback(mut self, cb: F) -> Self {
        self.cb = Some(cb);
        self
    }

    pub fn with_state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    pub fn with_specs(mut self, specs: GlueDesiredSpecs) -> Self {
        self.desired_specs = specs;
        self
    }

    pub fn callback(&self) -> F {
        self.cb.unwrap()
    }
}
