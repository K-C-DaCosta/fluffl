use crate::{collections::segment_tree::index_types::GlobalIndex, math::FP64};
use std::{fmt::Debug, ops::Deref};

/// Represents an interval
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct Interval {
    pub lo: FP64,
    pub hi: FP64,
}

impl Debug for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.lo, self.hi)
    }
}

impl Interval {
    /// creates and interval: \[`t0`,`t0`+`dt`\]
    pub fn from_point_and_length(t0: FP64, dt: FP64) -> Self {
        Self {
            lo: t0,
            hi: t0 + dt,
        }
    }
    /// creates an interval: \[0,`dt`\]
    pub fn from_length(dt: FP64) -> Self {
        Self {
            lo: FP64::zero(),
            hi: dt,
        }
    }

    /// returns an interval that is within `bounds` parts of interval outside of `bounds` gets clipped away
    pub fn clip_interval(mut self, bounds: Self) -> Self {
        if self.hi > bounds.hi {
            self.hi = bounds.hi;
        }
        if self.lo < bounds.lo {
            self.lo = bounds.lo;
        }
        self
    }

    pub fn inverted(&self) -> bool {
        self.lo > self.hi
    }

    pub fn is_seperating(&self, other_interval: &Self) -> bool {
        let &Self { lo: lo_a, hi: hi_a } = self;

        let &Self { lo: lo_b, hi: hi_b } = other_interval;

        lo_b > hi_a || lo_a > hi_b
    }

    pub fn distance(&self) -> FP64 {
        self.hi - self.lo
    }

    /// divide the interval into equal chunks of count `num_chunks`. returns the `chunk_idx`-th chunk
    pub fn chunk(&self, num_chunks: u64, chunk_idx: usize) -> Self {
        let num_chunks = FP64::from(num_chunks);
        let chunk_idx = FP64::from(chunk_idx as u32);
        let length = self.distance();
        let chunk_length = length / num_chunks;
        let lo = self.lo + chunk_length * chunk_idx;
        Self {
            lo,
            hi: lo + chunk_length,
        }
    }

    pub fn is_overlapping(&self, other_interval: &Self) -> bool {
        !self.is_seperating(other_interval)
    }

    pub fn is_within(&self, t: FP64) -> bool {
        let &Self { lo, hi } = self;
        t >= lo && t <= hi
    }

    #[allow(dead_code)]
    pub fn midpoint(&self) -> FP64 {
        self.lo + (self.hi - self.lo) / FP64::from(2)
    }

    pub fn to_tuple(&self) -> (u128, u128) {
        (self.lo.as_i64() as u128, self.hi.as_i64() as u128)
    }
    pub fn to_tuple_f32(&self) -> (f32, f32) {
        (self.lo.as_f64() as f32, self.hi.as_f64() as f32)
    }
}

impl<T> std::ops::Add<T> for Interval
where
    T: Into<u64> + Copy,
{
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = FP64::from(T::into(rhs));
        Self {
            lo: self.lo + rhs,
            hi: self.hi + rhs,
        }
    }
}

impl std::ops::Add<FP64> for Interval {
    type Output = Self;
    fn add(self, rhs: FP64) -> Self::Output {
        Self {
            lo: self.lo + rhs,
            hi: self.hi + rhs,
        }
    }
}

impl From<(u64, u64)> for Interval {
    fn from((lo, hi): (u64, u64)) -> Self {
        Self {
            lo: FP64::from(lo),
            hi: FP64::from(hi),
        }
    }
}
impl From<(i64, i64)> for Interval {
    fn from((lo, hi): (i64, i64)) -> Self {
        Self {
            lo: FP64::from(lo),
            hi: FP64::from(hi),
        }
    }
}
impl From<(i32, i32)> for Interval {
    fn from((lo, hi): (i32, i32)) -> Self {
        Self {
            lo: FP64::from(lo),
            hi: FP64::from(hi),
        }
    }
}
impl From<(FP64, FP64)> for Interval {
    fn from((lo, hi): (FP64, FP64)) -> Self {
        Self { lo, hi }
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
