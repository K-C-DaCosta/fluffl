use super::*;

mod lu;
mod stack;

mod tests;

pub use self::{lu::*, stack::*};

use std::{
    fmt::Display,
    ops::{Add, AddAssign, Deref, DerefMut, Div, Mul, Neg, Sub,SubAssign},
};

pub type Mat4<T> = Matrix<4, 4, T>;
pub type Mat3<T> = Matrix<3, 3, T>;
pub type SquareMat<const N:usize,T> = Matrix<N,N,T>;

#[derive(Copy, Clone)]
pub struct Matrix<const N: usize, const M: usize, T> {
    data: [[T; M]; N],
}

impl<const N: usize, const M: usize, T> Matrix<N, M, T>
where
    T: Default + Copy + HasScalar,
{
    pub fn new() -> Self {
        Self {
            data: [[T::default(); M]; N],
        }
    }

    pub fn with_data(mut self, data: [[T; M]; N]) -> Self {
        self.data = data;
        self
    }

    pub fn zero() -> Self {
        Self {
            data: [[T::zero(); M]; N],
        }
    }

    pub fn swap_rows(&mut self, i0: usize, i1: usize) {
        let min_row_idx = i0.min(i1);
        let max_row_idx = i0.max(i1);

        let (min_rows, max_rows) = self.split_at_mut(max_row_idx);
        let min_row = min_rows[min_row_idx].iter_mut();
        let max_row = max_rows[0].iter_mut();
        min_row.zip(max_row).for_each(|(r1, r2)| {
            let temp = *r1;
            *r1 = *r2;
            *r2 = temp;
        });
    }

    pub unsafe fn swap_rows_unchecked(&mut self, i0: usize, i1: usize) {
        let row_i0_ptr = self.get_unchecked_mut(i0) as *mut [T; M];
        let row_i1_ptr = self.get_unchecked_mut(i1) as *mut [T; M];
        let row_i0 = (*row_i0_ptr).iter_mut();
        let row_i1 = (*row_i1_ptr).iter_mut();
        row_i0.zip(row_i1).for_each(|(r1, r2)| {
            let temp = *r1;
            *r1 = *r2;
            *r2 = temp;
        });
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const T, N * M) }
    }

    pub fn as_slice_mut(&self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.data.as_ptr() as *mut T, N * M) }
    }
}

impl<const N: usize, T> Matrix<N, N, T>
where
    T: Default + Copy + HasScalar,
{
    pub fn identity() -> Self {
        let mut data = Self::zero();
        for i in 0..N {
            data[i][i] = T::one();
        }
        data
    }

    /// computes permulation matrix,that when multiplied from left, swaps rows `r0` and `r1`
    pub fn permute_swap_rows(r0: usize, r1: usize) -> Self {
        let mut permutation = Self::identity();
        permutation[r0][r0] = T::zero();
        permutation[r0][r1] = T::one();
        permutation[r1][r1] = T::zero();
        permutation[r1][r0] = T::one();
        permutation
    }
}
impl<const N: usize, const M: usize, T> Matrix<N, M, T>
where
    T: Default
        + HasScalar
        + Copy
        + Add<Output = T>
        + Mul<Output = T>
{
    /// computes A*v for some column vector `v`
    pub fn transform(&self, rhs: Vector<N, T>) -> Vector<N, T> {
        let mut result = Vector::zero();
        result
            .iter_mut()
            .zip(self.data.iter())
            .for_each(|(res, row)| {
                *res = row
                    .iter()
                    .zip(rhs.iter())
                    .fold(T::zero(), |acc, (&r, &c)| acc + r * c)
            });
        result
    }
}

impl<const N: usize, const M: usize, T> Matrix<N, M, T>
where
    T: Default
        + HasScalar
        + Copy
        + PartialOrd
        + Mul<Output = T>
        + Sub<Output = T>,
{
    /// does an element-wise comparison of matracies of similar dimension
    /// and retruns true if all elements relatively similar to each other
    pub fn is_similar(&self, other: &Self, tolerance: T) -> bool {
        let lhs_elems = self.iter().flat_map(|row| row.iter());
        let rhs_elems = other.iter().flat_map(|row| row.iter());
        lhs_elems.zip(rhs_elems).all(|(&lhs, &rhs)| {
            let disp = lhs - rhs;
            let dist = disp * disp;
            dist < tolerance
        })
    }
}

impl<const N: usize, T> Matrix<N, N, T>
where
    T: HasScalar + Default + Copy + AddAssign + Mul<Output = T> + Add<Output = T>,
{
    fn multiply_both_square(self, rhs: Self) -> Self {
        let mut result = Self::zero();
        for i in 0..N {
            for j in 0..N {
                for k in 0..N {
                    result.data[i][j] += self.data[i][k] * rhs.data[k][j];
                }
            }
        }
        result
    }
}

