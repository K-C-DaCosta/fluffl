use super::*;

use std::f64::consts;

const FRACTIONAL_BITS: i64 = 16;
const FRACTIONAL_MASK: i64 = (1i64 << FRACTIONAL_BITS) - 1;
const FIXED_POINT_FACTOR: f64 = (1i64 << FRACTIONAL_BITS) as f64;
const INV_FIXED_PONT_FACTOR_F64: f64 = 1.0 / FIXED_POINT_FACTOR;

const FP64_PI: i64 = (consts::PI * FIXED_POINT_FACTOR) as i64;
const FP64_FRACT_PI_2: i64 = (consts::PI * 0.5 * FIXED_POINT_FACTOR) as i64;
const FP64_PI_SQUARED: i64 = (consts::PI * consts::PI * FIXED_POINT_FACTOR) as i64;
const FP64_2PI: i64 = (2.0 * consts::PI * FIXED_POINT_FACTOR) as i64;
const FP64_INV_2PI: i64 = (FIXED_POINT_FACTOR / (2.0 * consts::PI)) as i64;

/// ## Description
/// A Custom fixed point utility, for storing a number with factional parts
/// ### Specs
/// - 64 bits with 16 fractional bits
/// - has basic math functions, mod, floor, ceil, min,max,clamp and comparison
/// - can convert from float to Fixedpoint and back  
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Default)]
pub struct FP64 {
    data: i64,
}
impl FP64 {
    pub const fn zero() -> Self {
        Self { data: 0 }
    }

    pub fn bits(&self) -> u64 {
        self.data as u64
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

    pub const fn pi() -> Self {
        Self { data: FP64_PI }
    }

    pub const fn pi_2() -> Self {
        Self { data: FP64_2PI }
    }

    pub const fn inv_2pi() -> Self {
        Self { data: FP64_INV_2PI }
    }

    pub const fn pi_fract_2() -> Self {
        Self {
            data: FP64_FRACT_PI_2,
        }
    }

    pub const fn pi_squared() -> Self {
        Self {
            data: FP64_PI_SQUARED,
        }
    }

    pub fn cos(self) -> Self {
        (self + Self::pi_fract_2()).sin()
    }

    /// computes x % 2^`exp`
    pub fn fast_mod(&self, exp: u8) -> Self {
        let mask = (1 << (exp + FRACTIONAL_BITS as u8)) - 1;
        Self::from_bits(self.data & mask)
    }

    pub fn as_i64(&self) -> i64 {
        (self.data >> FRACTIONAL_BITS) as i64
    }

    pub fn as_f64(&self) -> f64 {
        self.data as f64 * INV_FIXED_PONT_FACTOR_F64
    }

    pub const fn c_from_bits(data: i64) -> Self {
        Self { data }
    }
}

impl Add for FP64 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.data + rhs.data)
    }
}

impl AddAssign for FP64 {
    fn add_assign(&mut self, rhs: Self) {
        self.data += rhs.data;
    }
}

impl Add<i32> for FP64 {
    type Output = Self;
    fn add(self, rhs: i32) -> Self::Output {
        self + Self::from(rhs)
    }
}

impl Sub for FP64 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.data - rhs.data)
    }
}

impl Sub<i32> for FP64 {
    type Output = Self;
    fn sub(self, rhs: i32) -> Self::Output {
        self - Self::from(rhs)
    }
}

impl SubAssign for FP64 {
    fn sub_assign(&mut self, rhs: Self) {
        self.data -= rhs.data
    }
}

impl Mul for FP64 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_bits((self.data >> 8) * (rhs.data >> 8))
    }
}
impl Mul<i32> for FP64 {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        self * Self::from(rhs)
    }
}

impl MulAssign for FP64 {
    fn mul_assign(&mut self, rhs: Self) {
        self.data = (self.data >> 8) * (rhs.data >> 8)
    }
}

impl MulAssign<i32> for FP64 {
    fn mul_assign(&mut self, rhs: i32) {
        *self *= Self::from(rhs)
    }
}

impl Div for FP64 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::from_bits(((self.data << 8) / rhs.data) << 8)
    }
}
impl Div<i32> for FP64 {
    type Output = Self;
    fn div(self, rhs: i32) -> Self::Output {
        self / Self::from(rhs)
    }
}
impl DivAssign for FP64 {
    fn div_assign(&mut self, rhs: Self) {
        self.data = (self.data << 8 / rhs.data) << 8;
    }
}
impl DivAssign<i32> for FP64 {
    fn div_assign(&mut self, rhs: i32) {
        *self /= Self::from(rhs);
    }
}

impl Shr<u8> for FP64 {
    type Output = Self;
    fn shr(self, rhs: u8) -> Self::Output {
        Self::from_bits(self.data >> rhs)
    }
}
impl Shl<u8> for FP64 {
    type Output = Self;
    fn shl(self, rhs: u8) -> Self::Output {
        Self::from_bits(self.data << rhs)
    }
}

impl Display for FP64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_f64())
    }
}
impl Debug for FP64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_f64())
    }
}

