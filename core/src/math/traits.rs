use super::*; 

pub trait HasConsts {
    fn zero() -> Self;
    fn one() -> Self;
}

impl HasConsts for f32 {
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
}
impl HasConsts for f64 {
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
}

impl HasConsts for FP32 {
    fn zero() -> Self {
        Self::zero()
    }
    fn one() -> Self {
        Self::from(1)
    }
}
impl HasConsts for FP64 {
    fn zero() -> Self {
        Self::zero()
    }
    fn one() -> Self {
        Self::from(1)
    }
}