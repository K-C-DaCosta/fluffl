use std::{fmt::Display, ops::*};

const FRACTIONAL_BITS: i64 = 32;
const FRACTIONAL_MASK: i128 = (1i128 << FRACTIONAL_BITS) - 1;
const FIXED_POINT_FACTOR: f64 = (1i64 << 32) as f64;
const INV_FIXED_PONT_FACTOR_F64: f64 = 1.0 / FIXED_POINT_FACTOR;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Fixed96_32 {
    data: i128,
}
impl Fixed96_32 {
    /// does no conversion at all,use From trait for that
    pub fn from_bits<T: Into<i128>>(bits: T) -> Self {
        let data = bits.into();
        Self { data }
    }

    pub fn floor(&self) -> Self {
        Self::from_bits(self.data & !(FRACTIONAL_MASK))
    }

    pub fn fract(&self) -> Self {
        Self::from_bits(self.data & (FRACTIONAL_MASK))
    }

    pub fn as_int_i128(&self) -> i128 {
        self.data >> FRACTIONAL_BITS
    }

    pub fn as_f64(&self) -> f64 {
        self.data as f64 * INV_FIXED_PONT_FACTOR_F64
    }
}

impl Add for Fixed96_32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.data + rhs.data)
    }
}
impl AddAssign for Fixed96_32 {
    fn add_assign(&mut self, rhs: Self) {
        self.data += rhs.data;
    }
}
impl Sub for Fixed96_32 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.data - rhs.data)
    }
}
impl SubAssign for Fixed96_32 {
    fn sub_assign(&mut self, rhs: Self) {
        self.data -= rhs.data
    }
}
impl Mul for Fixed96_32 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_bits((self.data >> 16) * (rhs.data >> 16))
    }
}
impl MulAssign for Fixed96_32 {
    fn mul_assign(&mut self, rhs: Self) {
        self.data = (self.data >> 10) * (rhs.data >> 22)
    }
}

impl Div for Fixed96_32 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::from_bits(((self.data << 16) / rhs.data) << 16)
    }
}
impl DivAssign for Fixed96_32 {
    fn div_assign(&mut self, rhs: Self) {
        self.data = (self.data << 16 / rhs.data) << 16;
    }
}

impl Display for Fixed96_32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_f64())
    }
}

/*
 CONVERSION CRAP HERE
*/
impl From<i32> for Fixed96_32 {
    fn from(num: i32) -> Self {
        let num: i128 = num as i128;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}
impl From<i64> for Fixed96_32 {
    fn from(num: i64) -> Self {
        let num: i128 = num as i128;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}
impl From<i128> for Fixed96_32 {
    fn from(num: i128) -> Self {
        let num: i128 = num as i128;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}
impl From<f32> for Fixed96_32 {
    fn from(num: f32) -> Self {
        Fixed96_32::from_bits((num * FIXED_POINT_FACTOR as f32) as i128)
    }
}
impl From<f64> for Fixed96_32 {
    fn from(num: f64) -> Self {
        Fixed96_32::from_bits((num * FIXED_POINT_FACTOR) as i128)
    }
}


#[test]
fn test_number() {
    // let x = Fixed96_32::from(123);
    let val = Fixed96_32::from_bits(1i128 << 33);
    println!("x = {}, data = {} ", val, val.data);
}
