use crate::collections::{
    linked_list::LinkedList,
    segment_tree::{index_types::GlobalIndex, CircularSegmentTree, GlobalInterval, Interval},
};
use std::ops::Index;

pub mod streams;

#[derive(Clone, Copy)]
pub struct PCMBuffer<'a> {
    pub planar_samples: &'a [f32],
    /// samples per second
    pub frequency: u32,
    pub channels: u32,
}
impl<'a> PCMBuffer<'a> {
    pub fn new(samples: &'a mut [f32], frequency: u32, channels: u32) -> Self {
        Self {
            planar_samples: samples,
            frequency,
            channels,
        }
    }
    pub fn samples_mut(&'a self) -> &'a mut [f32]
    {
        let ptr = self.planar_samples as *const [f32] as *mut [f32];
        unsafe { &mut *ptr }
    }
    pub fn duration_in_ms(&self) -> u32 {
        let samples_per_ms = self.frequency / 1000;
        (self.planar_samples.len() as u32 / self.channels) / samples_per_ms
    }
}

#[derive(Copy, Clone)]
pub struct StreamState {
    /// in milliseconds
    pub local_time: u128,
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
impl StreamState {
    pub fn is_dead(&self) -> bool {
        self.global_interval.distance() >= self.local_time
    }
}

pub trait HasAudioStream {
    fn stream_state(&self) -> &StreamState;
    fn stream_state_mut(&mut self) -> &mut StreamState;
    /// will advance local time
    fn pull_samples(&mut self, samples: &mut [f32]);
}

/// mixes sounds together
pub struct Mixer {
    global_t: u128,
    sample_pull_in: Vec<f32>,
    track_list: CircularSegmentTree<Box<dyn HasAudioStream>>,
    current_streams: LinkedList<GlobalIndex>,
}
impl Mixer {
    pub fn new() -> Self {
        Self {
            global_t: 0,
            sample_pull_in: Vec::with_capacity(4096),
            current_streams: LinkedList::new(),
            track_list: CircularSegmentTree::new(14, 65536),
        }
    }

    pub fn mix_audio<'a>(&mut self, mut output_buffer: PCMBuffer<'a>)
    {
        let duration = output_buffer.duration_in_ms();
        let buffer = output_buffer.samples_mut();




        //update t
        self.global_t += duration as u128;
    }
}

pub struct TrackList {
    sound_track: Vec<Interval>,
}
impl TrackList {
    pub fn with_track(mut sound_track: Vec<Interval>) -> Self {
        sound_track.sort_by_key(|&i| i.lo);
        Self { sound_track }
    }

    /// fetches any track that fits within the time coordinate
    /// ### Notes:
    /// - complexity: `O(log(n))`
    pub fn get_any_track(&self, time: u128) -> Option<usize> {
        let mut lo = 0;
        let mut hi = self.sound_track.len() - 1;
        let mut left_most_interval = None;

        //binary search for the first interval that fits within the `time:u128` query
        //this is used as an initial starting point for the left-most search
        while lo <= hi {
            let mid = (hi - lo) / 2 + lo;
            let item = self.sound_track[mid];
            if item.is_within(time) {
                left_most_interval = Some(mid);
                break;
            } else if time < item.lo {
                //take left subarray
                hi = mid - 1;
            } else {
                //take right subarray
                lo = mid + 1;
            }
        }

        left_most_interval
    }

    /// fetches earliest track that intersects it
    /// ### Notes:
    /// - complexity: `O(log(n))`
    pub fn get_earliest_track(&self, time: u128) -> Option<usize> {
        let sound_track = &self.sound_track;
        let left_most_interval = self.get_any_track(time);

        // after the binary search you aren't nececiarily going to get the leftmost track that fits within `time`
        // so you have to do bisection-like iterations to get there quick
        if let Some(hi) = left_most_interval {
            let mut hi = hi;
            let mut lo = 0;
            loop {
                let mid = (hi - lo) / 2 + lo;
                let hi_in = sound_track[hi].is_within(time);
                let lo_in = sound_track[lo].is_within(time);
                let mid_in = sound_track[mid].is_within(time);
                if hi - lo <= 1 {
                    if lo_in {
                        return Some(lo);
                    }
                    if hi_in {
                        return Some(hi);
                    }
                } else if hi_in == lo_in {
                    return Some(lo);
                } else if mid_in != hi_in {
                    lo = mid;
                } else if lo_in != mid_in {
                    hi = mid;
                }
            }
        }

        left_most_interval
    }
}
impl Index<usize> for TrackList {
    type Output = Interval;
    fn index(&self, index: usize) -> &Self::Output {
        &self.sound_track[index]
    }
}
