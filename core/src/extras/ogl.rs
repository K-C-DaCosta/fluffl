//Note:This module basically a bunch of helper code to make dealing with opengl easier.
//It was not inteneded to be used by others.
use glow::*;

pub mod array;
pub mod buffer;
pub mod program;
pub mod texture;

type FixedString = ArrayString<[u8; 32]>;
use arrayvec::{ArrayString, ArrayVec};
use std::collections::HashMap;

pub struct OglIncomplete<T> {
    pub inner: T,
}

impl<T> OglIncomplete<T> {
    pub fn new(obj: T) -> Self {
        Self { inner: obj }
    }
}

trait BoolMap<F, T>
where
    F: FnMut() -> T,
{
    fn map(self, closure: F) -> Option<T>;
}

impl<F, T> BoolMap<F, T> for bool
where
    F: FnMut() -> T,
{
    fn map(self, mut closure: F) -> Option<T> {
        if self {
            Some(closure())
        } else {
            None
        }
    }
}

pub trait HasData {
    fn raw_bytes(&self) -> &[u8];
    fn raw_bytes_mut(&mut self) -> &mut [u8];
    fn set_by_byte(&mut self, val: u8) {
        self.raw_bytes_mut().iter_mut().for_each(|byte| *byte = val);
    }
    fn zero(&mut self) {
        self.set_by_byte(0)
    }
}

pub trait Bindable {
    /// binds object\
    /// if `opt` is true then object is bound otherwise object is 'unbound'
    fn bind(&self, opt: bool);
}

impl<T> HasData for Vec<T>
where
    T: Copy + Default + Sized,
{
    fn raw_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ptr() as *const u8,
                self.len() * std::mem::size_of::<T>(),
            )
        }
    }
    fn raw_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_ptr() as *mut u8,
                self.len() * std::mem::size_of::<T>(),
            )
        }
    }
}
