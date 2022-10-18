pub mod noise;

use std::mem;

#[derive(Copy, Clone)]
pub enum WaveKind {
    Sine = 0,
    Square = 1,
    Triangle = 2,
    SawTooth = 3,
    NoiseValue = 4,
    NoisePerlin = 5,
}
impl WaveKind {
    pub fn as_fn(self) -> fn(f64) -> f64 {
        match self {
            Self::Sine => sin,
            Self::NoiseValue => value_noise,
            Self::Square => square_wave,
            Self::SawTooth => sawtooth,
            Self::Triangle => triangle,
            Self::NoisePerlin => perlin_noise,
        }
    }
}

impl From<usize> for WaveKind {
    fn from(mut val: usize) -> Self {
        val %= 6;
        unsafe { mem::transmute_copy(&val) }
    }
}

pub fn angular_frequency(freq: f64) -> f64 {
    2.0 * std::f64::consts::PI * freq
}

pub fn square_wave(t: f64) -> f64 {
    const PERIOD: f64 = 2.0 * std::f64::consts::PI;
    const FREQ: f64 = 1.0 / PERIOD;
    let wave = 2.0 * (2.0 * (FREQ * t).floor() - (2.0 * FREQ * t).floor()) + 1.0;
    wave * 0.1
}

pub fn sawtooth(t: f64) -> f64 {
    const PERIOD: f64 = 2.0 * std::f64::consts::PI;
    let m = t / PERIOD;
    let tooth = 2.0 * (m - (0.5 + m).floor());
    tooth * 0.1
}

pub fn triangle(t: f64) -> f64 {
    const PERIOD: f64 = 2.0 * std::f64::consts::PI;
    let m = t / PERIOD;
    let unsigned_triangle = 2.0 * (m - (0.5 + m).floor()).abs();
    let signed_trangle = unsigned_triangle * 2.0 - 1.0;
    signed_trangle * 0.1
}

pub fn sin(t: f64) -> f64 {
    t.sin() * 0.1
}

pub fn value_noise(t: f64) -> f64 {
    let noise = (noise::value_noise_1d_octaves(t as f32, 8)) as f64;
    noise * 0.5
}

pub fn perlin_noise(t: f64) -> f64 {
    let amplitudes = [1.,];

    let (n, amp_max) = amplitudes
        .iter()
        .fold((0.0, 0.0), |(n_acc, amp_acc), &amp| {
            (
                n_acc + noise::perlin_noise_1d(t as f32 / amp) * amp,
                amp_acc + amp,
            )
        });
    (n/amp_max) as f64 * 0.1
}
