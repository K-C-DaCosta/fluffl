use super::*;

use std::f32::consts;

const FRACTIONAL_BITS: i32 = 16;
const FRACTIONAL_MASK: i32 = (1i32 << FRACTIONAL_BITS) - 1;
const FIXED_POINT_FACTOR: f32 = (1i32 << FRACTIONAL_BITS) as f32;
const INV_FIXED_PONT_FACTOR_F64: f32 = 1.0 / FIXED_POINT_FACTOR;

const PI: i32 = (consts::PI * FIXED_POINT_FACTOR) as i32;
const FRACT_PI_2: i32 = (consts::PI * 0.5 * FIXED_POINT_FACTOR) as i32;
const PI_SQUARED: i32 = (consts::PI * consts::PI * FIXED_POINT_FACTOR) as i32;
const PI_2: i32 = (2.0 * consts::PI * FIXED_POINT_FACTOR) as i32;
const INV_PI_2: i32 = (FIXED_POINT_FACTOR / (2.0 * consts::PI)) as i32;

/// ## Description
/// A Custom fixed point utility, for storing a number with factional parts
/// ### Specs
/// - its `Q15.16` or 32 bits with 16 fractional bits
/// - has basic math functions, mod, floor, ceil, min,max,clamp and comparison
/// - can convert from float to Fixedpoint and back  
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Default)]
pub struct FP32 {
    pub data: i32,
}
impl FP32 {
    pub const fn zero() -> Self {
        Self { data: 0 }
    }

    /// does no conversion at all,use From trait for that
    pub fn from_bits(bits: i32) -> Self {
        let data = bits.into();
        Self { data }
    }

    /// slower,because calculations take up an entire register on 64-bit
    /// machines but useful if we need precision
    pub fn div_exact(self, rhs: Self) -> Self {
        let lhs = self.data as i64;
        let rhs = rhs.data as i64;
        let quotient_exact = (lhs << 16) / rhs;
        Self::from_bits(quotient_exact as i32)
    }

    pub fn abs(&self) -> Self {
        Self::from_bits(self.data.abs())
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

    pub const fn pi() -> Self {
        Self { data: PI }
    }

    pub const fn pi_2() -> Self {
        Self { data: PI_2 }
    }

    pub const fn inv_2pi() -> Self {
        Self { data: INV_PI_2 }
    }

    pub const fn pi_fract_2() -> Self {
        Self { data: FRACT_PI_2 }
    }

    pub const fn pi_squared() -> Self {
        Self { data: PI_SQUARED }
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
        self.data = self.data.wrapping_add(rhs.data);
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
        Self::from_bits(((self.data << 7) / rhs.data) << 9)
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
        self.data = (self.data << 7 / rhs.data) << 9;
    }
}
impl DivAssign<i32> for FP32 {
    fn div_assign(&mut self, rhs: i32) {
        *self /= Self::from(rhs);
    }
}
impl Neg for FP32 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::from_bits(-self.data)
    }
}
impl BitAnd<i32> for FP32 {
    type Output = Self;
    fn bitand(self, rhs: i32) -> Self::Output {
        Self::from_bits(self.data & rhs)
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

impl FP32 {
    pub fn cos(self) -> Self {
        (self + Self::pi_fract_2()).sin()
    }
    /// Computes sin quickly by using spline approximations
    /// ## Comments
    /// - pros:
    ///     - no integer divisions
    ///     - branchless
    ///     - vectorizable
    ///     - accurate
    /// - cons:
    ///     - **LOTS** of shifting
    pub fn sin(self) -> Self {
        let inaccurate_spline = |x: Self| {
            const FP4_27_FACTOR: f32 = (1 << 27) as f32;
            const A: i32 = ((4.0 / consts::PI) * FP4_27_FACTOR) as i32;
            const B: i32 = (FP4_27_FACTOR / consts::PI) as i32;
            // because x is between -pi and pi we can convert from Q15.16 to Q3.27
            // which reduces truncation error
            let x = x.data << 11;
            let coef_1_term_0 = 1 << 27;
            let coef_1_term_1 = (x.abs() >> 8) * (B >> 19);
            let coef_0 = (A >> 15) * (x >> 12);
            let coef_1 = coef_1_term_0 - coef_1_term_1;
            let out = (coef_0 >> 15) * (coef_1 >> 12);
            out
        };
        let accurate_spline = |x: Self| {
            const K0F: f32 = 0.865;
            const FP0_30_FACTOR: f32 = (1 << 30) as f32;
            const K0: i32 = (K0F * FP0_30_FACTOR) as i32;
            const K1: i32 = ((1.0 - K0F) * FP0_30_FACTOR) as i32;
            // because the numbers are between -1 and 1 i can switch from Q3.27 to Q0.30 to
            // keep as much accuracy as possible
            let spline = inaccurate_spline(x) << 3;
            let spline_squared = (spline >> 15) * (spline >> 15);
            let spline_cubed = (spline_squared >> 15) * (spline >> 15);
            // better result by interpolating spline and spline_squared
            let accurate_estimation_term_1 = (K0 >> 15) * (spline >> 15);
            let accurate_estimation_term_2 = (K1 >> 15) * (spline_cubed >> 15);
            let accurate_estimation = accurate_estimation_term_1 + accurate_estimation_term_2;
            //convert to FP0.30 to FP15.16
            const ROUND: i32 = 1 << 15;
            let accurate_estimation_fp15_16 = (accurate_estimation + ROUND) >> 14;
            Self::from_bits(accurate_estimation_fp15_16)
        };

        let when_negative = self.data >> 31;
        //division in mod can be optimized out because divisor is constant
        let t = Self::pi()
            - ((Self::from_bits(self.data % Self::pi_2().data)) + (Self::pi_2() & when_negative));
        accurate_spline(t)
    }

    pub fn sqrt(self) -> Self {
        let y = self.data.abs();
        let mut l = 0;
        let mut u = y + 1;

        while u - l > 1 {
            let m = l + ((u - l) >> 1);
            let where_lt = (m - (y / m)) >> 31;
            l = (where_lt & m) | (!where_lt & l);
            u = (!where_lt & m) | (where_lt & u);
        }

        Self::from_bits(l << 8)
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

#[test]
fn trig_tests() {
    const NUM_STEPS: usize = 10000;
    let delta_f64 = 2.0 * 3.14159 / NUM_STEPS as f32;
    let delta_fp64 = FP32::from(delta_f64);
    let mut t_f64 = 0.0f32;
    let mut t_fp64 = FP32::zero();
    let mut max_error = 0.0f32;
    const MAX_TOLERANCE: f32 = 0.1;

    for k in 0..NUM_STEPS {
        let s_f64 = t_f64.sin();
        let s_fp64 = t_fp64.sin();
        let error = (s_f64 - s_fp64.as_f32()).abs();
        max_error = max_error.max(error);

        let meets_tolerance = error < MAX_TOLERANCE;
        if meets_tolerance == false {
            println!(
                "k = {k}\nangle_f64= {},angle_fp64={}\nf64={s_f64},fp64={s_fp64}\nerror = {error}",
                t_f64, t_fp64
            );
            println!("fp64_raw = {}", t_fp64.data);
            panic!("tolerance not met");
        }
        t_f64 += delta_f64;
        t_fp64 += delta_fp64
    }

    println!("max_error = {max_error}");
}
