use std::{
    fmt::{Debug, Display},
    ops::*,
};

mod fp32;

mod fp64;

pub use self::{fp32::*, fp64::FP64};
