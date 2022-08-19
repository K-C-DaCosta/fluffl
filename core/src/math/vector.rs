use std::{
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Deref, DerefMut, Div, Index, IndexMut, Mul, MulAssign, Sub},
};

use super::HasScalar;

pub type Vec2<T> = Vector<2, T>;
pub type Vec3<T> = Vector<3, T>;
pub type Vec4<T> = Vector<4, T>;

#[derive(Copy, Clone, Debug)]
pub struct Vector<const N: usize, T> {
    data: [T; N],
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
    T: Default + Copy + HasScalar,
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

    pub fn to_pos<const M: usize>(b: Vector<M, T>) -> Self {
        let mut res = Self::zero();
        for k in 0..M.min(N - 1) {
            res[k] = b[k];
        }
        res[N - 1] = T::one();
        res
    }

    pub fn from_array(data: [T; N]) -> Self {
        Self { data }
    }
}

impl<T> Vector<4, T>
where
    T: HasScalar + Copy + Default + Div<Output = T> + Mul<Output = T>,
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

impl<const N: usize, T> Mul<T> for Vector<N, T>
where
    T: Default + Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let mut data = [T::default(); N];
        data.iter_mut()
            .zip(self.data.iter())
            .for_each(|(res, &lhs)| {
                *res = lhs * rhs;
            });
        Self { data }
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

/// ## Description
/// Used to write vector components into arrays
pub struct ComponentWriter<'a, T, Len, Push> {
    cursor: usize,
    data: &'a mut T,
    len_cb: Len,
    push_cb: Option<Push>,
}

impl<'a, T, M, Len, Push> ComponentWriter<'a, T, Len, Push>
where
    Push: FnMut(&mut T, M) + Copy,
    Len: Fn(&T) -> usize + Copy,
    T: Index<usize, Output = M> + IndexMut<usize, Output = M>,
    M: Copy + Default,
{
    pub fn new(data: &'a mut T, len_cb: Len, push_cb: Option<Push>) -> Self {
        Self {
            cursor: 0,
            data,
            len_cb,
            push_cb,
        }
    }

    pub fn seek(&mut self, from_start: usize) {
        let len = self.len_cb;
        self.cursor = from_start.clamp(0, len(self.data));
    }

    pub fn write<const N: usize>(&mut self, vec: &Vector<N, M>) -> usize {
        match self.push_cb {
            Some(push) => self.write_pushable(push, vec),
            None => self.write_cant_push(vec),
        }
    }

    fn write_cant_push<const N: usize>(&mut self, vec: &Vector<N, M>) -> usize {
        let len = self.len_cb;
        let mut k = 0;
        while (k < N) && self.cursor < len(self.data) {
            self.data[self.cursor] = vec[k];
            self.cursor += 1;
            k += 1;
        }
        k
    }

    fn write_pushable<const N: usize>(&mut self, mut push: Push, vec: &Vector<N, M>) -> usize {
        let len = self.len_cb;
        for k in 0..N {
            if self.cursor >= len(self.data) {
                push(self.data, M::default())
            }
            self.data[self.cursor] = vec[k];
            self.cursor += 1;
        }
        N
    }
    pub fn done(self) {}
}

impl<'a, T> From<&'a mut Vec<T>>
    for ComponentWriter<'a, Vec<T>, fn(&Vec<T>) -> usize, fn(&mut Vec<T>, T)>
where
    T: Copy + Default,
{
    fn from(obj: &'a mut Vec<T>) -> Self {
        Self::new(
            obj,
            |vec: &Vec<_>| vec.len(),
            Some(|vec, item| vec.push(item)),
        )
    }
}

impl<'a, T> From<&'a mut [T]> for ComponentWriter<'a, [T], fn(&[T]) -> usize, fn(&mut [T], T)>
where
    [T]: Sized,
    T: Copy + Default,
{
    fn from(obj: &'a mut [T]) -> Self {
        Self::new(obj, |vec: &[_]| vec.len(), None)
    }
}

#[test]
pub fn writer_test() {
    let mut list = Vec::<f32>::new();
    let mut writer = ComponentWriter::from(&mut list);
    writer.write(&Vec4::from_array([1.0f32, 0.2, 0.3, 0.4]));
    writer.write(&Vec4::from_array([2.0f32, 3.2, -0.3, 9.]));
    println!("{:?}", list);
}

#[test]
pub fn color_test() {
    use super::FP32;
    let x = Vec4::<f32>::rgba_u32(0xffffffff);
    println!("{}", x);
}

#[test]
pub fn position_test() {
    let a = Vec2::<f32>::from_array([0.1, 0.2]);
    let b = Vec4::to_pos(a);
    println!("{}\nto\n{}", a, b);

    let a = Vec3::<f32>::from_array([3., 5., 9.5]);
    let b = Vec4::to_pos(a);
    println!("{}\nto\n{}", a, b);
}
