use super::{HasAudioStream, StreamState};
use crate::collections::segment_tree::Interval;

// an implicit function f(t), where t is in seconds and 1 <= f(t) <= -1;
pub struct ImplicitWave {
    wave_function: fn(f64) -> f64,
    state: StreamState,
}
impl ImplicitWave {
    pub fn new(func: fn(f64) -> f64, interval: Interval, frequency: u32) -> Self {
        Self {
            wave_function: func,
            state: StreamState {
                global_interval: interval,
                frequency,
                attack_time: 0,
                release_time: 0,
                local_time: 0,
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
    fn pull_samples(&mut self, samples: &mut [f32]) {
        let sampling_rate = self.state.frequency;
        let samples_per_ms = self.state.frequency / 1000;

        let wave_function = self.wave_function;
        let mut local_time_in_seconds = self.state.local_time as f64 / 1000.0;
        let dt = 1.0 / (sampling_rate as f64);

        for output in samples.iter_mut() {
            *output = wave_function(local_time_in_seconds) as f32;
            local_time_in_seconds += dt;
        }

        
        self.state.local_time += (samples.len() as u128) / samples_per_ms as u128;
    }
}
