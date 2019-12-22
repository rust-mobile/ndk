//! Bindings for `ANativeWindow`
use std::ptr::NonNull;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeWindow {
    ptr: NonNull<ffi::ANativeWindow>,
}

unsafe impl Send for NativeWindow {}
unsafe impl Sync for NativeWindow {}

impl NativeWindow {
    pub unsafe fn from_ptr(ptr: NonNull<ffi::ANativeWindow>) -> Self {
        Self { ptr }
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
