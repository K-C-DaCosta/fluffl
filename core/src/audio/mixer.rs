use crate::{
    audio::{Interval, PCMSlice},
    collections::{
        linked_list::{DoublyLinkedList, LLNodeOps, LLOps, LinkedList, OptionNode},
        segment_tree::{index_types::GlobalIndex, CircularSegmentTree, TreeIterState},
        Ptr,
    },
    math::FixedPoint,
    mem,
};
use std::{collections::HashMap, ops::Index};

pub mod streams;
pub mod time;
pub use time::SampleTime;

#[derive(Copy, Clone)]
struct MixerCursor {
    t0: SampleTime,
    delta: SampleTime,
}
impl MixerCursor {
    pub fn new(t0: SampleTime, delta: SampleTime) -> Self {
        Self {
            t0: t0,
            delta: delta,
        }
    }

    pub fn to_interval_ms(&self) -> Interval {
        let lo = self.t0.elapsed_in_ms_fp();
        let hi = self.t0.sum(&self.delta).elapsed_in_ms_fp();
        Interval { lo, hi }
    }

    pub fn to_interval_tuple_ms_f32(&self) -> (f32, f32) {
        let lo = self.t0.elasped_in_ms_f32();
        let hi = self.t0.sum(&self.delta).elasped_in_ms_f32();
        (lo, hi)
    }
}

#[derive(Copy, Clone)]
pub struct StreamState {
    /// in milliseconds
    pub local_time: SampleTime,
    /// interval is in milliseconds
    pub global_interval: Interval,
    /// attack time is in milliseconds
    pub attack_time: u32,
    /// in milliseconds
    pub release_time: u32,

    /// in samples per seconds
    pub frequency: u32,
    pub channels: u32,
}

pub trait HasAudioStream: Send {
    fn stream_state(&self) -> &StreamState;
    fn stream_state_mut(&mut self) -> &mut StreamState;

    fn interval(&self) -> &Interval {
        &self.stream_state().global_interval
    }

    fn frequency(&self) -> u32 {
        self.stream_state().frequency
    }

    fn interval_mut(&mut self) -> &mut Interval {
        &mut self.stream_state_mut().global_interval
    }
    ///given a time interval `dt`(in *milliseconds*) returns number of samples needed to represent the interval
    fn calculate_samples_needed(&self, dt: u32) -> u32 {
        const NUM_MILLISECONDS_IN_ONE_SECOND: u32 = 1000;
        (self.frequency() * dt) / NUM_MILLISECONDS_IN_ONE_SECOND
    }

    fn calculate_samples_needed_fp(&self, dt: FixedPoint) -> FixedPoint {
        let NUM_MILLISECONDS_IN_ONE_SECOND: FixedPoint = FixedPoint::from(1000);
        (FixedPoint::from(self.frequency()) * dt) / NUM_MILLISECONDS_IN_ONE_SECOND
    }

    fn calculate_samples_needed_f32(&self, dt: f32) -> f32 {
        const NUM_MILLISECONDS_IN_ONE_SECOND: f32 = 1000.0;
        (self.frequency() as f32 * dt) / NUM_MILLISECONDS_IN_ONE_SECOND
    }

    fn time_remaining(&self) -> FixedPoint {
        let state = self.stream_state();
        let local_t = state.local_time.elapsed_in_ms_fp();
        let interval = state.global_interval;
        interval.distance() - local_t
    }

    fn is_dead(&self) -> bool {
        let state = self.stream_state();
        let local_t = state.local_time.elapsed_in_ms_fp();
        let interval = state.global_interval;
        local_t > interval.distance()
    }

    /// will advance local time
    fn pull_samples<'a>(&mut self, audio_pcm: PCMSlice<'a, f32>) -> usize;
}

/// mixes sounds together (assumed 2 channels for now)
pub struct Mixer {
    global_t: SampleTime,
    sample_pull_in: Vec<f32>,
    track_chart: CircularSegmentTree<Box<dyn HasAudioStream>>,
    running_streams_table: HashMap<GlobalIndex, Ptr>,

