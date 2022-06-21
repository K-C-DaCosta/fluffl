use std::{
    fmt::{Debug, Display},
    ops::*,
};

const FRACTIONAL_BITS: i64 = 16;
const FRACTIONAL_MASK: i64 = (1i64 << FRACTIONAL_BITS) - 1;
const FIXED_POINT_FACTOR: f64 = (1i64 << 32) as f64;
const INV_FIXED_PONT_FACTOR_F64: f64 = 1.0 / FIXED_POINT_FACTOR;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Default)]
pub struct FixedPoint {
    data: i64,
}
impl FixedPoint {
    pub const fn zero() -> Self {
        Self { data: 0 }
    }

    /// does no conversion at all,use From trait for that
    pub fn from_bits<T: Into<i64>>(bits: T) -> Self {
        let data = bits.into();
        Self { data }
    }

    pub fn floor(&self) -> Self {
        Self::from_bits(self.data & !(FRACTIONAL_MASK))
    }
    pub fn ceil(&self) -> Self {
        (*self * (-1)).floor() * -1
    }

    pub fn fract(&self) -> Self {
        Self::from_bits(self.data & (FRACTIONAL_MASK))
    }

    /// computes x % 2^`exp`
    pub fn fast_mod(&self, exp: u8) -> Self {
        let mask = (1 << (exp + FRACTIONAL_BITS as u8)) - 1;
        Self::from_bits(self.data & mask)
    }

    pub fn as_int_i128(&self) -> i128 {
        (self.data >> FRACTIONAL_BITS) as i128
    }

    pub fn as_f64(&self) -> f64 {
        self.data as f64 * INV_FIXED_PONT_FACTOR_F64
    }
}

impl Add for FixedPoint {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.data + rhs.data)
    }
}

impl AddAssign for FixedPoint {
    fn add_assign(&mut self, rhs: Self) {
        self.data += rhs.data;
    }
}

impl Add<i32> for FixedPoint {
    type Output = Self;
    fn add(self, rhs: i32) -> Self::Output {
        self + Self::from(rhs)
    }
}

impl Sub for FixedPoint {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.data - rhs.data)
    }
}
impl SubAssign for FixedPoint {
    fn sub_assign(&mut self, rhs: Self) {
        self.data -= rhs.data
    }
}

impl Mul for FixedPoint {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_bits((self.data >> 8) * (rhs.data >> 8))
    }
}

impl MulAssign for FixedPoint {
    fn mul_assign(&mut self, rhs: Self) {
        self.data = (self.data >> 7) * (rhs.data >> 9)
    }
}

impl Mul<i32> for FixedPoint {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        self * FixedPoint::from(rhs)
    }
}

impl Div for FixedPoint {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::from_bits(((self.data << 8) / rhs.data) << 8)
    }
}
impl DivAssign for FixedPoint {
    fn div_assign(&mut self, rhs: Self) {
        self.data = (self.data << 8 / rhs.data) << 8;
    }
}

impl Shr<u8> for FixedPoint {
    type Output = Self;
    fn shr(self, rhs: u8) -> Self::Output {
        Self::from_bits(self.data >> rhs)
    }
}
impl Shl<u8> for FixedPoint {
    type Output = Self;
    fn shl(self, rhs: u8) -> Self::Output {
        Self::from_bits(self.data << rhs)
    }
}

impl Display for FixedPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_f64())
    }
}
impl Debug for FixedPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_f64())
    }
}

/*
 CONVERSION CRAP HERE
*/
impl From<i32> for FixedPoint {
    fn from(num: i32) -> Self {
        let num = num as i64;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}
impl From<i64> for FixedPoint {
    fn from(num: i64) -> Self {
        let num = num as i64;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}
impl From<i128> for FixedPoint {
    fn from(num: i128) -> Self {
        let num = num as i64;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}

impl From<u32> for FixedPoint {
    fn from(num: u32) -> Self {
        let num  = num as i64;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}
impl From<u64> for FixedPoint {
    fn from(num: u64) -> Self {
        let num = num as i64;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}


impl From<f32> for FixedPoint {
    fn from(num: f32) -> Self {
        FixedPoint::from_bits((num * FIXED_POINT_FACTOR as f32) as i64)
    }
}
impl From<f64> for FixedPoint {
    fn from(num: f64) -> Self {
        FixedPoint::from_bits((num * FIXED_POINT_FACTOR) as i64)
    }
}

#[test]
fn consersion_tests() {
    let val = FixedPoint::from(25);
    assert_eq!(25, val.as_int_i128());

    let val = FixedPoint::from(-1);
    assert_eq!(-1, val.as_int_i128());

    let val = FixedPoint::from(-10);
    assert_eq!(-10, val.as_int_i128());

    //exhaustive test
    for k in -900_000_000..=900_000_000 {
        // println!("k ={}",k );
        let val = FixedPoint::from(k);

        assert_eq!(k, val.as_int_i128(), "integer shotgun test failed");
    }
}

#[test]
fn fast_mod_tests() {
    let normal_mod = (0..500_000i128).map(|k| k % 16).collect::<Vec<_>>();
    let fixed_mod = (0..500_000i128)
        .map(|k| FixedPoint::from(k).fast_mod(4).as_int_i128())
        .collect::<Vec<_>>();

    assert_eq!(&normal_mod, &fixed_mod);
}
