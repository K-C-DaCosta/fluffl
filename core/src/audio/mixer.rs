use crate::collections::segment_tree::Interval;
use std::ops::Index;

// eventually i will use this to feed mixed audio to the audio backend
pub struct RingBuffer<const N: usize, T> {
    memory: [T; N],
    front: u32,
    rear: u32,
    len: u32,
}

pub trait HasAudioStream {
    fn frequency(&self) -> u64;
    fn channels(&self) -> u64;
    fn interval(&self) -> Interval;
    fn attack_time(&self) -> u128;
    fn release_time(&self) -> u128;
    fn is_dead(&self) -> bool;
    fn output_buffer(&self) -> &RingBuffer<512, f32>;
    fn output_buffer_mut(&mut self) -> &mut RingBuffer<512, f32>;
}

/// mixes sounds together
pub struct Mixer {
    sound_track: Vec<Interval>,
}
impl Mixer {
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
impl Index<usize> for Mixer {
    type Output = Interval;
    fn index(&self, index: usize) -> &Self::Output {
        &self.sound_track[index]
    }
}