impl<const N: usize, T> Matrix<N, N, T>
where
    T: Default + Copy,
{
    pub fn transpose_in_place(&mut self) {
        for i in 0..N {
            for j in i + 1..N {
                let temp = self[i][j];
                self[i][j] = self[j][i];
                self[j][i] = temp;
            }
        }
    }

    pub fn transpose(&self) -> Self {
        let mut res = self.clone();
        res.transpose_in_place();
        res
    }
}

impl<const N: usize, T> Mul for Matrix<N, N, T>
where
    T: HasScalar + Default + Copy + AddAssign + Mul<Output = T> + Add<Output = T>,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply_both_square(rhs)
    }
}

impl<const N: usize, const M: usize, T> Mul<Vector<N, T>> for Matrix<N, M, T>
where
    T: HasScalar + Default + Copy + Mul<Output = T> + Add<Output = T>,
{
    type Output = Vector<N, T>;
    fn mul(self, rhs: Vector<N, T>) -> Self::Output {
        self.transform(rhs)
    }
}
impl<const N: usize, T> Matrix<N, N, T>
where
    T: HasScalar + Default + Copy + Mul<Output = T> + Div<Output = T> + Sub<Output = T> + SubAssign,
{
    pub fn back_sub<V>(&self, b: V) -> Vector<N, T>
    where
        V: Into<Vector<N, T>>,
    {
        let b = b.into();
        let mut res = Vector::zero();
        for i in (0..N).rev() {
            let mut sol_comp = b[i];
            for j in (i + 1..N).rev() {
                sol_comp -= self[i][j] * res[j];
            }
            res[i] = sol_comp / self[i][i];
        }
        res
    }

    pub fn forward_sub<V>(&self, b: V) -> Vector<N, T>
    where
        V: Into<Vector<N, T>>,
    {
        let b = b.into();
        let mut res = Vector::zero();

        for i in 0..N {
            let mut sol_comp = b[i];
            for j in 0..i {
                sol_comp -= self[i][j] * res[j];
            }
            res[i] = sol_comp
        }

        res
    }
}

impl<const N: usize, const M: usize, T> std::fmt::Display for Matrix<N, M, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.data.iter() {
            for (k, e) in row.iter().enumerate() {
                if k == 0 {
                    write!(f, "[{:8.2}", e)?;
                } else {
                    write!(f, ",{:8.2}", e)?;
                }
            }
            write!(f, "]\n")?;
        }

        Ok(())
    }
}

impl<const N: usize, const M: usize, T> Deref for Matrix<N, M, T> {
    type Target = [[T; M]; N];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<const N: usize, const M: usize, T> DerefMut for Matrix<N, M, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[rustfmt::skip]
pub fn translate4<T>(translate:Vec4<T>)->Mat4<T>
where T:HasScalar+Default+Copy,
{
    Mat4::new().with_data([
        [T::one() , T::zero(),T::zero(),translate[0]],
        [T::zero(), T::one() ,T::zero(),translate[1]],  
        [T::zero(), T::zero(),T::one() ,translate[2]],
        [T::zero(), T::zero(),T::zero(),T::one()    ]
    ])
}

#[rustfmt::skip]
pub fn scale4<T>(scale:Vec4<T>)->Mat4<T>
where T:HasScalar+Default+Copy,
{
    
    Mat4::new().with_data([
        [scale[0] , T::zero(),T::zero(),T::zero()],
        [T::zero(), scale[1] ,T::zero(),T::zero()],
        [T::zero(), T::zero(),scale[2] ,T::zero()],
        [T::zero(), T::zero(),T::zero(),T::one()],
    ])
}

#[rustfmt::skip]
pub fn rotate_z<T>(rad:T)->Mat4<T>
where T:HasScalar+Default+Copy + HasTrig + Neg<Output=T>,
{
    let cos = rad.cos(); 
    let sin = rad.sin();
    Mat4::new().with_data([
        [cos      , -sin     ,T::zero(),T::zero()],
        [sin      ,  cos     ,T::zero(),T::zero()],
        [T::zero(), T::zero(),T::one() ,T::zero()],
        [T::zero(), T::zero(),T::zero(),T::one() ],
    ])
}

#[rustfmt::skip]
pub fn rotate_x<T>(rad:T)->Mat4<T>
where T:HasScalar+Default+Copy + HasTrig + Neg<Output=T>,
{
    let cos = rad.cos(); 
    let sin = rad.sin();
    Mat4::new().with_data([
        [T::one() ,  T::zero(),T::zero(),T::zero()],
        [T::zero(),  cos      , -sin    ,T::zero()],
        [T::zero(),  sin      , cos     ,T::zero()],
        [T::zero(), T::zero() ,T::zero(),T::one() ],
    ])
}

/// a orthographic matrix with fixed variables
#[rustfmt::skip]
pub fn calc_ortho_window_f32(w: f32, h: f32) -> Mat4<f32> {
    let data = [
        [2.0 / w,0.      ,0.   ,-1.0 ], 
        [0.     ,-2.0 / h,0.   , 1.0 ],
        [0.     ,0.      ,-0.01,  0. ],
        [0.     ,0.      ,0.   , 1.0 ],
    ];
    Mat4::new().with_data(data)
}
