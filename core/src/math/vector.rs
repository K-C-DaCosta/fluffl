use std::ops::{Add, AddAssign, Mul, MulAssign, Sub};

pub type Vec2<T> = Vec<2, T>;
pub type Vec3<T> = Vec<3, T>;

#[derive(Copy, Clone)]
pub struct Vec<const N: usize, T> {
    data: [T; N],
}

impl<const N: usize, T> Vec<N, T> {
    pub fn from_array(data: [T; N]) -> Self {
        Self { data }
    }
}

impl<const N: usize, T> Add for Vec<N, T>
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

impl<const N: usize, T> Sub for Vec<N, T>
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

impl<const N: usize, T> Mul for Vec<N, T>
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

impl<const N: usize, T> MulAssign<T> for Vec<N, T>
where
    T: Default + Mul<Output = T> + MulAssign + Copy,
{
    fn mul_assign(&mut self, rhs: T) {
        self.data.iter_mut().for_each(|comp| *comp *= rhs);
    }
}
