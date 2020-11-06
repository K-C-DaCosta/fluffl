pub mod mp3; 
pub mod music_player; 
pub mod ogg; 
pub mod wav;

/// A trait that converts a type into a list of `AudioSamples`
pub trait PcmConverter<T> {
    fn samples(self) -> Vec<AudioSample<T>>;
}

#[derive(Copy, Clone)]
/// A sample just a 2-tuple of type `T`
pub struct AudioSample<T> {
    pub channel: [T; 2],
}
/// An audio buffer is used for audio pla
pub trait AudioBuffer<T: Copy> {
    fn read(&mut self, out: &mut [AudioSample<T>]) -> usize;
    fn seek_to_start(&mut self);
}

impl<T> From<[T; 2]> for AudioSample<T> {
    fn from(list: [T; 2]) -> Self {
        Self { channel: list }
    }
}