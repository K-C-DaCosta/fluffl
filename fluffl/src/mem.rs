pub unsafe fn force_ptr_to_ref_mut<T>(ptr: *const T) -> &'static mut T {
    &mut *(ptr as *mut T)
}

#[allow(dead_code,clippy::needless_lifetimes)]
pub unsafe fn force_static<'a, T>(reference: &'a T) -> &'static T {
    &*(reference as *const T)
}

/// forces the reference to be seen as 'static', essentially disables the borrow checker
#[allow(clippy::needless_lifetimes)]
pub unsafe fn force_static_mut<'a, T>(reference: &'a mut T) -> &'static mut T {
    &mut *(reference as *mut T as *mut T)
}

/// basically clones a mutable reference with a different lifetime
#[allow(clippy::needless_lifetimes)]
pub unsafe fn force_borrow_mut<'a, 'b, T>(reference: &'a mut T) -> &'b mut T {
    &mut *(reference as *mut T as *mut T)
}
