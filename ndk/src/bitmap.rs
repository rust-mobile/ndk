#![cfg(feature = "bitmap")]

use jni_sys::{jobject, JNIEnv};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{convert::TryInto, mem::MaybeUninit, ptr::NonNull};

#[cfg(feature = "hardware_buffer")]
use crate::hardware_buffer::HardwareBuffer;

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BitmapError {
    Unknown,
    AllocationFailed = ffi::ANDROID_BITMAP_RESULT_ALLOCATION_FAILED,
    BadParameter = ffi::ANDROID_BITMAP_RESULT_BAD_PARAMETER,
    JniException = ffi::ANDROID_BITMAP_RESULT_JNI_EXCEPTION,
}

pub type BitmapResult<T, E = BitmapError> = std::result::Result<T, E>;

impl BitmapError {
    pub(crate) fn from_status<T>(status: i32, on_success: impl FnOnce() -> T) -> BitmapResult<T> {
        Err(match status {
            ffi::ANDROID_BITMAP_RESULT_SUCCESS => return Ok(on_success()),
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
    BitmapError::from_status(status, || unsafe { result.assume_init() })
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[allow(non_camel_case_types)]
pub enum BitmapFormat {
    NONE = ffi::AndroidBitmapFormat_ANDROID_BITMAP_FORMAT_NONE,
    RGBA_8888 = ffi::AndroidBitmapFormat_ANDROID_BITMAP_FORMAT_RGBA_8888,
    RGB_565 = ffi::AndroidBitmapFormat_ANDROID_BITMAP_FORMAT_RGB_565,
    #[deprecated = "Deprecated in API level 13. Because of the poor quality of this configuration, it is advised to use ARGB_8888 instead."]
    RGBA_4444 = ffi::AndroidBitmapFormat_ANDROID_BITMAP_FORMAT_RGBA_4444,
    A_8 = ffi::AndroidBitmapFormat_ANDROID_BITMAP_FORMAT_A_8,
    RGBA_F16 = ffi::AndroidBitmapFormat_ANDROID_BITMAP_FORMAT_RGBA_F16,
}

#[derive(Debug)]
pub struct AndroidBitmap {
    env: *mut ffi::JNIEnv,
    inner: ffi::jobject,
}

impl AndroidBitmap {
    /// Create an `AndroidBitmap` from JNI pointers
    ///
    /// # Safety
    /// By calling this function, you assert that it these are valid pointers to JNI objects.
    pub unsafe fn from_jni(env: *mut JNIEnv, bitmap: jobject) -> Self {
        Self {
            env: env as _,
            inner: bitmap as _,
        }
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
        BitmapError::from_status(status, || ())
    }

    #[cfg(all(feature = "hardware_buffer", feature = "api-level-30"))]
    pub fn get_hardware_buffer(&self) -> BitmapResult<HardwareBuffer> {
        unsafe {
            let result =
                construct(|res| ffi::AndroidBitmap_getHardwareBuffer(self.env, self.inner, res))?;
            let non_null = if cfg!(debug_assertions) {
                NonNull::new(result).expect("result should never be null")
            } else {
                NonNull::new_unchecked(result)
            };
            Ok(HardwareBuffer::from_ptr(non_null))
        }
    }
}

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

    pub fn format(&self) -> BitmapFormat {
        let format = self.inner.format as u32;
        format.try_into().unwrap()
    }

    #[cfg(feature = "api-level-30")]
    pub fn flags(&self) -> AndroidBitmapInfoFlags {
        self.inner.flags
    }
}
