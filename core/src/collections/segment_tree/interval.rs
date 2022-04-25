pub use super::*;

/// Represents an interval
#[derive(Copy, Clone, PartialEq, Default)]
pub struct Interval {
    pub lo: u128,
    pub hi: u128,
}

impl Debug for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.lo, self.hi)
    }
}

impl Interval {
    pub fn inverted(&self)->bool{
        self.lo > self.hi
    }
    
    pub fn is_seperating(&self, other_interval: &Self) -> bool {
        let &Self { lo: lo_a, hi: hi_a } = self;

        let &Self { lo: lo_b, hi: hi_b } = other_interval;

        lo_b > hi_a || lo_a > hi_b
    }

    pub fn distance(&self) -> u128 {
        self.hi - self.lo
    }

    /// divide the interval into equal chunks then get `chunk_idx`
    pub fn chunk(&self, num_chunks: u128, chunk_idx: usize) -> Self {
        let chunk_idx = chunk_idx as u128;
        let length = self.distance();
        let chunk_lenth = length / num_chunks;
        let lo = self.lo + chunk_lenth * chunk_idx;
        Self {
            lo,
            hi: lo + chunk_lenth,
        }
    }

    pub fn is_overlapping(&self, other_interval: &Self) -> bool {
        !self.is_seperating(other_interval)
    }

    pub fn is_within(&self, t: u128) -> bool {
        let &Self { lo, hi } = self;

        t >= lo && t <= hi
    }
    #[allow(dead_code)]
    pub fn midpoint(&self) -> u128 {
        self.lo + (self.hi - self.lo) / 2
    }
}

impl From<(u128, u128)> for Interval {
    fn from((start_time, stop_time): (u128, u128)) -> Self {
        Self {
            lo: start_time,
            hi: stop_time,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GlobalInterval<V> {
    pub interval: Interval,
    pub data: V,
}

impl<V> Deref for GlobalInterval<V> {
    type Target = Interval;
    fn deref(&self) -> &Self::Target {
        &self.interval
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TreeInterval {
    pub clipped_interval: Interval,
    pub global_idx: GlobalIndex,
}

impl Deref for TreeInterval {
    type Target = Interval;
    fn deref(&self) -> &Self::Target {
        &self.clipped_interval
    }
}
