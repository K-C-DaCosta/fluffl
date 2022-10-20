use super::*;

use std::ops::{Add, AddAssign, Mul, Sub};

/// 2D aabb
pub type AABB2<T> = AABB<2, T>;

/// 3D aabb
pub type AABB3<T> = AABB<3, T>;

/// ## Description
/// A representation of an axis-alligned rectangle
#[derive(Copy, Clone, Debug)]
pub struct AABB<const DIM: usize, T> {
    pub min_pos: Vector<DIM, T>,
    pub max_pos: Vector<DIM, T>,
}

impl<const DIM: usize, T> AABB<DIM, T>
where
    T: Copy
        + Default
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + AddAssign
        + PartialOrd
        + HasConstants,
{
    pub fn zero() -> Self {
        Self::from_segment(Vector::zero(), Vector::zero())
    }

    pub fn from_segment<V0, V1>(a: V0, b: V1) -> Self
    where
        V0: Into<Vector<DIM, T>>,
        V1: Into<Vector<DIM, T>>,
    {
        Self {
            min_pos: a.into(),
            max_pos: b.into(),
        }
    }

    pub fn from_point_and_lengths<V0, V1>(x0: V0, dim: V1) -> Self
    where
        V0:Copy+ Into<Vector<DIM, T>>,
        V1:Copy+ Into<Vector<DIM, T>>,
    {
        Self {
            min_pos: x0.into(),
            max_pos: x0.into() + dim.into(),
        }
    }

    pub fn dims(&self) -> Vector<DIM, T> {
        self.max_pos - self.min_pos
    }

    pub fn is_point_inside(&self, point: Vector<DIM, T>) -> bool {
        let dims = self.dims();
        let disp = point - self.min_pos;

        disp.iter()
            .zip(dims.iter())
            .all(|(&disp_comp, &dims)| disp_comp < dims && disp_comp > T::zero())
    }

    /// checks if `self` fully contains `b`
    pub fn fully_contains(&self, b: AABB<DIM, T>) -> bool {
        b.verts().all(|v| self.is_point_inside(v))
    }

    /// returns an iteratator that computes **ALL** vertexes for the AABB
    pub fn verts(&self) -> impl Iterator<Item = Vector<DIM, T>> {
        /*
            dim = 1 -> verts = 1
            dim = 2 -> verts = 4
            dim = 3 -> verts = 8
            see the pattern?
            for dim = 2
            (x,y) +  (0    , 0    ) -> bit pattern -> 00
            (x,y) +  (0    , dim.y) -> bit pattern -> 01
            (x,y) +  (dim.x, 0    ) -> bit pattern -> 10
            (x,y) +  (dim.x, dim.y) -> bit pattern -> 11

            for dim = 3 and beyond

            loop through bit patterns from 0..2^DIM and check bit to select components
        */
        let dim = self.dims();
        let min_p = self.min_pos;
        (0..1 << DIM).map(move |pattern| {
            let mut delta = Vector::zero();
            for k in 0..DIM {
                let pick_mask = T::from_i32((pattern >> k) & 1);
                delta[k] = pick_mask * dim[k];
            }
            min_p + delta
        })
    }

    pub fn translate(&mut self, disp: Vector<DIM, T>) {
        self.min_pos += disp;
        self.max_pos += disp;
    }
}

impl<const DIM: usize, T> AABB<DIM, T>
where
    T: Copy + Default + PartialOrd,
{
    /// merges curent aabbs with `other` so that the resulting AABB contains the minimal area AABB such that points from both `self` and `other`
    pub fn merge(&mut self, other: AABB<DIM, T>) {
        let min = |a, b| match a < b {
            true => a,
            false => b,
        };

        let max = |a, b| match a > b {
            true => a,
            false => b,
        };

        for k in 0..DIM {
            self.max_pos[k] = max(self.max_pos[k], other.max_pos[k]);
            self.min_pos[k] = min(self.min_pos[k], other.min_pos[k]);
        }
    }
}

impl<const DIM: usize> AABB<DIM, f32> {
    /// AABB that contains all points in R^n  
    pub fn infinity() -> Self {
        Self {
            min_pos: Vector::from_array([f32::NEG_INFINITY; DIM]),
            max_pos: Vector::from_array([f32::INFINITY; DIM]),
        }
    }

    /// AABB that contains negative dimentions useful for merge
    pub fn flipped_infinity() -> Self {
        Self {
            min_pos: Vector::from_array([f32::INFINITY; DIM]),
            max_pos: Vector::from_array([f32::NEG_INFINITY; DIM]),
        }
    }
}

#[rustfmt::skip]
impl<T> AABB2<T> 
where 
    T:Copy 
{
    pub fn x(&self) -> T { self.min_pos[0] }
    pub fn y(&self) -> T { self.min_pos[1] }
}

#[rustfmt::skip]
impl<T> AABB2<T>
where
    T: Copy + Sub<Output = T>,
{
    pub fn w(&self) -> T { self.max_pos[0] - self.min_pos[0] }
    pub fn h(&self) -> T { self.max_pos[1] - self.min_pos[1] }
}

#[rustfmt::skip]
impl<T> AABB3<T>
where
    T:Copy 
{
    pub fn x(&self) -> T { self.min_pos[0] }
    pub fn y(&self) -> T { self.min_pos[1] }
    pub fn z(&self) -> T { self.min_pos[2] }
}

#[rustfmt::skip]
impl<T> AABB3<T>
where
    T: Sub<Output = T> + Copy,
{
    pub fn w(&self) -> T { self.max_pos[0] - self.min_pos[0] }
    pub fn h(&self) -> T { self.max_pos[1] - self.min_pos[1] }
    pub fn d(&self) -> T { self.max_pos[2] - self.min_pos[2] }

}

#[test]
fn verts_contained_sanity() {
    let big = AABB2::from_point_and_lengths(Vec2::from([30.0, 30.0]), Vec2::from([100.0; 2]));
    let small = AABB2::from_point_and_lengths(Vec2::from([30.1; 2]), Vec2::from([98.0; 2]));

    println!("big verts");
    big.verts().for_each(|e| println!("{e}"));

    println!("small verts");
    small.verts().for_each(|e| println!("{e}"));

    let big_fully_contains_small = big.fully_contains(small);
    println!("big fully contains small =  {}", big_fully_contains_small);

    assert!(big_fully_contains_small)
}
