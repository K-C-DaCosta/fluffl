use super::*;


const FRACTIONAL_BITS: i32 = 16;
const FRACTIONAL_MASK: i32 = (1i32 << FRACTIONAL_BITS) - 1;
const FIXED_POINT_FACTOR: f32 = (1i32 << FRACTIONAL_BITS) as f32;
const INV_FIXED_PONT_FACTOR_F64: f32 = 1.0 / FIXED_POINT_FACTOR;

/// ## Description
/// A Custom fixed point utility, for storing a number with factional parts
/// ### Specs
/// - 32 bits with 16 fractional bits
/// - has basic math functions, mod, floor, ceil, min,max,clamp and comparison
/// - can convert from float to Fixedpoint and back  
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Default)]
pub struct FP32 {
    data: i32,
}
impl FP32 {
    pub const fn zero() -> Self {
        Self { data: 0 }
    }

    /// does no conversion at all,use From trait for that
    pub fn from_bits<T: Into<i32>>(bits: T) -> Self {
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

    pub fn as_i32(&self) -> i32 {
        (self.data >> FRACTIONAL_BITS) as i32
    }

    pub fn as_f32(&self) -> f32 {
        self.data as f32 * INV_FIXED_PONT_FACTOR_F64
    }
}

impl Add for FP32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.data + rhs.data)
    }
}

impl AddAssign for FP32 {
    fn add_assign(&mut self, rhs: Self) {
        self.data += rhs.data;
    }
}

impl Add<i32> for FP32 {
    type Output = Self;
    fn add(self, rhs: i32) -> Self::Output {
        self + Self::from(rhs)
    }
}

impl Sub for FP32 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.data - rhs.data)
    }
}

impl Sub<i32> for FP32 {
    type Output = Self;
    fn sub(self, rhs: i32) -> Self::Output {
        self - Self::from(rhs)
    }
}

impl SubAssign for FP32 {
    fn sub_assign(&mut self, rhs: Self) {
        self.data -= rhs.data
    }
}

impl Mul for FP32 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_bits((self.data >> 8) * (rhs.data >> 8))
    }
}
impl Mul<i32> for FP32 {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        self * Self::from(rhs)
    }
}

impl MulAssign for FP32 {
    fn mul_assign(&mut self, rhs: Self) {
        self.data = (self.data >> 8) * (rhs.data >> 8)
    }
}

impl MulAssign<i32> for FP32 {
    fn mul_assign(&mut self, rhs: i32) {
        *self *= Self::from(rhs)
    }
}

impl Div for FP32 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::from_bits(((self.data << 8) / rhs.data) << 8)
    }
}
impl Div<i32> for FP32 {
    type Output = Self;
    fn div(self, rhs: i32) -> Self::Output {
        self / Self::from(rhs)
    }
}
impl DivAssign for FP32 {
    fn div_assign(&mut self, rhs: Self) {
        self.data = (self.data << 8 / rhs.data) << 8;
    }
}
impl DivAssign<i32> for FP32 {
    fn div_assign(&mut self, rhs: i32) {
        *self /= Self::from(rhs);
    }
}

impl Shr<u8> for FP32 {
    type Output = Self;
    fn shr(self, rhs: u8) -> Self::Output {
        Self::from_bits(self.data >> rhs)
    }
}
impl Shl<u8> for FP32 {
    type Output = Self;
    fn shl(self, rhs: u8) -> Self::Output {
        Self::from_bits(self.data << rhs)
    }
}

impl Display for FP32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_f32())
    }
}
impl Debug for FP32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_f32())
    }
}

/*
 CONVERSION CRAP HERE
*/
impl From<i32> for FP32 {
    fn from(num: i32) -> Self {
        let num = num as i32;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}

impl From<u32> for FP32 {
    fn from(num: u32) -> Self {
        let num = num as i32;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}
impl From<u64> for FP32 {
    fn from(num: u64) -> Self {
        let num = num as i32;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}

impl From<f32> for FP32 {
    fn from(num: f32) -> Self {
        FP32::from_bits((num * FIXED_POINT_FACTOR as f32) as i32)
    }
}
impl From<f64> for FP32 {
    fn from(num: f64) -> Self {
        FP32::from_bits((num * FIXED_POINT_FACTOR as f64) as i32)
    }
}



    


#[test]
fn conversion_tests() {
    let val = FP32::from(25);
    assert_eq!(25, val.as_i32());

    let val = FP32::from(-1);
    assert_eq!(-1, val.as_i32());

    let val = FP32::from(-10);
    assert_eq!(-10, val.as_i32());

    //exhaustive test
    for k in -32768..=32767 {
        // println!("k ={}",k );
        let val = FP32::from(k);

        assert_eq!(k, val.as_i32(), "integer shotgun test failed");
    }
}

#[test]
fn fast_mod_tests() {
    let normal_mod = (0..10_000i32).map(|k| k % 16).collect::<Vec<_>>();
    let fixed_mod = (0..10_000i32)
        .map(|k| FP32::from(k).fast_mod(4).as_i32())
        .collect::<Vec<_>>();

    assert_eq!(&normal_mod, &fixed_mod);
}