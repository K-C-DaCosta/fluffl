use super::*;
pub mod stack; 
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul},
};

pub type Mat4<T> = Matrix<16, 4, 4, T>;
pub type Mat3<T> = Matrix<9, 3, 3, T>;

#[derive(Copy, Clone)]
pub struct Matrix<const CAPACITY: usize, const N: usize, const M: usize, T> {
    data: [T; CAPACITY],
}
impl<const CAP: usize, const N: usize, const M: usize, T> Matrix<CAP, N, M, T>
where
    T: Default + Copy + HasScalar,
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
        for i in 0..N {
            data[N * i + i] = T::one();
        }
        Self { data }
    }
}

impl<const CAP: usize, const N: usize, const M: usize, T> Mul for Matrix<CAP, N, M, T>
where
    T: HasScalar + Default + Copy + AddAssign + Mul<Output = T> + Add<Output = T>,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = Self::zero();

        debug_assert_eq!(N, M, "mul assumed to be square");

        for i in 0..N {
            for j in 0..N {
                for k in 0..N {
                    result.data[i * N + j] += self.data[i * N + k] * rhs.data[k * N + j];
                }
            }
        }

        result
    }
}

impl<const CAP: usize, const N: usize, const M: usize, T> Mul<Vector<N, T>> for Matrix<CAP, N, M, T>
where
    T: HasScalar + Default + Copy + AddAssign + Mul<Output = T> + Add<Output = T>,
{
    type Output = Vector<N, T>;
    fn mul(self, rhs: Vector<N, T>) -> Self::Output {
        let mut result = Self::Output::zero();
        for i in 0..N {
            for j in 0..M {
                result[i] += rhs[j] * self.data[i * M + j];
            }
        }
        result
    }
}

impl<const CAP: usize, const N: usize, const M: usize, T> std::fmt::Display for Matrix<CAP, N, M, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..N {
            for (k, e) in self.data[M * row..M * (row + 1)].iter().enumerate() {
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

impl<const CAP: usize, const N: usize, const M: usize, T> std::ops::Deref for Matrix<CAP, N, M, T> {
    type Target = [T; CAP];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
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

    let c = a * a;

    let x = c * Vec4::from_array([1.0, 2.0, 3.0, 4.0]);
    println!("x = {}", x);
}

#[rustfmt::skip]
pub fn translate4<T>(translate:Vec4<T>)->Mat4<T>
where T:HasScalar+Default+Copy,
{
    Mat4::new().with_data([
        T::one() , T::zero(),T::zero(),translate[0],
        T::zero(), T::one() ,T::zero(),translate[1],
        T::zero(), T::zero(),T::one() ,translate[2],
        T::zero(), T::zero(),T::zero(),T::one(),
    ])
}

#[rustfmt::skip]
pub fn scale4<T>(scale:Vec4<T>)->Mat4<T>
where T:HasScalar+Default+Copy,
{
    
    Mat4::new().with_data([
        scale[0] , T::zero(),T::zero(),T::zero(),
        T::zero(), scale[1] ,T::zero(),T::zero(),
        T::zero(), T::zero(),scale[2] ,T::zero(),
        T::zero(), T::zero(),T::zero(),T::one(),
    ])
}

/// a orthographic matrix with fixed variables
#[rustfmt::skip]
pub fn calc_ortho_window_f32(w: f32, h: f32) -> Mat4<f32> {
    let data = [
        2.0 / w,0.      ,0.,-1.0, 
        0.     ,-2.0 / h,0., 1.0,
        0.     ,0.      ,1.,  0.,
        0.     ,0.      ,0., 1.0,
    ];
    Mat4::new().with_data(data)
}