/*
 CONVERSION CRAP HERE
*/
impl From<i32> for FP64 {
    fn from(num: i32) -> Self {
        let num = num as i64;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}
impl From<i64> for FP64 {
    fn from(num: i64) -> Self {
        let num = num as i64;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}
impl From<i128> for FP64 {
    fn from(num: i128) -> Self {
        let num = num as i64;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}

impl From<u32> for FP64 {
    fn from(num: u32) -> Self {
        let num = num as i64;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}
impl From<u64> for FP64 {
    fn from(num: u64) -> Self {
        let num = num as i64;
        Self {
            data: num << FRACTIONAL_BITS,
        }
    }
}

impl From<f32> for FP64 {
    fn from(num: f32) -> Self {
        FP64::from_bits((num * FIXED_POINT_FACTOR as f32) as i64)
    }
}
impl From<f64> for FP64 {
    fn from(num: f64) -> Self {
        FP64::from_bits((num * FIXED_POINT_FACTOR) as i64)
    }
}

impl FP64 {

    /// divisor assumed to be relatively small
    pub fn remainder(self, inv_divisor: Self, divisor: Self) -> Self {
        let x = self;
        let x_scaled = Self::from_bits((x.data >> 14) * (inv_divisor.data >> 1) >> 1);
        let x_quotient = x_scaled.floor();
        let x_multiple = Self::from_bits((x_quotient.data >> 14) * (divisor.data >> 1) >> 1);
        let x_remainder = x - x_multiple;
        x_remainder
    }

    /// Computes sin quickly by using spline approximations
    /// ## Comments 
    /// - no multiplications
    pub fn sin(self) -> Self {
        let inaccurate_spline = |x: Self| {
            const TWO_OVER_PI_SQUARED: i64 =
                ((2.0 / consts::PI) * (2.0 / consts::PI) * FIXED_POINT_FACTOR) as i64;
            x * (Self::pi() - x) * Self::from_bits(TWO_OVER_PI_SQUARED)
        };

        let accurate_spline = |x: Self| {
            const K0F: f64 = 0.775;
            const K0: i64 = (K0F * FIXED_POINT_FACTOR) as i64;
            const K1: i64 = ((1.0 - K0F) * FIXED_POINT_FACTOR) as i64;

            let spline = inaccurate_spline(x);
            let spline_squared = spline * spline;
            
            // better result by interpolating spline and spline_squared
            let accurate_estimation =
                Self::from_bits(K0) * spline + Self::from_bits(K1) * spline_squared;

            // clip accurate spline between 0 and pi
            let when_gt_zero = (Self::zero() - x).data as i64 >>63;
            let when_lt_pi  =  (x-  Self::pi()).data as i64 >> 63; 
            Self::from_bits(accurate_estimation.data & (when_gt_zero & when_lt_pi))
        };

        // computes sin between 0 and 2pi
        let spline_0_2pi = |x: Self| accurate_spline(x) - accurate_spline(x - Self::pi());

        //repeats sin infinitely
        let t = self.remainder(Self::inv_2pi(), Self::pi_2());
        spline_0_2pi(t)
    }
}
#[test]
fn trig_eval_bug_2() {
    let t = FP64::from_bits(830130);
    let s = t.sin();

    println!("{s},raw = {}", s.data);
}
#[test]
fn conversion_tests() {
    let val = FP64::from(25);
    assert_eq!(25, val.as_i64());

    let val = FP64::from(-1);
    assert_eq!(-1, val.as_i64());

    let val = FP64::from(-10);
    assert_eq!(-10, val.as_i64());

    //exhaustive test
    for k in -900_000_00..=900_000_00 {
        // println!("k ={}",k );
        let val = FP64::from(k);

        assert_eq!(k, val.as_i64(), "integer shotgun test failed");
    }
}

#[test]
fn fast_mod_tests() {
    let normal_mod = (0..500_000i64).map(|k| k % 16).collect::<Vec<_>>();
    let fixed_mod = (0..500_000i64)
        .map(|k| FP64::from(k).fast_mod(4).as_i64())
        .collect::<Vec<_>>();

    assert_eq!(&normal_mod, &fixed_mod);
}
#[test]
fn trig_eval_bug_1() {
    let t = FP64::from_bits(38601);
    let s = t.sin();

    println!("{s},raw = {}", s.data);
}

#[test]
fn trig_tests() {
    const NUM_STEPS: usize = 2048;
    let delta_f64 = 2.0*3.14159 / NUM_STEPS as f64;
    let delta_fp64 = FP64::from(delta_f64);
    let mut t_f64 = 0.0f64;
    let mut t_fp64 = FP64::zero();
    const TOLERANCE: f64 = 0.03;

    for k in 0..NUM_STEPS {
        let s_f64 = t_f64.sin();
        let s_fp64 = t_fp64.sin();
        let distance = (s_f64 - s_fp64.as_f64()).abs();
        let meets_tolerance = distance < TOLERANCE;
        if meets_tolerance == false {
            println!(
                "k = {k}\nangle_f64= {},angle_fp64={}\nf64={s_f64},fp64={s_fp64}\ndistance = {distance}",
                t_f64, t_fp64
            );
            println!("fp64_raw = {}", t_fp64.data);
            panic!("tolerance not met");
        }
        t_f64 += delta_f64;
        t_fp64 += delta_fp64
    }
}
