pub struct RingBuffer<const N:usize,T>{
    memory:[T;N],
    front:u32,
    rear:u32,
    len:u32,
}



/// Represents a time interval in milliseconds
#[derive(Copy,Clone)]
pub struct Interval {
    pub start_time: u128,
    pub stop_time: u128,
}

impl Interval {
    pub fn is_overlapping(&self, other_interval: &Self) -> bool {
        let &Self {
            start_time: lo_a,
            stop_time: hi_a,
        } = self;

        let &Self {
            start_time: lo_b,
            stop_time: hi_b,
        } = other_interval;

        lo_b <= hi_a || lo_a <= hi_b 
    }
    pub fn is_within(&self, t: u128) -> bool {
        let &Self {
            start_time: lo,
            stop_time: hi,
        } = self;

        t >= lo && t <= hi
    }
}

pub trait HasAudioStream {
    fn frequency(&self) -> u64;
    fn channels(&self) -> u64;
    fn interval(&self) -> Interval;
    
    ///returns time local to interval
    fn attack_time(&self) -> u128;
    
    ///returns time local to interval
    fn release_time(&self) -> u128;

    fn is_dead(&self) -> bool;
    
    fn get_input_buffer(&mut self)->&mut RingBuffer<512,f32>;
    fn get_output_buffer(&mut self)->&mut RingBuffer<512,f32>;
}
