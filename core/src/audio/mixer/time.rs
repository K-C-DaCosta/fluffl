use crate::math::FixedPoint;

/// tracks time by counting samples processed, and using frequency to calculate time to arbitrary precisions
#[derive(Copy, Clone,Debug)]
pub struct SampleTime {
    samples_count: u128,
    sample_rate: u32,
}

impl SampleTime {
    pub fn new() -> Self {
        Self {
            samples_count: 0,
            sample_rate: 44_100,
        }
    }
    pub fn with_sample_count(mut self, sample_count: u128) -> Self {
        self.samples_count = sample_count;
        self
    }
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }
    pub fn elapsed_in_ms_fp(&self) -> FixedPoint {
        FixedPoint::from(self.samples_count * 1000) / FixedPoint::from(self.sample_rate)
    }
    pub fn elasped_in_ms_f32(&self) -> f32 {
        (self.samples_count as f32 * 1000.0) / self.sample_rate as f32
    }

    pub fn elapsed_in_ms_u128(&self) -> u128 {
        (self.samples_count * 1000) / self.sample_rate as u128
    }

    pub fn elapsed_in_sec_f64(&self) -> f64 {
        let sample_count = self.samples_count as f64;
        let sample_rate = self.sample_rate as f64;
        sample_count / sample_rate
    }

    pub fn sample_delta_in_sec_f64(&self) -> f64 {
        1.0 / self.sample_rate as f64
    }
    
    

    pub fn increment(&mut self, num_samples: u128) {
        self.samples_count += num_samples;
    }
    
    /// creates a new time with samples decremented 
    pub fn sub(mut self,offset:u128)->Self{
        self.samples_count-=offset;
        self        
    }
    
    /// ## Description
    /// sums two times together
    /// ### Comments  
    /// sample rates of two times is expected to be the same but will take max if they arent
    pub fn sum(&self, other: &SampleTime) -> Self {
        let samples_count = self.samples_count + other.samples_count;
        let sample_rate = self.sample_rate.max(other.sample_rate);
        Self {
            samples_count,
            sample_rate,
        }
    }
}
