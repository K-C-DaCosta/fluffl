use super::*; 

pub trait CanSwap: Debug {
    fn ext_swap(&mut self, a: usize, b: usize);
    fn print(&self) {
        println!("{:?}", self);
    }
}

#[derive(Debug)]
pub struct Swappable<ArrKind> {
    array: ArrKind,
}

impl<ArrKind> Swappable<ArrKind> {
    pub fn new(array: ArrKind) -> Self {
        Self { array }
    }
}

impl<T: Debug> CanSwap for Swappable<&mut Vec<T>> {
    fn ext_swap(&mut self, a: usize, b: usize) {
        self.array.swap(a, b);
    }
}
impl<T: Debug> CanSwap for Swappable<&mut [T]> {
    fn ext_swap(&mut self, a: usize, b: usize) {
        self.array.swap(a, b);
    }
}

