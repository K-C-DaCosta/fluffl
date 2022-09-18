use std::ops::{DivAssign, SubAssign};

use super::*;

#[derive(Copy, Clone)]
pub struct DecompositionLU<const N: usize, T> {
    pub lower: Matrix<N, N, T>,
    pub upper: Matrix<N, N, T>,
    pub permutation: Matrix<N, N, T>,
}

impl<const N: usize, T> DecompositionLU<N, T>
where
    T: HasScalar + Default + Copy + AddAssign + Mul<Output = T> + Add<Output = T>,
{
    pub fn recompose(&self) -> Matrix<N, N, T> {
        self.permutation.transpose() * self.lower * self.upper
    }
}

impl<const N: usize, T> DecompositionLU<N, T>
where
    T: HasScalar + Default + Copy + Display,
{
    pub fn print(&self) {
        println!("permu:\n{}", self.permutation);
        println!("lower:\n{}", self.lower);
        println!("upper:\n{}", self.upper);
    }
}

impl<const N: usize, T> Matrix<N, N, T>
where
    T: HasScalar
        + Default
        + Copy
        + AddAssign
        + Mul<Output = T>
        + Add<Output = T>
        + Div<Output = T>
        + Sub<Output = T>
        + SubAssign
        + PartialOrd,
{
    /// LU decomp by partial pivioting
    /// ## comments
    /// - not the fastest or most efficient solution, use conservatively
    pub fn lu_decomposition(&self) -> Option<DecompositionLU<N, T>> {
        let mut lower = Matrix::identity();
        let mut upper = *self;
        let mut permutation = Matrix::identity();
        let near_zero = T::from_i32(1) / T::from_i32(1024);

        //compute upper and lower triangle
        for j in 0..N {
            let mut best_pivot = upper[j][j] * upper[j][j];
            let mut best_pivot_idx = j;

            if best_pivot < near_zero {
                //find a better pivot
                for k in j..N {
                    let dist = upper[k][j] * upper[k][j];
                    if dist > best_pivot {
                        best_pivot = dist;
                        best_pivot_idx = k;
                    }
                }

                // if the new pivot is STLL below threshold then
                // the matrix is likely degenerate
                if best_pivot < near_zero {
                    return None;
                }

                if best_pivot_idx != j {
                    unsafe {
                        upper.swap_rows_unchecked(best_pivot_idx, j);
                    }
                    permutation[j][j] = T::zero();
                    permutation[j][best_pivot_idx] = T::one();
                    permutation[best_pivot_idx][best_pivot_idx] = T::zero();
                    permutation[best_pivot_idx][j] = T::one();
                }
            }

            let inv_pivot = T::from_i32(-1) / upper[j][j];

            for i in j + 1..N {
                let lu = inv_pivot * upper[i][j];
                lower[i][j] = lu * T::from_i32(-1);
                upper[i][j] = T::zero();

                for k in j + 1..N {
                    let offset = upper[j][k] * lu;
                    upper[i][k] += offset;
                }
            }
        }

        Some(DecompositionLU {
            lower,
            upper,
            permutation,
        })
    }

    /// solve the inverse matrix one column at a time with lu decomp
    /// ## Comments
    /// - Slow operation, use conservatively
    pub fn invert(&self) -> Option<Self> {
        let mut inv = Self::zero();
        let mut b = Vector::<N, T>::zero();

        let DecompositionLU {
            upper,
            lower,
            permutation,
        } = self.lu_decomposition()?;

        for j in 0..N {
            let prev_k = (j as isize - 1).max(0) as usize;
            b[prev_k] = T::zero();
            b[j] = T::one();

            //solve for column j of A^-1
            let inverse_column = upper.back_sub(lower.forward_sub(permutation * b));

            //copy column j into the resultant matrix
            for i in 0..N {
                inv[i][j] = inverse_column[i];
            }
        }

        Some(inv)
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

#[test]
fn lu_decomp_sanity() {
    const THRESHOLD: f32 = 0.0125;
    #[rustfmt::skip]
    let non_degen_3x3 = ||{
        Mat3::new().with_data([
            [1. , 2.  , 13.],
            [2. , 16. , 7. ],
            [9. , 10. , 11.],
        ])
    };

    #[rustfmt::skip]
    let degen_4x4 = ||{
        Mat4::new().with_data([
            [1. , 2.  , 3. ,4. ],
            [5. , 6.  , 7. ,8. ],
            [9. , 10. , 11.,12.],
            [13., 14. , 15.,16.],
        ])
    };

    #[rustfmt::skip]
    let rigid_rotation_4x4 = ||{
        translate4(Vec4::from([10.0,2.0,3.0,4.0]))*
        scale4(Vec4::from([1.0,2.0,3.0,4.0]))*
        rotate_x(1.0)*
        rotate_z(3.0)*
        translate4(Vec4::from([1.0,2.0,3.0,4.0]))
    };

    let mat = non_degen_3x3();
    let decomp = mat
        .lu_decomposition()
        .expect("this matrix is not degenerate");

    let recomposition = decomp.recompose();
    assert_eq!(true, mat.is_similar(&recomposition, THRESHOLD));

    // decomp.print();
    // let sol = decomp.upper.back_sub([1.0, 2.0, 3.0]);
    // println!("upper_sol={sol}");
    // let sol = decomp.lower.forward_sub([1.0, 2.0, 3.0]);
    // println!("lower_sol={sol}");

    let inverse = mat.invert().unwrap();
    assert_eq!(true, (inverse * mat).is_similar(&Mat3::identity(), THRESHOLD));

    //this should fail
    let mat = degen_4x4();
    let decomp = (mat).lu_decomposition();
    assert_eq!(true, decomp.is_none());

    //this should pass, rigid transforms are always invertable
    let mat = rigid_rotation_4x4();
    // println!("transform:\n{mat}");
    let decomp = mat.lu_decomposition().expect("shouldn't be degen");
    // decomp.print();
    assert_eq!(true, mat.is_similar(&decomp.recompose(), THRESHOLD));
}
