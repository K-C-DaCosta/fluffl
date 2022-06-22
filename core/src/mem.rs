pub unsafe fn force_ptr_to_ref_mut<T>(ptr: *const T) -> &'static mut T {
    &mut *(ptr as *mut T)
}

pub unsafe fn force_static<'a, T>(reference: &'a T) -> &'static T {
    &*(reference as *const T)
}
pub unsafe fn force_static_mut<'a, T>(reference: &'a T) -> &'static mut T {
    &mut *(reference as *const T as *mut T)
}