#![allow(dead_code)]

use std::ops::Index;

pub struct FixedStack<const N: usize, V> {
    memory: [V; N],
    cursor: usize,
    len: usize,
}
impl<const N: usize, V> FixedStack<N, V>
where
    V: Default + Copy,
{
    pub fn new() -> Self {
        Self {
            memory: [V::default(); N],
            len: 0,
            cursor: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn peek(&self) -> V {
        self.memory[(self.cursor as isize - 1).max(0) as usize]
    }

    pub fn push(&mut self, v: V) {
        self.memory[self.cursor.min(N - 1)] = v;
        self.cursor = (self.cursor + 1).min(N - 1);
        self.len += 1;
    }

    pub fn pop_multi(&mut self, num_pops: usize) {
        self.cursor = (self.cursor as isize - num_pops as isize).max(0) as usize;
        self.len = (self.len as isize - num_pops as isize).max(0) as usize;
    }

    pub fn pop(&mut self) -> Option<V> {
        (self.cursor > 0).then(|| {
            let popped_val = self.memory[self.cursor - 1];
            self.cursor -= 1;
            self.len -= 1;
            popped_val
        })
    }
}
impl<const N: usize, V: Copy> Index<usize> for FixedStack<N, V> {
    type Output = V;
    fn index(&self, index: usize) -> &Self::Output {
        &self.memory[index]
    }
}
impl<const N: usize, V: Copy> std::ops::IndexMut<usize> for FixedStack<N, V> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.memory[index]
    }
}
