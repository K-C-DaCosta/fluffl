use crate::math::FP64;

/// tracks time by counting samples processed, and using frequency to calculate time to arbitrary precisions.\
/// its basically the elapsed time stored as a rational number
#[derive(Copy, Clone, Debug)]
pub struct SampleTime {
    samples_count: u64,
    sample_rate: u32,
}

impl SampleTime {
    pub fn new() -> Self {
        Self {
            samples_count: 0,
            sample_rate: 44_100,
        }
    }

    pub fn samps(&self)->u64{
        self.samples_count
    }

    pub fn with_sample_count(mut self, sample_count: u64) -> Self {
        self.samples_count = sample_count;
        self
    }

    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    pub fn elapsed_in_ms_fp(&self) -> FP64 {
        FP64::from(self.samples_count * 1000) / FP64::from(self.sample_rate)
    }

    pub fn elapsed_in_ms_f32(&self) -> f32 {
        (self.samples_count as f32 * 1000.0) / self.sample_rate as f32
    }

    pub fn elapsed_in_ms_u64(&self) -> u64 {
        (self.samples_count * 1000) / self.sample_rate as u64
    }

    pub fn elapsed_in_sec_f64(&self) -> f64 {
        let sample_count = self.samples_count as f64;
        let sample_rate = self.sample_rate as f64;
        sample_count / sample_rate
    }

    pub fn sample_delta_in_sec_f64(&self) -> f64 {
        1.0 / self.sample_rate as f64
    }

    pub fn increment(&mut self, num_samples: u64) {
        self.samples_count += num_samples;
    }

    pub fn set_samps(&mut self,samps:u64){
        self.samples_count = samps;
    }

    /// creates a new time with samples decremented
    pub fn sub_samps(mut self, offset: u64) -> Self {
        self.samples_count -= offset;
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

    /// ## Description
    /// subs two times together
    /// ### Comments  
    /// sample rates of two times is expected to be the same but will take max if they arent
    pub fn sub(&self, other: &SampleTime) -> Self {
        let samples_count = self.samples_count - other.samples_count;
        let sample_rate = self.sample_rate.max(other.sample_rate);
        Self {
            samples_count,
            sample_rate,
        }
    }

    /// Computes a new SampleTime,with info from `self`, given `dt` in milliseconds
    pub fn from_time_in_ms_fp(&self, dt: FP64) -> Self {
        let sample_rate = self.sample_rate;
        let sample_count = (FP64::from(self.sample_rate) * dt) / FP64::from(1000);
        Self {
            samples_count: sample_count.as_i64() as u64,
            sample_rate,
        }
    }
    
    /// Computes a new SampleTime,with info from `self`, given `dt` in milliseconds
    pub fn from_time_in_ms_u64(&self, dt: u64) -> Self {
        let sample_rate = self.sample_rate;
        let sample_count = FP64::from(self.sample_rate as u64 * dt) / FP64::from(1000);
        Self {
            samples_count: sample_count.as_i64() as u64,
            sample_rate,
        }
    }
}
