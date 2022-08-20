use super::*;

use std::ops::{Add, AddAssign, Sub};

/// 2D aabb
pub type AABB2<T> = AABB<2, T>;

/// 3D aabb
pub type AABB3<T> = AABB<3, T>;

/// ## Description
/// A representation of an axis-alligned rectangle
#[derive(Copy, Clone)]
pub struct AABB<const DIM: usize, T> {
    s0: Vector<DIM, T>,
    s1: Vector<DIM, T>,
}

impl<const DIM: usize, T> AABB<DIM, T>
where
    T: Copy + Default + Add<Output = T> + Sub<Output = T> + AddAssign + PartialOrd + HasScalar,
{
    pub fn from_segment(a: Vector<DIM, T>, b: Vector<DIM, T>) -> Self {
        Self { s0: a, s1: b }
    }

    pub fn from_point_and_lengths(x0: Vector<DIM, T>, dim: Vector<DIM, T>) -> Self {
        Self {
            s0: x0,
            s1: x0 + dim,
        }
    }

    pub fn dims(&self) -> Vector<DIM, T> {
        self.s1 - self.s0
    }

    pub fn is_point_inside(&self, point: Vector<DIM, T>) -> bool {
        let dims = self.dims();
        let disp = point - self.s0;

        disp.iter()
            .zip(dims.iter())
            .all(|(&disp_comp, &dims)| disp_comp < dims && disp_comp > T::zero())
    }

    pub fn translate(&mut self, translate: Vector<DIM, T>) {
        self.s0 += translate;
        self.s1 += translate;
    }
}
