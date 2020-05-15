//! Bindings for the NDK media classes.
//!
//! See also [the NDK docs](https://developer.android.com/ndk/reference/group/media)
#![cfg(feature = "media")]

mod error;
pub mod image_reader;

pub use error::NdkMediaError;
use std::{mem::MaybeUninit, ptr::NonNull};

pub type Result<T, E = NdkMediaError> = std::result::Result<T, E>;

fn construct<T>(with_ptr: impl FnOnce(*mut T) -> ffi::camera_status_t) -> Result<T> {
    let mut result = MaybeUninit::uninit();
    let status = with_ptr(result.as_mut_ptr());
    NdkMediaError::from_status(status, || unsafe { result.assume_init() })
}

fn construct_never_null<T>(
    with_ptr: impl FnOnce(*mut *mut T) -> ffi::camera_status_t,
) -> Result<NonNull<T>> {
    let result = construct(with_ptr)?;
    let non_null = if cfg!(debug_assertions) {
        NonNull::new(result).expect("result should never be null")
    } else {
        unsafe { NonNull::new_unchecked(result) }
    };
    Ok(non_null)
}
