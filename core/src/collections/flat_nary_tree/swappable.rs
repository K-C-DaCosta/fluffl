use super::*; 

pub trait CanSwap {
    fn ext_swap(&mut self, a: usize, b: usize);
}

pub struct Swappable<ArrKind> {
    array: ArrKind,
}

impl<ArrKind> Swappable<ArrKind> {
    pub fn new(array: ArrKind) -> Self {
        Self { array }
    }
}

impl<T> CanSwap for Swappable<&mut Vec<T>> {
    fn ext_swap(&mut self, a: usize, b: usize) {
        self.array.swap(a, b);
    }
}
impl<T> CanSwap for Swappable<&mut [T]> {
    fn ext_swap(&mut self, a: usize, b: usize) {
        self.array.swap(a, b);
    }
}

