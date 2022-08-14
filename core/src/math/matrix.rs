use super::*;

use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul},
};

pub mod traits;

use self::traits::HasConsts;

pub type Mat4<T> = Matrix<16, T>;
pub type Mat3<T> = Matrix<9, T>;

#[derive(Copy, Clone)]
pub struct Matrix<const CAPACITY: usize, T> {
    data: [T; CAPACITY],
}
impl<const CAP: usize, T> Matrix<CAP, T>
where
    T: Default + Copy + HasConsts,
{
    pub fn new() -> Self {
        Self {
            data: [T::default(); CAP],
        }
    }

    pub fn with_data(mut self, data: [T; CAP]) -> Self {
        self.data = data;
        self
    }
    pub fn zero() -> Self {
        Self {
            data: [T::zero(); CAP],
        }
    }

    pub fn identity() -> Self {
        let mut data = [T::default(); CAP];
        let rows = compile_time_root(CAP);
        for i in 0..rows {
            data[rows * i + i] = T::one();
        }
        Self { data }
    }
}
impl<const CAP: usize, T> Mul for Matrix<CAP, T>
where
    T: HasConsts + Default + Copy + AddAssign + Mul<Output = T> + Add<Output = T>,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let n = compile_time_root(CAP);
        let mut result = Self::zero();

        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    result.data[i * n + j] += self.data[i * n + k] * rhs.data[k * n + j];
                }
            }
        }

        result
    }
}

impl<const CAP: usize, T> std::fmt::Display for Matrix<CAP, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = compile_time_root(CAP);
        for k in 0..n {
            for (k, e) in self.data[n * k..n * (k + 1)].iter().enumerate() {
                if k == 0 {
                    write!(f, "[{:5}", e)?;
                } else {
                    write!(f, ",{:5}", e)?;
                }
            }
            write!(f, "]\n")?;
        }
        Ok(())
    }
}
const fn compile_time_root(x: usize) -> usize {
    if x == 0 || x == 1 {
        return x;
    }
    // Starting from 1, try all numbers until
    // i*i is greater than or equal to x.
    let mut i = 1;
    let mut result = 1;
    while result <= x {
        i += 1;
        result = i * i;
    }
    i - 1
}

#[test]
fn sanity() {
    /*


       0., 1.,2.,0.,       0., 1.,2.,0.,
       0., 1.,2.,0.,       0., 1.,2.,0.,
       0., 1.,2.,0.,       0., 1.,2.,0.,
       0., 1.,2.,0.,       0., 1.,2.,0.,

    */
    let a = Mat4::<f32>::new().with_data([
        0., 1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12., 13., 14., 15.,
    ]);
    let mut b = Mat4::<f32>::identity();

    println!("b=\n{}", b);

    println!("b*b =...\n{}", b * b);

    println!("a*a =...\n{}", a * a);
}
