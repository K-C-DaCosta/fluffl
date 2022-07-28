use super::*;
use crate::audio;

use adhoc_audio::{AdhocCodec, SeekFrom, StreamInfo, Streamable};

const MAX_CHANNELS_TO_MIX: usize = 8;

#[derive(Copy, Clone)]
pub enum ScaleMode {
    /// will strech audio if the elapsed time specified by interval is larger than the elapsed time of the audio
    Stretch,
    /// will loop if the elapsed time specified by interval is larger than the elapsed time of the audio
    Repeat,
}
impl Default for ScaleMode {
    fn default() -> Self {
        Self::Repeat
    }
}

pub struct ExplicitWave {
    state: StreamState,
    explicit_wave: AdhocCodec,
    /// the duration of the wave expessed as a rational number 
    explicit_wave_duration: SampleTime,
    scale_mode: ScaleMode,
}
impl ExplicitWave {
    pub fn new(explicit_wave: AdhocCodec, mode: ScaleMode) -> Self {
        let info = explicit_wave.info();
        let total_samples = explicit_wave.calculate_sample_count_per_channel();
        let wave_duration = SampleTime::new()
            .with_sample_count(total_samples)
            .with_sample_rate(info.frequency() as u32);

        Self {
            state: StreamState {
                local_time: SampleTime::new()
                    .with_sample_count(0)
                    .with_sample_rate(info.frequency() as u32),
                global_interval: Interval::from_length(wave_duration.elapsed_in_ms_fp()),
                attack_time: 2000,
                release_time: 2000,
                frequency: info.frequency() as u32,
                channels: info.channels() as u32,
                gain: 1.0,
                pan: 0.5,
            },
            explicit_wave,
            explicit_wave_duration: wave_duration,
            scale_mode: mode,
        }
    }