    /// streams where the cursor first intersects the track
    running_streams_on_intersection: Vec<GlobalIndex>,

    /// streams where the cursor has already intersected the track
    running_streams: LinkedList<GlobalIndex>,

    track_removal_stack: Vec<Ptr>,

    /// a coef that i use to avoid clipping
    clip_avoidance_dampening_coef: f32,
}
impl Mixer {
    pub fn new(sample_rate: u32, _channels: u32) -> Self {
        Self {
            global_t: SampleTime::new().with_sample_rate(sample_rate),
            sample_pull_in: vec![0.0f32; 4096 * 2],
            running_streams_on_intersection: Vec::new(),
            running_streams: LinkedList::new(),
            running_streams_table: HashMap::new(),
            track_chart: CircularSegmentTree::new(40, 1 << 40),
            track_removal_stack: vec![],
            clip_avoidance_dampening_coef: 1.0,
        }
    }

    pub fn get_time(&self) -> SampleTime {
        self.global_t
    }

    pub fn add_track(&mut self, track: Box<dyn HasAudioStream>) {
        let &interval = track.interval();
        self.track_chart.insert(interval, track);
    }

    pub fn mix_audio<'a>(&mut self, mut output_buffer: PCMSlice<'a, f32>) {
        let cursor = MixerCursor::new(
            self.global_t,
            self.global_t.with_sample_count(output_buffer.num_samples()),
        );

        output_buffer.zero_slice();

        self.search_for_active_tracks(cursor);

        self.mix_active_tracks(cursor, output_buffer);

        self.normalize_audio(output_buffer);

        self.remove_irrelevent_tracks(cursor);

        //update t
        self.global_t.increment(output_buffer.num_samples());
    }
    fn mix_active_tracks(&mut self, cursor: MixerCursor, output_buffer: PCMSlice<f32>) {
        // its easier to pull audio from tracks KNOWING that the cursor for the output buffer starts at ZERO
        self.handle_intersecting_tracks_not_first_time(cursor, output_buffer);

        // tracks that intersect the cursor for the first time are handled differently than
        self.handle_intersecting_tracks(cursor, output_buffer);
    }
    fn handle_intersecting_tracks(
        &mut self,
        cursor: MixerCursor,
        mut output_buffer: PCMSlice<f32>,
    ) {
        let track_chart = &mut self.track_chart;
        let running_streams_table = &mut self.running_streams_table;
        let running_streams_on_intersection = &mut self.running_streams_on_intersection;
        let running_streams_on_intersection_ptr =
            running_streams_on_intersection as *const Vec<GlobalIndex>;
        let running_streams = &mut self.running_streams;

        let sample_scratch_space = &mut self.sample_pull_in;

        // println!("list len = {}",running_streams_on_intersection.len());

        let iter = running_streams_on_intersection.iter().enumerate().rev();

        for (stream_vec_idx, &gi) in iter {
            let current_track = &mut track_chart[gi];
            let accurate_track_interval = current_track.interval().to_tuple_f32();
            let fractional_cursor = cursor.to_interval_tuple_ms_f32();

            let clipped_track_interval = {
                let lo = accurate_track_interval
                    .0
                    .clamp(fractional_cursor.0, fractional_cursor.1);
                let hi = accurate_track_interval
                    .1
                    .clamp(fractional_cursor.0, fractional_cursor.1);
                (lo, hi)
            };
            let clipped_duration = clipped_track_interval.1 - clipped_track_interval.0;
            let samples_needed = current_track.calculate_samples_needed_f32(clipped_duration);

            let estimated_position_in_output = estimate_position_in_buffer_f32(
                fractional_cursor,
                clipped_track_interval,
                output_buffer.frequency() as f32,
                output_buffer.channels() as f32,
            );

            let samples_needed_truncated = samples_needed.round() as usize;
            let (estimated_truncated_lo, estimated_truncated_hi) = (
                estimated_position_in_output.0.max(0.0) as usize,
                estimated_position_in_output
                    .1
                    .min(output_buffer.len() as f32 - 1.0)
                    .round() as usize,
            );

            // println!(
            //     "[est position = {:?},est samps = {}]->[trunc pos = {:?},trunc samps = {}]",
            //     estimated_position_in_output,
            //     samples_needed,
            //     (estimated_truncated_lo, estimated_truncated_hi),
            //     samples_needed_truncated
            // );
            println!(
                "attack [trunc pos = {:?},trunc samps = {}]",
                (estimated_truncated_lo, estimated_truncated_hi),
                samples_needed_truncated
            );

            //actually pull required pulses from track
            let samples_read = current_track.pull_samples(
                output_buffer
                    .with_buffer(&mut sample_scratch_space[0..samples_needed_truncated * 2]),
            );

            //sound gets added to
            mix_resample_audio_both_2_channels(
                &sample_scratch_space[0..samples_read],
                &mut output_buffer[estimated_truncated_lo..=estimated_truncated_hi],
                &mut [],
            );

            let stream_idx = unsafe {
                let on_intersection_ref =
                    mem::force_ptr_to_ref_mut(running_streams_on_intersection_ptr);
                //it should be save to swap and pop while iterating in reverse
                on_intersection_ref.swap_remove(stream_vec_idx)
            };

            // add the stream index to the linkedlist of non-attack streams
            running_streams.push_rear(stream_idx);

            //assign pointer in the table to a valid linkedlist address/offset because
            //previously it was set to NULL
            let stream_ptr = running_streams.get_rear();
            if let Some(ptr) = running_streams_table.get_mut(&stream_idx) {
                *ptr = stream_ptr;
            }
        }
    }
    fn handle_intersecting_tracks_not_first_time(
        &mut self,
        cursor: MixerCursor,
        mut output_buffer: PCMSlice<f32>,
    ) {
        let track_chart = &mut self.track_chart;
        let running_streams = &mut self.running_streams;
        let sample_scratch_space = &mut self.sample_pull_in;

        //shrinks the cursor delta so i dont read samples too quickly
        //it should okay to be behind by a few milliseconds,
        //i speculate that if I read too quickly, I'll get popping
        const CURSOR_SHINK_DELTA: u128 = 2;

        let cursor_elapsed = cursor.delta.sub(CURSOR_SHINK_DELTA).elapsed_in_ms_fp();
        // println!("elapsed = {}",cursor_elapsed);

        running_streams
            .iter()
            .filter_map(|e| e.get_data())
            .for_each(|&gi| {
                let current_track = &mut track_chart[gi];

                //samples needed to represent `cursor_elaped` time
                let samples_required =
                    current_track.calculate_samples_needed_fp(cursor_elapsed) * 2;

                let samples_required_truncated = samples_required
                    .ceil()
                    .as_int_i128()
                    .max((output_buffer.len()) as i128)
                    as usize;

                let samples_read = current_track.pull_samples(
                    output_buffer.with_buffer(&sample_scratch_space[0..samples_required_truncated]),
                );

                //sound gets added to
                mix_resample_audio_both_2_channels(
                    &sample_scratch_space[0..samples_read],
                    &mut output_buffer[0..samples_required_truncated],
                    &mut [],
                );
            });
    }

    fn search_for_active_tracks(&mut self, cursor: MixerCursor) {
        let track_chart = &mut self.track_chart;
        let running_streams_on_intersection = &mut self.running_streams_on_intersection;
        let running_streams_table = &mut self.running_streams_table;
        track_chart
            .search_interval(&mut TreeIterState::new(), cursor.to_interval_ms())
            .for_each(|(gi, _)| {
                if running_streams_table.contains_key(&gi) == false {
                    running_streams_on_intersection.push(gi);
                    // At this stage it is good enough to know that the stream is being mixed
                    // Inserting (GlobalIndex, Ptr::NULL) into the table tells us the stream is being mixed but
                    // hasn't been added to the running_streams linkedlist yet (its still intersecting)
                    running_streams_table.insert(gi, Ptr::null());

                    // println!("[{:?}] added",gi);
                }
            });
    }

    fn normalize_audio(&mut self, mut output_buffer: PCMSlice<f32>) {
        let mut coef = self.clip_avoidance_dampening_coef;

        let amplitude = output_buffer.iter().map(|&e|e.abs()).reduce(f32::max).unwrap_or(1.0);
        let we_are_not_clipping = amplitude < 1.0; 

        if we_are_not_clipping {
            coef += (1.0-coef)*0.0125;
        }else{
            let inv_amplitude = 1.0/amplitude;
            coef += (inv_amplitude - coef)*0.9;
        }

        for pcm in output_buffer.iter_mut() {
            *pcm = (*pcm*coef).clamp(-1.0, 1.0);
        }

        self.clip_avoidance_dampening_coef = coef;
    }

    fn remove_irrelevent_tracks(&mut self, _cursor: MixerCursor) {
        let track_chart = &mut self.track_chart;
        let running_streams = &mut self.running_streams;
        let running_streams_table = &mut self.running_streams_table;
        let track_removal_stack = &mut self.track_removal_stack;

        for node_ptr in running_streams.node_index_iter() {
            let &gi = running_streams[node_ptr]
                .get_data()
                .expect("should be available");

            if track_chart[gi].time_remaining() < FixedPoint::from(-100i128) {
                //queue track to be removed
                track_removal_stack.push(node_ptr);
                //remove it from the table as well
                running_streams_table.remove(&gi);

                println!("removed track [{:?}]", gi);
            }
        }

        while let Some(node_ptr) = track_removal_stack.pop() {
            running_streams.remove(node_ptr);
        }
    }
}

