use super::*;

pub trait HasScalar {
    fn zero() -> Self;
    fn one() -> Self;
    fn from_i32(val: i32) -> Self;
}

impl HasScalar for isize {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
    fn from_i32(val: i32) -> Self {
        val as Self
    }
}

impl HasScalar for usize {
    fn zero() -> Self {
        0
    }
    fn one() -> Self {
        1
    }
    fn from_i32(val: i32) -> Self {
        val.max(0) as Self
    }
}
impl HasScalar for f32 {
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
    fn from_i32(val: i32) -> Self {
        val as Self
    }
}
impl HasScalar for f64 {
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
    fn from_i32(val: i32) -> Self {
        val as Self
    }
}

impl HasScalar for FP32 {
    fn zero() -> Self {
        Self::zero()
    }
    fn one() -> Self {
        Self::from(1)
    }
    fn from_i32(val: i32) -> Self {
        Self::from(val)
    }
}
impl HasScalar for FP64 {
    fn zero() -> Self {
        Self::zero()
    }
    fn one() -> Self {
        Self::from(1)
    }
    fn from_i32(val: i32) -> Self {
        Self::from(val)
    }
}
