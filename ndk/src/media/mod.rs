//! Bindings for the NDK media classes.
//!
//! See also [the NDK docs](https://developer.android.com/ndk/reference/group/media)
#![cfg(any(feature = "media", feature = "midi"))]

mod error;

#[cfg(feature = "media")]
pub mod image_reader;
#[cfg(feature = "media")]
pub mod media_codec;

pub use error::NdkMediaError;
use std::mem::MaybeUninit;

#[cfg(feature = "media")]
use std::ptr::NonNull;

pub type Result<T, E = NdkMediaError> = std::result::Result<T, E>;

pub(crate) fn construct<T>(with_ptr: impl FnOnce(*mut T) -> ffi::media_status_t) -> Result<T> {
    let mut result = MaybeUninit::uninit();
    let status = with_ptr(result.as_mut_ptr());
    NdkMediaError::from_status(status).map(|()| unsafe { result.assume_init() })
}

#[cfg(feature = "media")]
fn construct_never_null<T>(
    with_ptr: impl FnOnce(*mut *mut T) -> ffi::media_status_t,
) -> Result<NonNull<T>> {
    let result = construct(with_ptr)?;
    let non_null = if cfg!(debug_assertions) {
        NonNull::new(result).expect("result should never be null")
    } else {
        unsafe { NonNull::new_unchecked(result) }
    };
    Ok(non_null)
}

/// Function is not expected to ever return `null`, but this
/// cannot be validated through the Android documentation.
///
/// As such this function always asserts on `null` values,
/// even when `cfg!(debug_assertions)` is disabled.
#[cfg(feature = "media")]
fn get_unlikely_to_be_null<T>(get_ptr: impl FnOnce() -> *mut T) -> NonNull<T> {
    let result = get_ptr();
    NonNull::new(result).expect("result should never be null")
}
