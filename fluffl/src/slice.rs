use std::ops::{Index, IndexMut, Range};

/// represents a contigious range in an array or slice that is stored elsewhere.
/// This range has documented operations to make certain parts of my code more readable
#[derive(Copy, Clone, Default, Debug)]
pub struct IdxSlice {
    lbound: usize,
    len: usize,
}

impl IdxSlice {
    pub fn new(lbound: usize) -> Self {
        Self { lbound, len: 0 }
    }

    pub fn pop_front(&mut self, off: usize) {
        self.lbound += off;
        self.len -= off;
    }

    pub fn push_rear(&mut self, off: usize) {
        self.len += off;
    }

    pub fn lbound(&self) -> usize {
        self.lbound
    }

    pub fn ubound_exclusive(&self) -> usize {
        self.lbound + self.len
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// equivalent to poping the front and pushing the rear
    pub fn shift(&mut self, off: usize) {
        self.lbound += off;
    }

    pub fn as_range(&self) -> Range<usize> {
        Range {
            start: self.lbound,
            end: self.lbound + self.len,
        }
    }

    pub fn is_in_range(&self, idx: usize) -> bool {
        idx >= self.lbound && (idx - self.lbound) < self.len
    }

    pub fn is_in_range_include_upper_bound(&self, idx: usize) -> bool {
        idx >= self.lbound && (idx - self.lbound) <= self.len
    }

    pub fn get_slice<'a, A, B>(&self, sliceable: &'a A) -> &'a B
    where
        A: ?Sized + Index<Range<usize>, Output = B>,
        B: ?Sized,
    {
        &sliceable[self.as_range()]
    }

    pub fn get_slice_mut<'a, A, B>(&self, sliceable: &'a mut A) -> &'a mut B
    where
        A: ?Sized + Index<Range<usize>, Output = B> + IndexMut<Range<usize>>,
        B: ?Sized,
    {
        &mut sliceable[self.as_range()]
    }
}
