use super::{HasAudioStream, StreamState, *};
use crate::audio::interval::*;

fn smoothstep_f32(x: f32, e0: f32, e1: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3. - 2. * t)
}

fn linear_t_f64(x: f64, e0: f64, e1: f64) -> f64 {
    ((x - e0) / (e1 - e0)).clamp(0.0, 1.0)
}

fn smooth_f64(x: f64, e0: f64, e1: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3. - 2. * t)
}

// an implicit function f(t), where t is in seconds and -1 <= f(t) <= 1;
pub struct ImplicitWave {
    wave_function: fn(f64) -> f64,
    state: StreamState,
}

impl ImplicitWave {
    pub fn new(func: fn(f64) -> f64, interval: Interval, frequency: u32) -> Self {
        //attack and release should be at least 2% of the elapsed time, to avoid cracks
        const ATTACK_RELEASE_RATIO: f64 = 0.2;
        let total_elapsed_time = interval.distance() as f64;
        let default_attack_and_release = (ATTACK_RELEASE_RATIO * total_elapsed_time).ceil() as u32;
        println!(
            "default attack and release = {}ms ",
            default_attack_and_release
        );
        Self {
            wave_function: func,
            state: StreamState {
                global_interval: interval,
                frequency,
                attack_time: default_attack_and_release,
                release_time: default_attack_and_release,
                local_time: SampleTime::new().with_sample_rate(frequency),
                channels: 1,
            },
        }
    }
}
impl HasAudioStream for ImplicitWave {
    fn stream_state(&self) -> &StreamState {
        &self.state
    }
    fn stream_state_mut(&mut self) -> &mut StreamState {
        &mut self.state
    }

    fn pull_samples(&mut self, mut pcm_buffer: PCMSlice<f32>) -> usize {
        let num_samples = pcm_buffer.len() / 2;
        let wave_function = self.wave_function;
        let end_time_in_seconds = self.interval().hi as f64 / 1000.0;
        let local_time = &mut self.state.local_time;

        let mut time_in_seconds = local_time.elapsed_in_sec_f64();
        let dt = local_time.sample_delta_in_sec_f64();

        let attack_time_in_seconds = self.state.attack_time as f64 / 1000.;
        let release_time_in_seconds = self.state.release_time as f64 / 1000.0;

        for block_idx in 0..num_samples {
            let attack_t = linear_t_f64(time_in_seconds, 0.0, attack_time_in_seconds);
            let release_t = linear_t_f64(
                time_in_seconds,
                end_time_in_seconds - release_time_in_seconds,
                end_time_in_seconds,
            );
            let attack_coef = 1.0 - (1.0 - attack_t).powf(2.);
            let release_coef = 1.0-(release_t).powf(2.);

            let output = attack_coef * release_coef * wave_function(time_in_seconds);

            let output = output as f32;
            pcm_buffer[2 * block_idx + 0] = output;
            pcm_buffer[2 * block_idx + 1] = output;

            local_time.increment(1);
            time_in_seconds += dt;
        }

        num_samples * 2
    }
}
