use super::*;

pub type MatStack<T> = MatrixStack<32, T>;

/// # Description
/// Used for scene-graphs and GUI stuff  
pub struct MatrixStack<const N: usize, T> {
    cursor: usize,
    stack: [Mat4<T>; N],
}

impl<const N: usize, T> MatrixStack<N, T>
where
    T: Copy + Default + HasConstants + Mul<Output = T> + Add<Output = T> + AddAssign,
{
    pub fn new() -> Self {
        Self {
            cursor: 1,
            stack: [Mat4::identity(); N],
        }
    }

    pub fn is_empty(&self)->bool{
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.cursor - 1
    }

    pub fn clear(&mut self) {
        self.cursor = 1;
    }

    pub fn is_full(&self) -> bool {
        self.cursor >= N
    }

    pub fn peek(&self) -> &Mat4<T> {
        &self.stack[self.cursor - 1]
    }

    pub fn peek_mut(&mut self) -> &mut Mat4<T> {
        &mut self.stack[self.cursor - 1]
    }

    pub fn push(&mut self, mat: Mat4<T>) -> bool {
        let prev = self.stack[self.cursor - 1];
        self.stack[self.cursor] = prev * mat;
        self.cursor = (self.cursor + 1).min(N - 1);
        true
    }

    /// pop `num_pops` times
    pub fn pop_multi(&mut self, num_pops: usize) {
        self.cursor = (self.cursor as isize - num_pops as isize).max(1) as usize;
    }

    pub fn pop(&mut self) -> Mat4<T> {
        let prev = self.stack[self.cursor - 1];
        self.cursor = (self.cursor - 1).max(1);
        prev
    }

    pub fn iter(&self) -> impl Iterator<Item = &Mat4<T>> {
        self.stack.iter().take(self.cursor)
    }
}

impl<const N: usize, T> Default for MatrixStack<N, T>
where
    T: Copy + Default + HasConstants + Mul<Output = T> + Add<Output = T> + AddAssign,
{
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn sanity() {
    let mut stack = MatStack::<f32>::new();
    stack.push(translate4(Vec4::from_array([1.0, 0., 5., 0.])));
    stack.push(translate4(Vec4::from_array([25.0, 0., 10., 0.])));
    stack.push(scale4(Vec4::from_array([2.0, 2.0, 2.0, 0.0])));

    for mat in stack.iter() {
        println!("{}", mat);
    }
}
