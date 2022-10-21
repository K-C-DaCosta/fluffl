use std::ops::{MulAssign, SubAssign};

use super::*;

pub struct DecompLUInplace<'a, const N: usize, T> {
    lu: &'a SquareMat<N, T>,
}

impl<'a, const N: usize, T> DecompLUInplace<'a, N, T>
where
    T: HasConstants
        + Default
        + Copy
        + Mul<Output = T>
        + Div<Output = T>
        + Sub<Output = T>
        + SubAssign,
{
    pub fn data(&self) -> SquareMat<N, T> {
        *self.lu
    }

    pub fn invert(&self) -> SquareMat<N, T> {
        let mut inv = SquareMat::zero();
        let mut b = Vector::<N, T>::zero();
        for j in 0..N {
            let prev_k = (j as isize - 1).max(0) as usize;
            b[prev_k] = T::zero();
            b[j] = T::one();
            //solve for column j of A^-1
            let inverse_column = self.solve(b);
            //copy column j into the resultant matrix
            for i in 0..N {
                inv[i][j] = inverse_column[i];
            }
        }
        inv
    }

    /// solves system of equation: 'Ax=`b`' in `O(n^2)`
    /// ## returns
    /// solution `x`
    pub fn solve(&self, b: Vector<N, T>) -> Vector<N, T> {
        self.back_sub(self.forward_sub(b))
    }

    ///computes determinant
    pub fn det(&self) -> T {
        let mut product = self[0][0];
        for k in 1..N {
            product = product * self[k][k]
        }
        product
    }
}

impl<'a, const N: usize, T> Deref for DecompLUInplace<'a, N, T> {
    type Target = SquareMat<N, T>;
    fn deref(&self) -> &Self::Target {
        self.lu
    }
}

/// PLU Decomp
/// can be used to:
/// - solve for  Ax=b in O(n^2)  rather than O(n^3)
/// - get determinants
/// - compute A^-1
#[derive(Copy, Clone)]
pub struct DecompPLU<const N: usize, T> {
    pub lower: Matrix<N, N, T>,
    pub upper: Matrix<N, N, T>,
    pub permutation: Matrix<N, N, T>,
}
impl<const N: usize, T> DecompPLU<N, T> {
    pub fn new(
        lower: SquareMat<N, T>,
        upper: SquareMat<N, T>,
        permutation: SquareMat<N, T>,
    ) -> Self {
        Self {
            lower,
            upper,
            permutation,
        }
    }
}

impl<const N: usize, T> DecompPLU<N, T>
where
    T: HasConstants
        + Default
        + Copy
        + AddAssign
        + Mul<Output = T>
        + MulAssign
        + Add<Output = T>
        + Div<Output = T>
        + Sub<Output = T>
        + SubAssign
        + PartialOrd,
{
    // useless only used for tests
    #[allow(dead_code)]
    fn recompose(&self) -> Matrix<N, N, T> {
        self.permutation.transpose() * self.lower * self.upper
    }

    /// ## Description
    /// solves the equation Ax=b in `O(n^2)`
    /// ## Returns
    /// - `x`
    /// ## Comments
    /// - reminder A = PLU
    pub fn solve(&self, b: Vector<N, T>) -> Vector<N, T> {
        self.upper
            .back_sub(self.lower.forward_sub(self.permutation * b))
    }

    /// solve the inverse matrix one column at a time with lu decomp
    /// ## Comments
    /// - Slow operation `O(n^3)` , use conservatively
    pub fn invert(&self) -> Matrix<N, N, T> {
        let mut inv = Matrix::<N, N, T>::zero();
        let mut b = Vector::<N, T>::zero();
        for j in 0..N {
            let prev_k = (j as isize - 1).max(0) as usize;
            b[prev_k] = T::zero();
            b[j] = T::one();
            //solve for column j of A^-1
            let inverse_column = self.solve(b);
            //copy column j into the resultant matrix
            for i in 0..N {
                inv[i][j] = inverse_column[i];
            }
        }
        inv
    }

    pub fn det(&self) -> T {
        let mut diag_product = self.upper[0][0];
        for i in 1..N {
            diag_product *= self.upper[i][i];
        }
        diag_product
    }
}

impl<const N: usize, T> DecompPLU<N, T>
where
    T: HasConstants + Default + Copy + Display,
{
    pub fn print(&self) {
        println!("permu:\n{}", self.permutation);
        println!("lower:\n{}", self.lower);
        println!("upper:\n{}", self.upper);
    }
}

