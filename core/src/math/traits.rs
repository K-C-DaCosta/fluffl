use super::*;

pub trait HasTrig: Copy {
    fn sin(self) -> Self;
    fn cos(self) -> Self;
}

#[rustfmt::skip]
impl HasTrig for f32{
    fn cos(self) -> Self { f32::cos(self) }
    fn sin(self) -> Self { f32::sin(self) }
}

#[rustfmt::skip]
impl HasTrig for f64{
    fn cos(self) -> Self { f64::cos(self) }
    fn sin(self) -> Self { f64::sin(self) }
}

pub trait HasScalar {
    fn zero() -> Self;
    fn one() -> Self;
    fn from_i32(val: i32) -> Self;
}

#[rustfmt::skip]
impl HasScalar for isize {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn from_i32(val: i32) -> Self { val as Self }
}
#[rustfmt::skip]
impl HasScalar for usize {
    fn zero() -> Self{ 0 }
    fn one() -> Self { 1 }
    fn from_i32(val: i32) -> Self { val.max(0) as Self }
}
#[rustfmt::skip]
impl HasScalar for f32 {
    fn zero() -> Self { 0.0 }
    fn one() -> Self { 1.0 }
    fn from_i32(val: i32) -> Self { val as Self }
}
#[rustfmt::skip]
impl HasScalar for f64 {
    fn zero() -> Self { 0.0 }
    fn one() -> Self { 1.0 }
    fn from_i32(val: i32) -> Self { val as Self }
}

#[rustfmt::skip]
impl HasScalar for FP32 {
    fn zero() -> Self{ Self::zero() }
    fn one() -> Self { Self::from(1) }
    fn from_i32(val: i32) -> Self { Self::from(val) }
}

#[rustfmt::skip]
impl HasScalar for FP64 {
    fn zero() -> Self { Self::zero()}
    fn one() -> Self { Self::from(1)}
    fn from_i32(val: i32) -> Self {Self::from(val)}
}
