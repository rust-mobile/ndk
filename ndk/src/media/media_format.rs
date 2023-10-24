//! Bindings for [`AMediaFormat`]
//!
//! [`AMediaFormat`]: https://developer.android.com/ndk/reference/group/media#amediaformat

use std::{
    ffi::{CStr, CString},
    fmt,
    ptr::{self, NonNull},
    slice,
};

use crate::media_error::MediaError;

/// A native [`AMediaFormat *`]
///
/// [`AMediaFormat *`]: https://developer.android.com/ndk/reference/group/media#amediaformat
#[derive(Debug)]
pub struct MediaFormat {
    inner: NonNull<ffi::AMediaFormat>,
}

impl fmt::Display for MediaFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c_str = unsafe { CStr::from_ptr(ffi::AMediaFormat_toString(self.as_ptr())) };
        f.write_str(c_str.to_str().unwrap())
    }
}

impl Default for MediaFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaFormat {
    /// Assumes ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::AMediaFormat`].
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AMediaFormat>) -> Self {
        Self { inner: ptr }
    }

    pub fn as_ptr(&self) -> *mut ffi::AMediaFormat {
        self.inner.as_ptr()
    }

    pub fn new() -> Self {
        Self {
            inner: NonNull::new(unsafe { ffi::AMediaFormat_new() }).unwrap(),
        }
    }

    pub fn i32(&self, key: &str) -> Option<i32> {
        let name = CString::new(key).unwrap();
        let mut out = 0;
        if unsafe { ffi::AMediaFormat_getInt32(self.as_ptr(), name.as_ptr(), &mut out) } {
            Some(out)
        } else {
            None
        }
    }

    pub fn i64(&self, key: &str) -> Option<i64> {
        let name = CString::new(key).unwrap();
        let mut out = 0;
        if unsafe { ffi::AMediaFormat_getInt64(self.as_ptr(), name.as_ptr(), &mut out) } {
            Some(out)
        } else {
            None
        }
    }

    pub fn f32(&self, key: &str) -> Option<f32> {
        let name = CString::new(key).unwrap();
        let mut out = 0.0;
        if unsafe { ffi::AMediaFormat_getFloat(self.as_ptr(), name.as_ptr(), &mut out) } {
            Some(out)
        } else {
            None
        }
    }

    pub fn usize(&self, key: &str) -> Option<usize> {
        let name = CString::new(key).unwrap();
        let mut out = 0;
        if unsafe { ffi::AMediaFormat_getSize(self.as_ptr(), name.as_ptr(), &mut out) } {
            Some(out)
        } else {
            None
        }
    }

    pub fn buffer(&self, key: &str) -> Option<&[u8]> {
        let name = CString::new(key).unwrap();
        let mut out_buffer = ptr::null_mut();
        let mut out_size = 0;
        unsafe {
            ffi::AMediaFormat_getBuffer(
                self.as_ptr(),
                name.as_ptr(),
                &mut out_buffer,
                &mut out_size,
            )
        }
        .then(|| unsafe { slice::from_raw_parts(out_buffer.cast(), out_size) })
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        let name = CString::new(key).unwrap();
        let mut out = ptr::null();
        unsafe { ffi::AMediaFormat_getString(self.as_ptr(), name.as_ptr(), &mut out) }
            .then(|| unsafe { CStr::from_ptr(out) }.to_str().unwrap())
    }

    pub fn set_i32(&self, key: &str, value: i32) {
        let name = CString::new(key).unwrap();
        unsafe { ffi::AMediaFormat_setInt32(self.as_ptr(), name.as_ptr(), value) };
    }

    pub fn set_i64(&self, key: &str, value: i64) {
        let name = CString::new(key).unwrap();
        unsafe { ffi::AMediaFormat_setInt64(self.as_ptr(), name.as_ptr(), value) };
    }

    pub fn set_f32(&self, key: &str, value: f32) {
        let name = CString::new(key).unwrap();
        unsafe { ffi::AMediaFormat_setFloat(self.as_ptr(), name.as_ptr(), value) };
    }

    pub fn set_str(&self, key: &str, value: &str) {
        let name = CString::new(key).unwrap();
        let c_string = CString::new(value).unwrap();
        unsafe { ffi::AMediaFormat_setString(self.as_ptr(), name.as_ptr(), c_string.as_ptr()) };
    }

    pub fn set_buffer(&self, key: &str, value: &[u8]) {
        let name = CString::new(key).unwrap();
        unsafe {
            ffi::AMediaFormat_setBuffer(
                self.as_ptr(),
                name.as_ptr(),
                value.as_ptr().cast(),
                value.len(),
            )
        };
    }

    #[cfg(feature = "api-level-28")]
    pub fn f64(&self, key: &str) -> Option<f64> {
        let name = CString::new(key).unwrap();
        let mut out = 0.0;
        if unsafe { ffi::AMediaFormat_getDouble(self.as_ptr(), name.as_ptr(), &mut out) } {
            Some(out)
        } else {
            None
        }
    }

    /// Returns (left, top, right, bottom)
    #[cfg(feature = "api-level-28")]
    pub fn rect(&self, key: &str) -> Option<(i32, i32, i32, i32)> {
        let name = CString::new(key).unwrap();
        let mut left = 0;
        let mut top = 0;
        let mut right = 0;
        let mut bottom = 0;
        if unsafe {
            ffi::AMediaFormat_getRect(
                self.as_ptr(),
                name.as_ptr(),
                &mut left,
                &mut top,
                &mut right,
                &mut bottom,
            )
        } {
            Some((left, top, right, bottom))
        } else {
            None
        }
    }

    #[cfg(feature = "api-level-28")]
    pub fn set_f64(&self, key: &str, value: f64) {
        let name = CString::new(key).unwrap();
        unsafe { ffi::AMediaFormat_setDouble(self.as_ptr(), name.as_ptr(), value) };
    }

    #[cfg(feature = "api-level-28")]
    pub fn set_rect(&self, key: &str, left: i32, top: i32, right: i32, bottom: i32) {
        let name = CString::new(key).unwrap();
        unsafe {
            ffi::AMediaFormat_setRect(self.as_ptr(), name.as_ptr(), left, top, right, bottom)
        };
    }

    #[cfg(feature = "api-level-28")]
    pub fn set_usize(&self, key: &str, value: usize) {
        let name = CString::new(key).unwrap();
        unsafe { ffi::AMediaFormat_setSize(self.as_ptr(), name.as_ptr(), value) };
    }
}

impl Drop for MediaFormat {
    fn drop(&mut self) {
        let status = unsafe { ffi::AMediaFormat_delete(self.as_ptr()) };
        MediaError::from_status(status).unwrap();
    }
}
