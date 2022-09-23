use super::*;

mod constants;
mod trig;

pub use self::{constants::*, trig::*};

pub trait HasBits {
    fn to_bits(self) -> u64;
    fn from_bits(bits: u64) -> Self;
}

impl HasBits for f32 {
    fn from_bits(bits: u64) -> Self {
        f32::from_bits(bits as u32)
    }

    fn to_bits(self) -> u64 {
        f32::to_bits(self) as u64
    }
}

impl HasBits for f64 {
    fn from_bits(bits: u64) -> Self {
        f64::from_bits(bits)
    }

    fn to_bits(self) -> u64 {
        f64::to_bits(self)
    }
}

impl HasBits for FP64 {
    fn from_bits(bits: u64) -> Self {
        FP64::from_bits(bits as i64)
    }

    fn to_bits(self) -> u64 {
        self.bits()
    }
}
