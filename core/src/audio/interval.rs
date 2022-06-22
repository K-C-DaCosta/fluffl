use crate::{
    collections::{fixed_stack::FixedStack, segment_tree::index_types::GlobalIndex},
    math::FixedPoint,
};
use std::{fmt::Debug, ops::Deref};

/// Represents an interval
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct Interval {
    pub lo: FixedPoint,
    pub hi: FixedPoint,
}

impl Debug for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.lo, self.hi)
    }
}

impl Interval {
    pub fn from_point_and_length(t0: FixedPoint, dt: FixedPoint) -> Self {
        Self {
            lo: t0,
            hi: t0 + dt,
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

    pub fn distance(&self) -> FixedPoint {
        self.hi - self.lo
    }

    /// divide the interval into equal chunks of count `num_chunks`. returns the `chunk_idx`-th chunk
    pub fn chunk(&self, num_chunks: u64, chunk_idx: usize) -> Self {
        let num_chunks = FixedPoint::from(num_chunks);
        let chunk_idx = FixedPoint::from(chunk_idx as u32);
        let length = self.distance();
        let chunk_length = length / num_chunks;
        let lo = self.lo + chunk_length * chunk_idx;
        Self {
            lo,
            hi: lo + chunk_length ,
        }
    }

    pub fn is_overlapping(&self, other_interval: &Self) -> bool {
        !self.is_seperating(other_interval)
    }

    pub fn is_within(&self, t: FixedPoint) -> bool {
        let &Self { lo, hi } = self;
        t >= lo && t <= hi
    }

    #[allow(dead_code)]
    pub fn midpoint(&self) -> FixedPoint {
        self.lo + (self.hi - self.lo) / FixedPoint::from(2)
    }

    pub fn to_tuple(&self) -> (u128, u128) {
        (self.lo.as_int_i64() as u128, self.hi.as_int_i64() as u128)
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
        let rhs = FixedPoint::from(T::into(rhs));
        Self {
            lo: self.lo + rhs,
            hi: self.hi + rhs,
        }
    }
}

impl From<(u64, u64)> for Interval {
    fn from((lo, hi): (u64, u64)) -> Self {
        Self {
            lo: FixedPoint::from(lo),
            hi: FixedPoint::from(hi),
        }
    }
}
impl From<(i64, i64)> for Interval {
    fn from((lo, hi): (i64, i64)) -> Self {
        Self {
            lo: FixedPoint::from(lo),
            hi: FixedPoint::from(hi),
        }
    }
}
impl From<(i32, i32)> for Interval {
    fn from((lo, hi): (i32, i32)) -> Self {
        Self {
            lo: FixedPoint::from(lo),
            hi: FixedPoint::from(hi),
        }
    }
}
impl From<(FixedPoint, FixedPoint)> for Interval {
    fn from((lo, hi): (FixedPoint, FixedPoint)) -> Self {
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
