use super::{HasAudioStream, StreamState, *};
use crate::audio::interval::*;

mod explicit_wave;
mod implicit_wave;

pub use self::{
    explicit_wave::{ExplicitWave, ScaleMode},
    implicit_wave::ImplicitWave,
};

#[allow(dead_code)]
fn smoothstep_f32(x: f32, e0: f32, e1: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3. - 2. * t)
}

#[allow(dead_code)]
fn linear_t_f64(x: f64, e0: f64, e1: f64) -> f64 {
    ((x - e0) / (e1 - e0)).clamp(0.0, 1.0)
}

#[allow(dead_code)]
fn linear_t_f32(x: f32, e0: f32, e1: f32) -> f32 {
    ((x - e0) / (e1 - e0)).clamp(0.0, 1.0)
}

#[allow(dead_code)]
fn smooth_f64(x: f64, e0: f64, e1: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3. - 2. * t)
}
