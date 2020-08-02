//! Bindings for the NDK tracing API.
//!
//! See also [the NDK docs](https://developer.android.com/ndk/reference/group/tracing)
#![cfg(feature = "trace")]
use std::ffi::{CString, NulError};

pub fn is_trace_enabled() -> bool {
    unsafe { ffi::ATrace_isEnabled() }
}

#[derive(Debug)]
pub struct Section {
    _private: *mut (),
}

impl Section {
    pub fn new(name: &str) -> Result<Self, NulError> {
        let section_name = CString::new(name)?;
        unsafe { ffi::ATrace_beginSection(section_name.as_ptr()) };

        Ok(Section {
            _private: std::ptr::null_mut(),
        })
    }

    pub fn end(self) {
        std::mem::drop(self)
    }
}

impl Drop for Section {
    fn drop(&mut self) {
        unsafe { ffi::ATrace_endSection() };
    }
}

/// Unique identifier for distinguishing simultaneous events
#[derive(Debug)]
#[cfg(feature = "api-level-29")]
pub struct Cookie(pub i32);

#[derive(Debug)]
#[cfg(feature = "api-level-29")]
pub struct AsyncSection {
    section_name: CString,
    cookie: Cookie,
    _private: *mut (),
}

#[cfg(feature = "api-level-29")]
unsafe impl Send for AsyncSection {}

#[cfg(feature = "api-level-29")]
impl AsyncSection {
    pub fn new(name: &str, cookie: Cookie) -> Result<Self, NulError> {
        let section_name = CString::new(name)?;
        unsafe { ffi::ATrace_beginAsyncSection(section_name.as_ptr(), cookie.0) };

        Ok(AsyncSection {
            section_name,
            cookie,
            _private: std::ptr::null_mut(),
        })
    }

    pub fn end(self) {
        std::mem::drop(self)
    }
}

#[cfg(feature = "api-level-29")]
impl Drop for AsyncSection {
    fn drop(&mut self) {
        unsafe { ffi::ATrace_endAsyncSection(self.section_name.as_ptr(), self.cookie.0) };
    }
}

#[cfg(feature = "api-level-29")]
#[derive(Debug)]
pub struct Counter {
    name: CString,
}

#[cfg(feature = "api-level-29")]
impl Counter {
    pub fn new(name: &str) -> Result<Self, NulError> {
        let name = CString::new(name)?;
        Ok(Self { name })
    }

    pub fn set_value(&self, value: i64) {
        unsafe { ffi::ATrace_setCounter(self.name.as_ptr(), value) }
    }
}
