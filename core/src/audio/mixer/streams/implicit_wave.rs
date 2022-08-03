use super::*;
use crate::{audio, math};

/// represents an implicit function `f(t)`, where:
/// - `t` is in seconds
/// - |`f(t)`| <= 1
pub struct ImplicitWave {
    wave_frequency: f64,
    wave_function: fn(f64) -> f64,
    state: StreamState,
}

impl ImplicitWave {
    pub fn new(func: fn(f64) -> f64, interval: Interval, wave_frequency: f64) -> Self {
        //attack and release should be at least 2% of the elapsed time, to avoid cracks
        const ATTACK_RELEASE_RATIO: f64 = 0.2;
        //mixer frequency assumed to be 44_100hz
        const TRACK_FREQUENCY: u32 = 44_100;

        let total_elapsed_time = interval.distance().as_f64();
        let default_attack_and_release = (ATTACK_RELEASE_RATIO * total_elapsed_time).ceil() as u32;

        // println!(
        //     "default attack and release = {}ms ",
        //     default_attack_and_release
        // );
        Self {
            wave_function: func,
            wave_frequency,
            state: StreamState {
                global_interval: interval,
                frequency: TRACK_FREQUENCY,
                attack_time: default_attack_and_release,
                release_time: default_attack_and_release,
                local_time: SampleTime::new().with_sample_rate(TRACK_FREQUENCY),
                channels: 1,
                gain: 1.0,
                pan: 0.5,
            },
        }
    }
}
impl Debug for ImplicitWave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[freq:{}]", self.frequency())
    }
}
impl HasAudioStream for ImplicitWave {
    fn stream_state(&self) -> &StreamState {
        &self.state
    }
    fn stream_state_mut(&mut self) -> &mut StreamState {
        &mut self.state
    }

    fn seek(&mut self, global_time: SampleTime) {
        let interval = *self.interval();
        let duration_in_ms = interval.distance();
        let global_time_in_ms = global_time.elapsed_in_ms_fp();
        let local_time = global_time_in_ms - interval.lo;

        if local_time <= FP64::zero() || local_time >= duration_in_ms {
            self.state.local_time.set_samps(0);
            return;
        }

        self.state.local_time = self.state.local_time.from_time_in_ms_fp(local_time);
    }

    fn pull_samples(
        &mut self,
        _scratch_space: &mut [f32],
        mut pcm_buffer: PCMSlice<f32>,
    ) -> PullInfo {
        let gain = self.gain() as f64;

        let wave_function = self.wave_function;
        let local_time = &mut self.state.local_time;
        let interval = &self.state.global_interval;
        let wave_frequency = self.wave_frequency;

        //calculate local time and deltas
        let mut local_time_in_seconds = local_time.elapsed_in_sec_f64();
        let elapsed_time_for_a_single_sample_in_seconds = local_time.sample_delta_in_sec_f64();

        //calculate attack and release times
        let attack_time_in_ms = self.state.attack_time;
        let release_time_in_ms = self.state.release_time;
        let attack_time_in_seconds = attack_time_in_ms as f64 / 1000.;
        let release_time_in_seconds = release_time_in_ms as f64 / 1000.0;
        let release_end_time_in_seconds = interval.distance().as_f64() / 1000.0;
        let release_start_time_in_seconds = release_end_time_in_seconds - release_time_in_seconds;

        //fetch info about the output
        let samples_per_channel_in_output = pcm_buffer.samples_per_channel() as usize;
        let num_channels_in_output = pcm_buffer.channels() as usize;

        for output_sample_idx in 0..samples_per_channel_in_output {
            let attack_t = linear_t_f64(local_time_in_seconds, 0.0, attack_time_in_seconds);
            let release_t = linear_t_f64(
                local_time_in_seconds,
                release_start_time_in_seconds,
                release_end_time_in_seconds,
            );
            let attack_coef = 1.0 - (1.0 - attack_t) * (1.0 - attack_t);
            let release_coef = 1.0 - release_t * release_t;
            let output = attack_coef
                * wave_function(math::angular_frequency(wave_frequency) * local_time_in_seconds)
                * release_coef
                * gain;

            //write output sample in every channel (for now)
            for channel_idx in 0..num_channels_in_output {
                pcm_buffer[num_channels_in_output * output_sample_idx + channel_idx] =
                    output as f32;
            }

            //dont forget to increment time per sample
            local_time.increment(1);
            local_time_in_seconds += elapsed_time_for_a_single_sample_in_seconds;
        }

        PullInfo {
            samples_read: samples_per_channel_in_output * num_channels_in_output,
            samples_read_per_channel: samples_per_channel_in_output,
            elapsed_audio_in_ms: audio::calculate_elapsed_time_in_ms_fp(
                self.frequency(),
                samples_per_channel_in_output,
            ),
        }
    }
}
