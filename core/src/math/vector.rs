use std::{
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Deref, DerefMut, Div, Index, IndexMut, Mul, MulAssign, Sub},
};

mod tests; 
mod component_writer; 
use super::HasConstants;

pub use self::{component_writer::ComponentWriter}; 

pub type Vec2<T> = Vector<2, T>;
pub type Vec3<T> = Vector<3, T>;
pub type Vec4<T> = Vector<4, T>;

#[derive(Copy, Clone, Debug)]
pub struct Vector<const N: usize, T> {
    pub data: [T; N],
}

impl<const N: usize, T> Display for Vector<N, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:3}", self.data[0])?;
        for k in 1..N {
            write!(f, ",{:3}", self.data[k])?;
        }
        write!(f, "]")
    }
}

impl<const N: usize, T> Vector<N, T>
where
    T: Default + Copy + HasConstants,
{
    pub fn zero() -> Self {
        Self {
            data: [T::default(); N],
        }
    }

    pub fn data(&self) -> &[T; N] {
        &self.data
    }

    pub fn convert<const M: usize>(b: Vector<M, T>) -> Self {
        let mut res = Self::zero();
        for k in 0..M.min(N) {
            res[k] = b[k];
        }
        res
    }

    pub fn axis(self, index: usize) -> Self {
        let mut res = Self::zero();
        res[index] = self[index];
        res
    }

    pub fn to_pos<const M: usize>(b: Vector<M, T>) -> Self {
        let mut res = Self::zero();

        res.iter_mut()
            .zip(b.iter())
            .take(M.min(N - 1))
            .for_each(|(r, &e)| *r = e);

        // for k in 0..M.min(N - 1) {
        //     res[k] = b[k];
        // }
        res[N - 1] = T::one();
        res
    }

    pub fn from_array(data: [T; N]) -> Self {
        Self { data }
    }
}

impl<const N: usize, T> Vector<N, T>
where
    T: Default + Copy + Add<Output = T> + Mul<Output = T>,
{
    pub fn dot(lhs: Self, rhs: Self) -> T {
        lhs.data
            .iter()
            .zip(rhs.data)
            .fold(T::default(), |acc, (&lhs, rhs)| acc + (lhs * rhs))
    }

    pub fn length_squared(self) -> T {
        Self::dot(self, self)
    }
}

impl<const N: usize, T> Vector<N, T>
where
    T: Default + Copy + Mul<Output = T>,
{
    fn component_wise_multiplication(lhs: Self, rhs: Self) -> Self {
        let mut data = [T::default(); N];
        data.iter_mut()
            .zip(lhs.iter())
            .zip(rhs.iter())
            .for_each(|((res, &lhs), &rhs)| {
                *res = lhs * rhs;
            });
        Self { data }
    }

    fn scale(lhs: Self, rhs: T) -> Self {
        let mut data = [T::default(); N];
        data.iter_mut().zip(lhs.iter()).for_each(|(res, &lhs)| {
            *res = lhs * rhs;
        });
        Self { data }
    }
}

impl<const N: usize, T> Vector<N, T>
where
    T: Default + Copy + Div<Output = T>,
{
    fn component_wise_division(lhs: Self, rhs: Self) -> Self {
        let mut data = [T::default(); N];
        data.iter_mut()
            .zip(lhs.iter())
            .zip(rhs.iter())
            .for_each(|((res, &lhs), &rhs)| {
                *res = lhs / rhs;
            });
        Self { data }
    }
}

impl<const N: usize, T> Vector<N, T>
where
    T: MulAssign + Copy,
{
    fn scale_assign(&mut self, scalar: T) {
        self.data.iter_mut().for_each(|comp| *comp *= scalar);
    }
}

impl<const N: usize> Vector<N, f32> {
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }
}