    fn pull_samples_repeat_non_repeat<'a>(
        &mut self,
        scratch_space: &mut [f32],
        mut audio_pcm: PCMSlice<'a, f32>,
    ) -> PullInfo {
        let mut sample_group = [0.0; MAX_CHANNELS_TO_MIX];

        //compute and temporarily store local_time, local_attack and local_release times
        let mut local_time_in_ms = self.state.local_time.elapsed_in_ms_f32();
        let track_delta_in_ms = 1000.0 / self.frequency() as f32;
        let elapsed_track_time_in_ms = self.interval().distance().as_f64() as f32;
        let local_attack_in_ms = self.state.attack_time as f32;
        let local_release_in_ms = elapsed_track_time_in_ms - self.state.release_time as f32;

        let num_channels_in_output = audio_pcm.channels() as usize;
        let num_channels_in_explicit_wave = self.explicit_wave.info().channels() as usize;
        let samples_writeable_per_channel = audio_pcm.samples_per_channel() as usize;
        let samples_needed_to_read = num_channels_in_explicit_wave * samples_writeable_per_channel;

        //read from compressed stream
        let samples_read = self
            .explicit_wave
            .decode(&mut scratch_space[0..samples_needed_to_read])
            .unwrap_or_default();

        let samples_read_per_channel = samples_read / num_channels_in_explicit_wave;

        //increment local time
        self.state
            .local_time
            .increment(samples_read_per_channel as u64);

        let samples_decoded = &scratch_space[..samples_read];

        for samp_idx in 0..samples_read_per_channel {
            //write concurrent samples into a small temporary array
            for channel_idx in 0..num_channels_in_explicit_wave {
                let samp =
                    samples_decoded[(num_channels_in_explicit_wave * samp_idx) + channel_idx];
                sample_group[channel_idx] = samp;
            }

            // extends sample_group
            // suppose num_channels_in_explicit_wave = 3 and sample_group = [ .1, .2, .3 , 0, 0, 0, 0, 0 ]
            // then after sample_group becomes: [.1 , .2, .3 , .3, .3, .3, .3, .3]
            let last_sample = sample_group[num_channels_in_explicit_wave - 1];
            for sample_element in &mut sample_group[num_channels_in_explicit_wave..] {
                *sample_element = last_sample;
            }

            //mix auxilary channels into existing channels
            for aux_sample_idx in num_channels_in_output..num_channels_in_explicit_wave {
                let aux_sample = sample_group[aux_sample_idx];
                //loop over samples I want to keep and blend the aux signal into it
                for kept_samples_idx in 0..num_channels_in_output {
                    sample_group[kept_samples_idx] =
                        (sample_group[kept_samples_idx] + aux_sample) * 0.5;
                }
            }

            //write samples to output
            for channel_idx in 0..num_channels_in_output {
                audio_pcm[num_channels_in_output * samp_idx + channel_idx] =
                    sample_group[channel_idx];
            }
        }

        // apply attack and release blending
        let gain = self.state.gain;
        for samp_idx in 0..samples_read_per_channel {
            //compute attack and release spline coefs
            let attack_t = 1.0 - linear_t_f32(local_time_in_ms, 0.0, local_attack_in_ms);
            let release_t = linear_t_f32(
                local_time_in_ms,
                elapsed_track_time_in_ms,
                local_release_in_ms,
            );

            let attack_coef = 1.0 - attack_t * attack_t;
            let relase_coef = release_t * release_t;
            let attack_release_gain = attack_coef * relase_coef * gain;

            for channel_idx in 0..num_channels_in_output {
                audio_pcm[samp_idx * num_channels_in_output + channel_idx] *= attack_release_gain;
            }

            //advance local time
            local_time_in_ms += track_delta_in_ms;
        }

        let mixed_samples_written = samples_read_per_channel * num_channels_in_output;

        PullInfo {
            samples_read: mixed_samples_written,
            samples_read_per_channel: samples_read_per_channel,
            elapsed_audio_in_ms: audio::calculate_elapsed_time_in_ms_fp(
                self.frequency(),
                samples_read_per_channel,
            ),
        }
    }

    /// ## Description
    /// pulls samples in repeat mode
    /// ## Comments & Warnings
    /// this code will break if  `audio_pcm` is larger than the sound track because it will require multiple seeks back to start
    /// and this code will only do one seek (only one seek is required when the cursor is smaller than the audio )  
    fn pull_samples_repeat_repeat<'a>(
        &mut self,
        scratch_space: &mut [f32],
        mut audio_pcm: PCMSlice<'a, f32>,
    ) -> PullInfo {
        let samples_needed_per_channel = audio_pcm.samples_per_channel() as usize;
        let first_pull_info = self.pull_samples_repeat_non_repeat(scratch_space, audio_pcm);
        let has_reached_the_end_of_explicit_wave =
            first_pull_info.samples_read_per_channel < samples_needed_per_channel;

        let audio_pcm_len = audio_pcm.len();

        if has_reached_the_end_of_explicit_wave {
            // copy previously pulled data into scratch space for temporary storage
            // im storing it at the end of scratch space array btw
            let previous_pull_audio_start = scratch_space.len() - first_pull_info.samples_read - 1;
            for idx in 0..first_pull_info.samples_read {
                scratch_space[previous_pull_audio_start + idx] = audio_pcm[idx];
            }

            // seek all the way back to the beginning of the track
            self.explicit_wave.seek(SeekFrom::Start(0));

            // do another,final, pull to complete the loop
            let second_pull_info = self.pull_samples_repeat_non_repeat(scratch_space, audio_pcm);

            // copy second pull to end out output
            let offset_needed_to_shift_to_end_of_pcm_buffer =
                audio_pcm_len - second_pull_info.samples_read;
            for idx in (0..second_pull_info.samples_read).rev() {
                audio_pcm[idx + offset_needed_to_shift_to_end_of_pcm_buffer] = audio_pcm[idx];
            }

            // copy audio from the initial pull back into the start of the pcm buffer
            let previous_pull_audio_slice = &mut scratch_space[previous_pull_audio_start..];
            for (idx, &e) in previous_pull_audio_slice.iter().enumerate() {
                audio_pcm[idx] = e;
            }

            PullInfo {
                samples_read: audio_pcm.len(),
                samples_read_per_channel: samples_needed_per_channel,
                elapsed_audio_in_ms: audio::calculate_elapsed_time_in_ms_fp(
                    audio_pcm.frequency(),
                    audio_pcm.len(),
                ),
            }
        } else {
            first_pull_info
        }
    }

    pub fn pull_samples_stretch<'a>(
        &mut self,
        scratch_space: &mut [f32],
        audio_pcm: PCMSlice<'a, f32>,
    ) -> PullInfo {
        unimplemented!("stretch not implemeted");
        PullInfo {
            samples_read: 0,
            samples_read_per_channel: 0,
            elapsed_audio_in_ms: FixedPoint::zero(),
        }
    }
}
impl Debug for ExplicitWave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} duration={}ms",
            self.state,
            self.explicit_wave_duration.elapsed_in_ms_f32()
        )
    }
}
impl HasAudioStream for ExplicitWave {
    fn stream_state(&self) -> &StreamState {
        &self.state
    }
    fn stream_state_mut(&mut self) -> &mut StreamState {
        &mut self.state
    }
    fn pull_samples<'a>(
        &mut self,
        scratch_space: &mut [f32],
        audio_pcm: PCMSlice<'a, f32>,
    ) -> PullInfo {
        match self.scale_mode {
            ScaleMode::Repeat => self.pull_samples_repeat_repeat(scratch_space, audio_pcm),
            ScaleMode::Stretch => self.pull_samples_stretch(scratch_space, audio_pcm),
        }
    }
    fn seek(&mut self, global_time: SampleTime) {
        let global_interval = self.state.global_interval;
        let elapsed_time_in_ms = global_interval.distance();
        let frequency = self.frequency();

        match self.scale_mode {
            ScaleMode::Repeat => {
                //
                let explicit_wave_duration_ms = self.explicit_wave_duration.elapsed_in_ms_u64();

                let new_local_time_in_ms = (global_time.elapsed_in_ms_fp() - global_interval.lo)
                    .clamp(FixedPoint::zero(), elapsed_time_in_ms)
                    .as_int_i64();

                // circular time because the track interval may over-extend the duration of the explicit wave
                let new_local_time_in_ms_circular =
                    new_local_time_in_ms as u64 % explicit_wave_duration_ms;

                // seek to circular time
                self.explicit_wave
                    .seek(SeekFrom::Start(new_local_time_in_ms_circular));

                // update local time cursor
                self.state.local_time = audio::calculate_samples_needed_per_channel_st(
                    frequency,
                    FixedPoint::from(new_local_time_in_ms),
                );
            }
            ScaleMode::Stretch => {
                let new_local_time_in_ms = (global_time.elapsed_in_ms_fp() - global_interval.lo)
                    .clamp(FixedPoint::zero(), elapsed_time_in_ms)
                    .as_int_i64();

                self.explicit_wave
                    .seek(SeekFrom::Start(new_local_time_in_ms as u64))
            }
        };
    }
}

