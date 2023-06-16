//! Bindings for the NDK media classes.
//!
//! See also [the NDK docs](https://developer.android.com/ndk/reference/group/media)
#![cfg(feature = "media")]

pub mod image_reader;
pub mod media_codec;

use crate::media_error::{construct, NdkMediaError, Result};
use std::ptr::NonNull;

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