impl<T> Vector<4, T>
where
    T: HasConstants + Copy + Default + Div<Output = T> + Mul<Output = T>,
{
    pub fn rgba_u32(color: u32) -> Self {
        let mut result = Self::zero();
        let inv_denom = T::from_i32(1) / T::from_i32(255);
        for k in 0..4 {
            let comp_byte = (color >> 8 * (4 - k - 1)) & 0xFF;
            let comp = T::from_i32(comp_byte as i32) * inv_denom;
            result[k] = comp;
        }
        result
    }

    pub fn rgb_u32(color: u32) -> Self {
        Self::rgba_u32((color << 8) | 0xff)
    }
}

impl<const N: usize, T> Add for Vector<N, T>
where
    T: Default + Add<Output = T> + Copy,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut data = [T::default(); N];
        data.iter_mut()
            .zip(self.data.iter())
            .zip(rhs.data)
            .for_each(|((res, &lhs), rhs)| {
                *res = lhs + rhs;
            });
        Self { data }
    }
}

impl<const N: usize, T> AddAssign for Vector<N, T>
where
    T: Default + AddAssign<T> + Copy,
{
    fn add_assign(&mut self, rhs: Self) {
        self.data
            .iter_mut()
            .zip(rhs.data.iter())
            .for_each(|(lhs_c, &rhs_c)| *lhs_c += rhs_c);
    }
}

impl<const N: usize, T> Sub for Vector<N, T>
where
    T: Default + Sub<Output = T> + Copy,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut data = [T::default(); N];
        data.iter_mut()
            .zip(self.data.iter())
            .zip(rhs.data)
            .for_each(|((res, &lhs), rhs)| {
                *res = lhs - rhs;
            });
        Self { data }
    }
}

impl<const N: usize, T> Mul for Vector<N, T>
where
    T: Default + Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::component_wise_multiplication(self, rhs)
    }
}

impl<const N: usize, T> Div for Vector<N, T>
where
    T: Default + Div<Output = T> + Copy,
{
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::component_wise_division(self, rhs)
    }
}

impl<const N: usize, T> Mul<T> for Vector<N, T>
where
    T: Default + Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::scale(self, rhs)
    }
}

impl<const N: usize, T> MulAssign<T> for Vector<N, T>
where
    T: Default + Mul<Output = T> + MulAssign + Copy,
{
    fn mul_assign(&mut self, rhs: T) {
        self.scale_assign(rhs)
    }
}

impl<const N: usize, T> Deref for Vector<N, T> {
    type Target = [T; N];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<const N: usize, T> DerefMut for Vector<N, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<const N: usize, T> Index<usize> for Vector<N, T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
impl<const N: usize, T> IndexMut<usize> for Vector<N, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<const N: usize> From<[f32; N]> for Vector<N, f32> {
    fn from(arr: [f32; N]) -> Self {
        Self::from_array(arr)
    }
}
impl From<[f32; 2]> for Vector<4, f32> {
    fn from(arr: [f32; 2]) -> Self {
        Self::from_array([arr[0], arr[1], 0.0, 0.0])
    }
}
impl From<[f32; 3]> for Vector<4, f32> {
    fn from(arr: [f32; 3]) -> Self {
        Self::from_array([arr[0], arr[1], arr[2], 0.0])
    }
}

#[rustfmt::skip]
impl <T:Copy> Vec4<T>{
    pub fn x(&self)->T{self.data[0]}
    pub fn y(&self)->T{self.data[1]}
    pub fn z(&self)->T{self.data[2]}
    pub fn w(&self)->T{self.data[3]}
}

#[rustfmt::skip]
impl <T:Copy> Vec3<T>{
    pub fn x(&self)->T{self.data[0]}
    pub fn y(&self)->T{self.data[1]}
    pub fn z(&self)->T{self.data[2]}
}

#[rustfmt::skip]
impl <T:Copy> Vec2<T>{
    pub fn x(&self)->T{self.data[0]}
    pub fn y(&self)->T{self.data[1]}
}