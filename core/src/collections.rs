use serde::{Deserialize, Serialize};
use std::fmt::Display;

//custom data structures I've written for various purposes
pub mod binary_tree;
pub mod bitarray;
pub mod fixed_stack;
pub mod linked_list;
pub mod nary_forest;
pub mod flat_nary_tree; 
pub mod segment_tree;

/// ## Description
/// The Official Pseudo Pointer type for this module
/// ### comments
/// - Most of the collections are vector-based or 'arena based' because they are:
///     - Easy to serialize
///     - Cache local
///     - less likely to segfault when bounds checks are off
///     - easier to implement
#[derive(Copy, Clone, PartialEq, Debug, Hash, Eq, Serialize, Deserialize)]
pub struct Ptr {
    idx: u64,
}
impl Ptr {
    pub fn as_usize(&self) -> usize {
        self.idx as usize
    }

    pub const fn null() -> Self {
        Self { idx: !0 }
    }
    
    pub fn is_null(&self) -> bool{
        *self == Self::null()
    }
}
impl Default for Ptr {
    fn default() -> Self {
        Self::null()
    }
}
impl From<usize> for Ptr {
    fn from(idx: usize) -> Self {
        Self { idx: idx as u64 }
    }
}

impl Display for Ptr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.idx)
    }
}
