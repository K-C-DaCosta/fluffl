#![allow(unused_imports, dead_code)]
use fluffl::{
    audio::*,
    console::*,
    //playing music files requires more than what the base library provides
    //so here is my implementation of certain things like "text rendering" and music playing
    extras::hiero_pack::*,
    io::*,
    math::FP64,
    prelude::*,
    text_writer::*,
    // net::*,
    window::{event_util::*, *},
    *,
};

use std::collections::*;

fn sinf32(t: f32) -> f32 {
    (2.0 * std::f32::consts::PI * t).sin() * 0.2
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

pub struct AudioState {
    pub channels: u32,
    pub frequency: u32,
    pub t: f64,
    pub amplitude: f32,
    pub sound_waves: VecDeque<SoundWave>,
}
impl AudioState {
    pub fn new<CB>(mut init: CB) -> Self
    where
        CB: FnMut(&mut Self),
    {
        let mut state = Self {
            channels: 0,
            frequency: 0,
            t: 0.0,
            amplitude: 1.0,
            sound_waves: vec![].into_iter().collect(),
        };
        init(&mut state);
        state
    }
}

#[allow(clippy::identity_op)]
fn synth_callback_cb(state: &mut AudioState, output: &mut [f32]) {
    // Naive way of adding multiple implicit waves together (mixing but only for implicit waves )
    // upsides:
    // - easy to implement
    // downsides:
    // - practical only for implicit waves where you can compute f(t) easily, computing f(t) for sampled audio requires seeking (can be slow)
    // - can't add sampled audio like mp3's or whatever
    // - expensive, spatial datastructure can shorten innter loop by O(log(number_of_waves)) rather than  O(number_of_waves)
    // ------------------------------------------------------------------------------------------------------------------------

    let conversion_factor_sec = 1.0 / state.frequency as f64;
    let conversion_factor_ms = 1000.0 * conversion_factor_sec;

    for samp_idx in 0..output.len() / 2 {
        let mut dst = 0.0;

        let time_in_ms = (state.t * conversion_factor_ms) as f32;
        let time_in_seconds = state.t * conversion_factor_sec;

        let is_in_bounds = |wave: &&SoundWave| {
            time_in_ms > (wave.interval.0 + -1.0) && time_in_ms < (wave.interval.1 + 1.0)
        };

        for wave in state.sound_waves.iter().filter(is_in_bounds) {
            dst += wave.evaluate(time_in_seconds as f32);
        }

        output[2 * samp_idx + 0] = dst;
        output[2 * samp_idx + 1] = dst;
        state.t += 1.0;
    }
}

type ShortState = AudioState;
type ShortDeviceCB = fn(&mut ShortState, &mut [f32]);
type ShortDeviceContext = FlufflAudioDeviceContext<ShortDeviceCB, ShortState>;

#[fluffl(Debug)]
pub async fn main() {
    unimplemented!("example is in construction");
}

// ## Description
// returns approximate position of the clipped interval in the output buffer
// fn estimate_position_in_buffer(
//     cursor: Interval,
//     track_interval: Interval,
//     mixer_sample_rate: u32,
//     mixer_channels: u128,
// ) -> Interval {
//     let minimum_samples = ((track_interval.lo - cursor.lo) * mixer_sample_rate as u128) / 1000;
//     let maximum_samples = ((track_interval.hi - cursor.lo) * mixer_sample_rate as u128) / 1000;
//     Interval {
//         lo: minimum_samples * mixer_channels,
//         hi: maximum_samples * mixer_channels,
//     }
// }

// pub struct TrackList {
//     sound_track: Vec<Interval>,
// }
// impl TrackList {
//     pub fn with_track(mut sound_track: Vec<Interval>) -> Self {
//         sound_track.sort_by_key(|&i| i.lo);
//         Self { sound_track }
//     }

//     /// fetches any track that fits within the time coordinate
//     /// ### Notes:
//     /// - complexity: `O(log(n))`
//     pub fn get_any_track(&self, time: u128) -> Option<usize> {
//         let mut lo = 0;
//         let mut hi = self.sound_track.len() - 1;
//         let mut left_most_interval = None;

//         //binary search for the first interval that fits within the `time:u128` query
//         //this is used as an initial starting point for the left-most search
//         while lo <= hi {
//             let mid = (hi - lo) / 2 + lo;
//             let item = self.sound_track[mid];
//             if item.is_within(time) {
//                 left_most_interval = Some(mid);
//                 break;
//             } else if time < item.lo {
//                 //take left subarray
//                 hi = mid - 1;
//             } else {
//                 //take right subarray
//                 lo = mid + 1;
//             }
//         }

//         left_most_interval
//     }

//     /// fetches earliest track that intersects it
//     /// ### Notes:
//     /// - complexity: `O(log(n))`
//     pub fn get_earliest_track(&self, time: u128) -> Option<usize> {
//         let sound_track = &self.sound_track;
//         let left_most_interval = self.get_any_track(time);

//         // after the binary search you aren't nececiarily going to get the leftmost track that fits within `time`
//         // so you have to do bisection-like iterations to get there quick
//         if let Some(hi) = left_most_interval {
//             let mut hi = hi;
//             let mut lo = 0;
//             loop {
//                 let mid = (hi - lo) / 2 + lo;
//                 let hi_in = sound_track[hi].is_within(time);
//                 let lo_in = sound_track[lo].is_within(time);
//                 let mid_in = sound_track[mid].is_within(time);
//                 if hi - lo <= 1 {
//                     if lo_in {
//                         return Some(lo);
//                     }
//                     if hi_in {
//                         return Some(hi);
//                     }
//                 } else if hi_in == lo_in {
//                     return Some(lo);
//                 } else if mid_in != hi_in {
//                     lo = mid;
//                 } else if lo_in != mid_in {
//                     hi = mid;
//                 }
//             }
//         }

//         left_most_interval
//     }
// }
// impl Index<usize> for TrackList {
//     type Output = Interval;
//     fn index(&self, index: usize) -> &Self::Output {
//         &self.sound_track[index]
//     }
// }
