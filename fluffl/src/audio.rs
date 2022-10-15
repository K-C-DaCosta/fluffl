use crate::math::FP64;



pub mod interval;
pub mod mixer;
pub mod pcm_util;
pub mod audio_backends;


//expose these from audio itself
pub use interval::Interval;
pub use pcm_util::PCMSlice;

/// When playing/generating sound a callback will be required and it will need to be of this format.
pub type DeviceCB<State> = fn(&mut State, &mut [f32]);

/// Platform specific code awaits

pub use audio_backends::*;

use self::mixer::SampleTime;
/// A trait used to define properties of the sound before playing
pub trait GenericAudioSpecs {
    fn sample_rate(&self) -> Option<u32>;
    fn bits_per_sample(&self) -> Option<u32>;
    fn channels(&self) -> Option<u32>;
}

/// A POD-ish struct for defining properties of the sound we with to play \
/// If one of the fields isn't defined it will fallback to a somewhat sane default value
pub struct DesiredSpecs {
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub buffer_size: Option<u32>,
}
impl DesiredSpecs {
    #[allow(dead_code)]
    fn get_specs(&self) -> (u32, usize, usize) {
        (
            self.sample_rate.unwrap_or(48000),
            self.channels.unwrap_or(2) as usize,
            self.buffer_size.unwrap_or(1024) as usize,
        )
    }
}

/// The core of `AudioDeviceCore` has common resources all platform-specific implementations will need. \
/// Creating this object is not enough to play sound. We will need to convert this into a `FlufflAudioDeviceContex`\
/// This struct is mostly used as a way to setup state, define a callback and specify things like channels, frequency, etc.\
/// Look at the examples for a complete example on how to do this.
pub struct AudioDeviceCore<Callback, State> {
    cb: Option<Callback>,
    state: Option<State>,
    desired_specs: DesiredSpecs,
}

impl<Callback, State> AudioDeviceCore<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + 'static,
    State: 'static,
{
    pub fn new() -> Self {
        Self {
            cb: None,
            state: None,
            desired_specs: DesiredSpecs {
                sample_rate: None,
                channels: None,
                buffer_size: None,
            },
        }
    }
    ///A callback is needed to supply the audio backend with sound samples.
    ///Sound samples are expected to be interleaved-pcm
    pub fn with_callback(mut self, cb: Callback) -> Self {
        self.cb = Some(cb);
        self
    }

    pub fn with_state(mut self, state: State) -> Self {
        self.state = Some(state);
        self
    }

    pub fn with_specs(mut self, specs: DesiredSpecs) -> Self {
        self.desired_specs = specs;
        self
    }

    pub fn callback(&self) -> Callback {
        self.cb.unwrap()
    }
}

/// given `frequency` (in sample/sec) and `dt`(in milliseconds), it can calculate samples required per channel
pub fn calculate_samples_needed_per_channel_st(frequency: u32, dt: FP64) -> SampleTime {
    const MILLISECONDS_IN_ONE_SEC: i32 = 1000;
    let sample_count = (FP64::from(frequency) * dt) / MILLISECONDS_IN_ONE_SEC;
    SampleTime::new()
        .with_sample_rate(frequency)
        .with_sample_count(sample_count.as_i64() as u64)
}

/// given `frequency` (in sample/sec) and `dt`(in milliseconds), it can calculate samples required per channel
pub fn calculate_samples_needed_per_channel_fp(frequency: u32, dt: FP64) -> FP64 {
    const MILLISECONDS_IN_ONE_SEC: i32 = 1000;
    let result = (FP64::from(frequency) * dt) / MILLISECONDS_IN_ONE_SEC;
    // let mut samps = result.as_f64();
    // samps += 1.0;
    // samps -= 1.0;
    result
}

/// given a `num_samples` and `frequency` it returns the elapsed time in ms
/// ## Comments
/// this is a single channel calculation
pub fn calculate_elapsed_time_in_ms_fp(frequency: u32, num_samples: usize) -> FP64 {
    let result = FP64::from(num_samples as u64 * 1000) / FP64::from(frequency);
    // let mut samps = result.as_f64();
    // samps += 1.0;
    // samps -= 1.0;
    result
}
