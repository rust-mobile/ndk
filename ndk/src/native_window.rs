//! Bindings for [`ffi::ANativeWindow`]
use std::ptr::NonNull;

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeWindow {
    ptr: NonNull<ffi::ANativeWindow>,
}

unsafe impl Send for NativeWindow {}
unsafe impl Sync for NativeWindow {}

impl Drop for NativeWindow {
    fn drop(&mut self) {
        unsafe { ffi::ANativeWindow_release(self.ptr.as_ptr()) }
    }
}

impl Clone for NativeWindow {
    fn clone(&self) -> Self {
        unsafe {
            ffi::ANativeWindow_acquire(self.ptr.as_ptr());
            Self { ptr: self.ptr }
        }
    }
}

impl NativeWindow {
    /// Assumes ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::ANativeWindow`].
    pub unsafe fn from_ptr(ptr: NonNull<ffi::ANativeWindow>) -> Self {
        Self { ptr }
    }

    /// Acquires ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::ANativeWindow`].
    pub unsafe fn clone_from_ptr(ptr: NonNull<ffi::ANativeWindow>) -> Self {
        ffi::ANativeWindow_acquire(ptr.as_ptr());
        Self::from_ptr(ptr)
    }

    pub fn ptr(&self) -> NonNull<ffi::ANativeWindow> {
        self.ptr
    }
}

impl NativeWindow {
    pub fn height(&self) -> i32 {
        unsafe { ffi::ANativeWindow_getHeight(self.ptr.as_ptr()) }
    }

    pub fn width(&self) -> i32 {
        unsafe { ffi::ANativeWindow_getWidth(self.ptr.as_ptr()) }
    }
}
