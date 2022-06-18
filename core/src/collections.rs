use serde::{Serialize,Deserialize};

//custom data structures I've written for various purposes
pub mod nary_forest;
pub mod linked_list;
pub mod binary_tree;
pub mod bitarray;
pub mod fixed_stack;
pub mod segment_tree;


/// ## Description
/// The Official Pseudo Pointer type for this module
/// ### comments
/// - Most of the collections are vector-based because they are:
///     - Easy to serialize 
///     - Cache local
///     - less likely to segfault when bounds checks are off
///     - easier to implement
#[derive(Copy, Clone, PartialEq, Debug,Hash, Eq, Serialize,Deserialize)]
pub struct Ptr {
    idx: u64,
}
impl Ptr{
    pub fn as_usize(&self)->usize{
        self.idx as usize
    }
}
impl Default for Ptr {
    fn default() -> Self {
        Self::null()
    }
}
impl Ptr {
    pub const fn null() -> Self {
        Self { idx: !0 }
    }
}
impl From<usize> for Ptr {
    fn from(idx: usize) -> Self {
        Self { idx: idx as u64 }
    }
}