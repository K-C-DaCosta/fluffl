pub mod mp3;
pub mod music_player;
pub mod ogg;
pub mod wav;

/// A trait that converts a type into a list of `AudioSamples`
pub trait PcmConverter<T> {
    fn samples(self) -> Vec<AudioSample<T>>;
}

#[derive(Copy, Clone)]
/// A sample is just a 2-tuple of type `T`
pub struct AudioSample<T> {
    pub channel: [T; 2],
}
/// Structs that implement the `AudioBuffer` trait contain encoded **PCM**. \
/// This is used to fetch decoded PCM within **Ogg** and **Mp3** files
pub trait AudioBuffer<T: Copy> {
    /// Decode `usize` samples and write it into the `out` slice
    fn read(&mut self, out: &mut [AudioSample<T>]) -> usize;
    /// Just sets a pointer back to the beggining of the track
    fn seek_to_start(&mut self);
}

impl<T> From<[T; 2]> for AudioSample<T> {
    fn from(list: [T; 2]) -> Self {
        Self { channel: list }
    }
}
