//! Bindings for [`AndroidBitmap`] functions
//!
//! These functions operate directly on a JNI [`android.graphics.Bitmap`] instance.
//!
//! [`AndroidBitmap`]: https://developer.android.com/ndk/reference/group/bitmap
//! [`android.graphics.Bitmap`]: https://developer.android.com/reference/android/graphics/Bitmap
#![cfg(feature = "bitmap")]

use jni_sys::{jobject, JNIEnv};
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use std::mem::MaybeUninit;

#[cfg(feature = "api-level-30")]
use crate::hardware_buffer::HardwareBufferRef;

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BitmapError {
    Unknown,
    AllocationFailed = ffi::ANDROID_BITMAP_RESULT_ALLOCATION_FAILED,
    BadParameter = ffi::ANDROID_BITMAP_RESULT_BAD_PARAMETER,
    JniException = ffi::ANDROID_BITMAP_RESULT_JNI_EXCEPTION,
}

pub type BitmapResult<T, E = BitmapError> = Result<T, E>;

impl BitmapError {
    pub(crate) fn from_status(status: i32) -> BitmapResult<()> {
        Err(match status {
            ffi::ANDROID_BITMAP_RESULT_SUCCESS => return Ok(()),
            ffi::ANDROID_BITMAP_RESULT_ALLOCATION_FAILED => BitmapError::AllocationFailed,
            ffi::ANDROID_BITMAP_RESULT_BAD_PARAMETER => BitmapError::BadParameter,
            ffi::ANDROID_BITMAP_RESULT_JNI_EXCEPTION => BitmapError::JniException,
            _ => BitmapError::Unknown,
        })
    }
}

fn construct<T>(with_ptr: impl FnOnce(*mut T) -> i32) -> BitmapResult<T> {
    let mut result = MaybeUninit::uninit();
    let status = with_ptr(result.as_mut_ptr());
    BitmapError::from_status(status).map(|()| unsafe { result.assume_init() })
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[allow(non_camel_case_types)]
pub enum BitmapFormat {
    NONE = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_NONE.0,
    RGBA_8888 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGBA_8888.0,
    RGB_565 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGB_565.0,
    #[deprecated = "Deprecated in API level 13. Because of the poor quality of this configuration, it is advised to use ARGB_8888 instead."]
    RGBA_4444 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGBA_4444.0,
    A_8 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_A_8.0,
    RGBA_F16 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGBA_F16.0,
    RGBA_1010102 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGBA_1010102.0,
}

/// An immediate wrapper over [`android.graphics.Bitmap`]
///
/// [`android.graphics.Bitmap`]: https://developer.android.com/reference/android/graphics/Bitmap
#[derive(Debug)]
pub struct AndroidBitmap {
    env: *mut JNIEnv,
    inner: jobject,
}

impl AndroidBitmap {
    /// Create an [`AndroidBitmap`] wrapper from JNI pointers
    ///
    /// # Safety
    /// This function should be called with a healthy JVM pointer and with a non-null
    /// [`android.graphics.Bitmap`], which must be kept alive on the Java/Kotlin side.
    ///
    /// [`android.graphics.Bitmap`]: https://developer.android.com/reference/android/graphics/Bitmap
    pub unsafe fn from_jni(env: *mut JNIEnv, bitmap: jobject) -> Self {
        Self { env, inner: bitmap }
    }

    pub fn get_info(&self) -> BitmapResult<AndroidBitmapInfo> {
        let inner =
            construct(|res| unsafe { ffi::AndroidBitmap_getInfo(self.env, self.inner, res) })?;

        Ok(AndroidBitmapInfo { inner })
    }

    pub fn lock_pixels(&self) -> BitmapResult<*mut std::os::raw::c_void> {
        construct(|res| unsafe { ffi::AndroidBitmap_lockPixels(self.env, self.inner, res) })
    }

    pub fn unlock_pixels(&self) -> BitmapResult<()> {
        let status = unsafe { ffi::AndroidBitmap_unlockPixels(self.env, self.inner) };
        BitmapError::from_status(status)
    }

    /// Retrieve the native object associated with a `HARDWARE` [`AndroidBitmap`].
    ///
    /// Client must not modify it while an [`AndroidBitmap`] is wrapping it.
    #[cfg(feature = "api-level-30")]
    pub fn get_hardware_buffer(&self) -> BitmapResult<HardwareBufferRef> {
        unsafe {
            let result =
                construct(|res| ffi::AndroidBitmap_getHardwareBuffer(self.env, self.inner, res))?;
            let non_null = if cfg!(debug_assertions) {
                std::ptr::NonNull::new(result).expect("result should never be null")
            } else {
                std::ptr::NonNull::new_unchecked(result)
            };
            Ok(HardwareBufferRef::from_ptr(non_null))
        }
    }
}

/// A native [`AndroidBitmapInfo`]
///
/// [`AndroidBitmapInfo`]: https://developer.android.com/ndk/reference/struct/android-bitmap-info#struct_android_bitmap_info
#[derive(Copy, Clone, Debug)]
pub struct AndroidBitmapInfo {
    inner: ffi::AndroidBitmapInfo,
}

// TODO: flesh out when API 30 is released
#[cfg(feature = "api-level-30")]
pub type AndroidBitmapInfoFlags = u32;

impl AndroidBitmapInfo {
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    pub fn height(&self) -> u32 {
        self.inner.height
    }

    pub fn stride(&self) -> u32 {
        self.inner.stride
    }

    /// Convert the internal, native [`ffi::AndroidBitmapInfo::format`] into a [`BitmapFormat`].
    ///
    /// # Panics
    ///
    /// This function panics if the underlying value does not have a corresponding variant in
    /// [`BitmapFormat`]. Use [`try_format()`][AndroidBitmapInfo::try_format()] for an infallible
    /// version of this function.
    pub fn format(&self) -> BitmapFormat {
        self.try_format().unwrap()
    }

    /// Attempt to convert the internal, native [`ffi::AndroidBitmapInfo::format`] into a
    /// [`BitmapFormat`]. This may fail if the value does not have a corresponding Rust enum
    /// variant.
    pub fn try_format(&self) -> Result<BitmapFormat, TryFromPrimitiveError<BitmapFormat>> {
        let format = self.inner.format as u32;
        format.try_into()
    }

    #[cfg(feature = "api-level-30")]
    pub fn flags(&self) -> AndroidBitmapInfoFlags {
        self.inner.flags
    }
}
