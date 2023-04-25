use std::ops::{Deref, DerefMut};

/// A slice of linear PCM audio data. Its like a normal slice
/// but carrying a little more information about the data
#[derive(Clone, Copy)]
pub struct PCMSlice<'a, T> {
    /// a slice like normal
    planar_pcm: &'a [T],
    /// **additional info**: samples per second
    frequency: u32,
    /// **additional info**: number of channels in the interleaved pcm
    channels: u32,
}

impl<'a, T: Copy + Clone + Default> PCMSlice<'a, T> {
    pub fn new(pcm_buffer: &'a mut [T], frequency: u32, channels: u32) -> Self {
        Self {
            planar_pcm: pcm_buffer,
            frequency,
            channels,
        }
    }

    /// creates new a buffer with same `frequency` and `channels` but with a different slice backing it
    pub fn with_slice<'b>(mut self, pcm_buffer: &'b [T]) -> Self
    where
        'b: 'a,
    {
        self.planar_pcm = pcm_buffer;
        self
    }

    pub fn frequency(&self) -> u32 {
        self.frequency
    }

    pub fn channels(&self) -> u32 {
        self.channels
    }

    pub fn samples(&self) -> usize {
        self.planar_pcm.len()
    }
    pub fn samples_per_channel(&self) -> u64 {
        (self.planar_pcm.len() as u32 / self.channels) as u64
    }

    fn planar_pcm_mut<'b>(&'a self) -> &'b mut [T]
    where
        'a: 'b,
    {
        let ptr = self.planar_pcm as *const [T] as *mut [T];
        unsafe { &mut *ptr }
    }
    /// sets everything in the slice to zero (when T is numeric)
    pub fn set_zero(&mut self) {
        self.iter_mut().for_each(|e| *e = T::default())
    }

    pub fn duration_in_ms_u32(&self) -> u32 {
        const NUM_MILLISECONDS_IN_ONE_SECOND: u32 = 1000;
        (self.planar_pcm.len() as u32 * NUM_MILLISECONDS_IN_ONE_SECOND)
            / (self.channels * self.frequency)
    }

    pub fn duration_in_ms_f32(&self) -> f32 {
        const NUM_MILLISECONDS_IN_ONE_SECOND: f32 = 1000.0;
        (self.planar_pcm.len() as f32 * NUM_MILLISECONDS_IN_ONE_SECOND)
            / (self.channels as f32 * self.frequency as f32)
    }
}

impl<'a, T> Deref for PCMSlice<'a, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.planar_pcm
    }
}

impl<'a, T: Copy + Default> DerefMut for PCMSlice<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.planar_pcm_mut()
    }
}