impl<const N: usize, T> Matrix<N, N, T>
where
    T: HasConstants
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
    /// PLU decomp by partial pivioting
    /// ## Parameters
    /// - `epsilon` is threshold for the pivot
    /// ## Return
    /// function returns `None` if matrix is degenerate/singular
    /// ## comments
    /// - supposedly numerically stable
    /// - not the fastest or most efficient solution, many optimizations are still possible, use conservatively
    pub fn decomp_plu<E>(&self, epsilon: E) -> Option<DecompPLU<N, T>>
    where
        E: Into<Option<T>>,
    {
        let mut lower = Matrix::identity();
        let mut upper = *self;
        let mut permutation = Matrix::identity();

        let default_epsilon = T::from_i32(1) / T::from_i32(1024);
        let near_zero = epsilon.into().map(|e| e * e).unwrap_or(default_epsilon);

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
                    let new_permutation = Matrix::permute_swap_rows(best_pivot_idx, j);
                    permutation = permutation * new_permutation;
                    lower = new_permutation * lower * new_permutation;
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

        Some(DecompPLU {
            lower,
            upper,
            permutation,
        })
    }

    /// does LU decomposition with NO pivoting,gaussian elimination style, **IN PLACE**
    /// ## Comments
    /// - much, **MUCH** faster than PLU decomp since compiler can actually unroll this to make it branchless
    /// - im currently exploring ways of trying to see if i can write this a certain way to get it to autovectorize
    /// - in general not numerically stable
    /// - can only be safely used for special matracies like:
    ///     - diagonally dominant matracies
    ///     - rigid transform matracies
    ///     - orthoganal matracies
    pub fn decomp_lu_inplace_gaussian(&mut self) -> DecompLUInplace<'_, N, T> {
        for j in 0..N {
            let inv_pivot = T::from_i32(-1) / self[j][j];
            for i in j + 1..N {
                let lu = inv_pivot * self[i][j];
                self[i][j] = lu * T::from_i32(-1);
                for k in j + 1..N {
                    let offset = self[j][k] * lu;
                    self[i][k] += offset;
                }
            }
        }
        DecompLUInplace { lu: self }
    }
}

impl<const N: usize, T> Matrix<N, N, T>
where
    T: HasConstants
        + Default
        + Copy
        + AddAssign
        + Mul<Output = T>
        + Add<Output = T>
        + Div<Output = T>
        + Sub<Output = T>
        + SubAssign
        + HasBits,
{
    /// does LU decomposition with do_little
    /// ## Comments
    /// - tried to make this as branchless as possible
    /// - source: https://www.javatpoint.com/doolittle-algorithm-lu-decomposition
    pub fn decomp_lu_inplace_doolittle(&mut self) -> DecompLUInplace<'_, N, T> {
        for i in 1..N {
            for j in 0..N {
                let a = self[i][j];
                let is_upper = (i <= j) as usize;
                let is_upper_mask = (is_upper as u64) * (!0);
                let mut sum = T::zero();
                let loop_bound = is_upper * i + (1 - is_upper) * j;

                for k in 0..loop_bound {
                    sum += self[i][k] * self[k][j];
                }

                let numerator = a - sum;

                let when_upper = numerator.to_bits() & is_upper_mask;
                let when_lower = (numerator / self[j][j]).to_bits() & !is_upper_mask;

                self[i][j] = T::from_bits(when_lower | when_upper);
            }
        }
        DecompLUInplace { lu: self }
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
    let _requires_pivot  = || {
        Mat3::<f32>::new().with_data([
            [ 0., 91., 26.], 
            [60., 3. , 75.], 
            [45., 90., 31.]
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
    let decomp = mat.decomp_plu(0.01).expect("this matrix is not degenerate");

    let recomposition = decomp.recompose();
    assert!(mat.is_similar(&recomposition, THRESHOLD));

    // decomp.print();
    // let sol = decomp.upper.back_sub([1.0, 2.0, 3.0]);
    // println!("upper_sol={sol}");
    // let sol = decomp.lower.forward_sub([1.0, 2.0, 3.0]);
    // println!("lower_sol={sol}");

    let inverse = mat
        .decomp_plu(0.01)
        .map(|lu| lu.invert())
        .expect("matrix is not degenerate");
    assert!((inverse * mat).is_similar(&Mat3::identity(), THRESHOLD));

    //this should fail
    let mat = degen_4x4();
    let decomp = mat.decomp_plu(0.01);
    assert!(decomp.is_none());

    //this should pass, rigid transforms are always invertable
    let mat = rigid_rotation_4x4();
    // println!("transform:\n{mat}");
    let decomp = mat.decomp_plu(0.01).expect("shouldn't be degen");
    // decomp.print();
    assert!(mat.is_similar(&decomp.recompose(), THRESHOLD));

    let mat = rigid_rotation_4x4();
    let mat_inv = rigid_rotation_4x4().decomp_lu_inplace_gaussian().invert();

    // println!("lu:\n{}", mat * mat_inv);

    assert!(
        (mat * mat_inv).is_similar(&SquareMat::identity(), THRESHOLD),
        "rigid transforms should be okay for no-pivot lu decomposition"
    );
}