#[test]
fn test_candidates() {
    let mut candidates = vec![10, 1, 2, 7, 6, 1, 5];
    let res = candidate_target(&mut candidates, 8);
    for arr in res {
        println!("{:?}", arr);
    }
}
fn candidate_target(candidates: &mut Vec<u32>, target: u32) -> Vec<Vec<u32>> {
    use std::collections::HashSet;

    // candidates.sort();

    let mut candidate_groups = Vec::<u128>::new();
    let candidates_len = candidates.len();
    let max_permutation = 1 << candidates.len();
    let mut unique_values_selected_table = HashSet::<u32>::new();

    for permutation_set in 0..max_permutation {
        let mut sum = 0;
        let mut values_selected_code = 0;
        for k in 0..candidates_len {
            let is_chosen = ((permutation_set & (1 << k)) != 0) as u32;
            sum += is_chosen * candidates[k];
            values_selected_code |= (1 << candidates[k]) * is_chosen;
            if sum > target {
                break;
            }
        }
        if unique_values_selected_table.contains(&values_selected_code) == false && sum == target {
            unique_values_selected_table.insert(values_selected_code);
            candidate_groups.push(permutation_set);
        }
    }
    candidate_groups
        .into_iter()
        .map(|bitset| {
            //rebuild set from bitset
            let mut items = vec![];
            for k in 0..candidates_len {
                if (bitset & (1 << k)) != 0 {
                    items.push(candidates[k]);
                }
            }
            items
        })
        .collect::<Vec<_>>()
}
