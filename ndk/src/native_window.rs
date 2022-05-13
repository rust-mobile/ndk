//! Bindings for [`ffi::ANativeWindow`]

use jni_sys::{jobject, JNIEnv};
use raw_window_handle::{AndroidNdkHandle, HasRawWindowHandle, RawWindowHandle};
use std::{ffi::c_void, ptr::NonNull};

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

unsafe impl HasRawWindowHandle for NativeWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = AndroidNdkHandle::empty();
        handle.a_native_window = self.ptr.as_ptr() as *mut c_void;
        RawWindowHandle::AndroidNdk(handle)
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

    pub fn height(&self) -> i32 {
        unsafe { ffi::ANativeWindow_getHeight(self.ptr.as_ptr()) }
    }

    pub fn width(&self) -> i32 {
        unsafe { ffi::ANativeWindow_getWidth(self.ptr.as_ptr()) }
    }

    /// Return the [`NativeWindow`] associated with a JNI [`android.view.Surface`] pointer.
    ///
    /// # Safety
    /// By calling this function, you assert that `env` is a valid pointer to a [`JNIEnv`] and
    /// `surface` is a valid pointer to an [`android.view.Surface`].
    ///
    /// [`android.view.Surface`]: https://developer.android.com/reference/android/view/Surface
    pub unsafe fn from_surface(env: *mut JNIEnv, surface: jobject) -> Option<Self> {
        let ptr = ffi::ANativeWindow_fromSurface(env, surface);
        Some(Self::from_ptr(NonNull::new(ptr)?))
    }

    /// Return a JNI [`android.view.Surface`] pointer derived from this [`NativeWindow`].
    ///
    /// # Safety
    /// By calling this function, you assert that `env` is a valid pointer to a [`JNIEnv`].
    ///
    /// [`android.view.Surface`]: https://developer.android.com/reference/android/view/Surface
    #[cfg(feature = "api-level-26")]
    pub unsafe fn to_surface(&self, env: *mut JNIEnv) -> jobject {
        ffi::ANativeWindow_toSurface(env, self.ptr().as_ptr())
    }
}