/// `src` and `dst` are both assumed to be 2 channels interleaved
fn mix_resample_audio_both_2_channels(src: &[f32], dst: &mut [f32], _mix: &mut [f32]) {
    const NUM_CHANNELS: usize = 2;
    let src_sample_count = src.len() / NUM_CHANNELS;
    let dst_sample_count = dst.len() / NUM_CHANNELS;

    let scale_ratio = src_sample_count as f32 / dst_sample_count as f32;
    for dst_i in 0..dst_sample_count {
        let src_i_estimate = dst_i as f32 * scale_ratio;
        let src_i = src_i_estimate as usize;
        let lerp_t = src_i_estimate.fract();
        //interpolate both channels
        for k in 0..NUM_CHANNELS {
            // accumulate destination here
            let dst_index_sub_sample = NUM_CHANNELS * dst_i + k;

            let cur_block = (src_i + 0).max(0);
            let nxt_block = (src_i + 1).min(src_sample_count - 1);
            let cur = src[NUM_CHANNELS * cur_block + k];
            let nxt = src[NUM_CHANNELS * nxt_block + k];

            let old_value = dst[dst_index_sub_sample];
            //interpolated src
            let new_value = (nxt - cur) * lerp_t + cur;
            // let mixed_value = old_value + new_value;
            let mixed_value = (old_value + new_value) ;
            dst[dst_index_sub_sample] = mixed_value;
        }
    }
    // //clip correction
    // if let Some(amplitude) = dst.iter().map(|&e| e.abs()).reduce(f32::max) {
    //     if amplitude > 1.0 {
    //         dst.iter_mut().for_each(|e| *e /= amplitude);
    //     }
    // }
}

/// ## Description
/// returns approximate position of the clipped interval in the output buffer
fn estimate_position_in_buffer_f32(
    cursor: (f32, f32),
    track_interval: (f32, f32),
    mixer_sample_rate: f32,
    mixer_channels: f32,
) -> (f32, f32) {
    let (cursor_lo, _cursor_hi) = cursor;
    let (track_interval_lo, track_interval_hi) = track_interval;

    let minimum_samples = ((track_interval_lo - cursor_lo) * mixer_sample_rate) / 1000.;
    let maximum_samples = ((track_interval_hi - cursor_lo) * mixer_sample_rate) / 1000.;
    (
        minimum_samples * mixer_channels,
        maximum_samples * mixer_channels,
    )
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
