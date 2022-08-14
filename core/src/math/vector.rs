use std::{
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Mul, MulAssign, Sub},
};

pub type Vec2<T> = Vector<2, T>;
pub type Vec3<T> = Vector<3, T>;
pub type Vec4<T> = Vector<4, T>;

#[derive(Copy, Clone, Debug)]
pub struct Vector<const N: usize, T> {
    data: [T; N],
}

impl<const N: usize, T> Display for Vector<N, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl<const N: usize, T> Vector<N, T> {
    pub fn from_array(data: [T; N]) -> Self {
        Self { data }
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
    T: Default + Mul<Output = T> + Add<Output = T> + Copy,
{
    type Output = T;

    fn mul(self, rhs: Self) -> Self::Output {
        let dot = self
            .data
            .iter()
            .zip(rhs.data)
            .fold(T::default(), |acc, (&lhs, rhs)| acc + (lhs * rhs));

        dot
    }
}

impl<const N: usize, T> MulAssign<T> for Vector<N, T>
where
    T: Default + Mul<Output = T> + MulAssign + Copy,
{
    fn mul_assign(&mut self, rhs: T) {
        self.data.iter_mut().for_each(|comp| *comp *= rhs);
    }
}
