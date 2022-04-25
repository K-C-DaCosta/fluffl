use super::*; 

#[derive(Copy, Clone, Debug)]
pub struct GlobalIndex {
    pub idx: usize,
}
impl From<usize> for GlobalIndex {
    fn from(idx: usize) -> Self {
        Self { idx }
    }
}
impl Default for GlobalIndex {
    fn default() -> Self {
        Self{
            idx:!0, 
        }
    }
}

#[derive(Copy, Clone,Debug)]
pub struct BucketIndex {
    pub idx: usize,
}
impl BucketIndex {
    pub fn from_usize(i: usize) -> Self {
        Self { idx: i }
    }
}
impl Deref for BucketIndex {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.idx
    }
}