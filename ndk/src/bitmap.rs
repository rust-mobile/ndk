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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitmapError {
    Unknown,
    #[doc(alias = "ANDROID_BITMAP_RESULT_ALLOCATION_FAILED")]
    AllocationFailed = ffi::ANDROID_BITMAP_RESULT_ALLOCATION_FAILED,
    #[doc(alias = "ANDROID_BITMAP_RESULT_BAD_PARAMETER")]
    BadParameter = ffi::ANDROID_BITMAP_RESULT_BAD_PARAMETER,
    #[doc(alias = "ANDROID_BITMAP_RESULT_JNI_EXCEPTION")]
    JniException = ffi::ANDROID_BITMAP_RESULT_JNI_EXCEPTION,
}

pub type Result<T, E = BitmapError> = std::result::Result<T, E>;

impl BitmapError {
    pub(crate) fn from_status(status: i32) -> Result<()> {
        Err(match status {
            ffi::ANDROID_BITMAP_RESULT_SUCCESS => return Ok(()),
            ffi::ANDROID_BITMAP_RESULT_ALLOCATION_FAILED => BitmapError::AllocationFailed,
            ffi::ANDROID_BITMAP_RESULT_BAD_PARAMETER => BitmapError::BadParameter,
            ffi::ANDROID_BITMAP_RESULT_JNI_EXCEPTION => BitmapError::JniException,
            _ => BitmapError::Unknown,
        })
    }
}

fn construct<T>(with_ptr: impl FnOnce(*mut T) -> i32) -> Result<T> {
    let mut result = MaybeUninit::uninit();
    let status = with_ptr(result.as_mut_ptr());
    BitmapError::from_status(status).map(|()| unsafe { result.assume_init() })
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[allow(non_camel_case_types)]
#[doc(alias = "AndroidBitmapFormat")]
pub enum BitmapFormat {
    #[doc(alias = "ANDROID_BITMAP_FORMAT_NONE")]
    NONE = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_NONE.0,
    #[doc(alias = "ANDROID_BITMAP_FORMAT_RGBA_8888")]
    RGBA_8888 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGBA_8888.0,
    #[doc(alias = "ANDROID_BITMAP_FORMAT_RGB_565")]
    RGB_565 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGB_565.0,
    #[deprecated = "Deprecated in API level 13. Because of the poor quality of this configuration, it is advised to use ARGB_8888 instead."]
    #[doc(alias = "ANDROID_BITMAP_FORMAT_RGBA_4444")]
    RGBA_4444 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGBA_4444.0,
    #[doc(alias = "ANDROID_BITMAP_FORMAT_A_8")]
    A_8 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_A_8.0,
    #[doc(alias = "ANDROID_BITMAP_FORMAT_RGBA_F16")]
    RGBA_F16 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGBA_F16.0,
    #[doc(alias = "ANDROID_BITMAP_FORMAT_RGBA_1010102")]
    RGBA_1010102 = ffi::AndroidBitmapFormat::ANDROID_BITMAP_FORMAT_RGBA_1010102.0,
}

/// An immediate wrapper over [`android.graphics.Bitmap`]
///
/// [`android.graphics.Bitmap`]: https://developer.android.com/reference/android/graphics/Bitmap
#[derive(Debug)]
pub struct Bitmap {
    env: *mut JNIEnv,
    inner: jobject,
}

impl Bitmap {
    /// Create a [`Bitmap`] wrapper from JNI pointers
    ///
    /// # Safety
    /// This function should be called with a healthy JVM pointer and with a non-null
    /// [`android.graphics.Bitmap`], which must be kept alive on the Java/Kotlin side.
    ///
    /// [`android.graphics.Bitmap`]: https://developer.android.com/reference/android/graphics/Bitmap
    pub unsafe fn from_jni(env: *mut JNIEnv, bitmap: jobject) -> Self {
        Self { env, inner: bitmap }
    }

    /// Fills out and returns the [`BitmapInfo`] struct for the given Java bitmap object.
    #[doc(alias = "AndroidBitmap_getInfo")]
    pub fn info(&self) -> Result<BitmapInfo> {
        let inner =
            construct(|res| unsafe { ffi::AndroidBitmap_getInfo(self.env, self.inner, res) })?;

        Ok(BitmapInfo { inner })
    }

    /// Attempt to lock the pixel address.
    ///
    /// Locking will ensure that the memory for the pixels will not move until the
    /// [`Bitmap::unlock_pixels()`] call, and ensure that, if the pixels had been previously purged,
    /// they will have been restored.
    ///
    /// If this call succeeds, it must be balanced by a call to [`Bitmap::unlock_pixels()`], after
    /// which time the address of the pixels should no longer be used.
    #[doc(alias = "AndroidBitmap_lockPixels")]
    pub fn lock_pixels(&self) -> Result<*mut std::os::raw::c_void> {
        construct(|res| unsafe { ffi::AndroidBitmap_lockPixels(self.env, self.inner, res) })
    }

    /// Call this to balance a successful call to [`Bitmap::lock_pixels()`].
    #[doc(alias = "AndroidBitmap_unlockPixels")]
    pub fn unlock_pixels(&self) -> Result<()> {
        let status = unsafe { ffi::AndroidBitmap_unlockPixels(self.env, self.inner) };
        BitmapError::from_status(status)
    }

    /// Retrieve the native object associated with an [`ffi::ANDROID_BITMAP_FLAGS_IS_HARDWARE`]
    /// [`Bitmap`] (requires [`BitmapInfoFlags::is_hardware()`] on [`BitmapInfo::flags()`] to return
    /// [`true`]).
    ///
    /// Client must not modify it while a [`Bitmap`] is wrapping it.
    #[cfg(feature = "api-level-30")]
    #[doc(alias = "AndroidBitmap_getHardwareBuffer")]
    pub fn hardware_buffer(&self) -> Result<HardwareBufferRef> {
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

/// Possible values for [`ffi::ANDROID_BITMAP_FLAGS_ALPHA_MASK`] within [`BitmapInfoFlags`]
#[cfg(feature = "api-level-30")]
#[derive(Clone, Copy, Debug)]
#[doc(alias = "ANDROID_BITMAP_FLAGS_ALPHA_MASK")]
pub enum BitmapInfoFlagsAlpha {
    /// Pixel components are premultiplied by alpha.
    #[doc(alias = "ANDROID_BITMAP_FLAGS_ALPHA_PREMUL")]
    Premultiplied,
    /// Pixels are opaque.
    #[doc(alias = "ANDROID_BITMAP_FLAGS_ALPHA_OPAQUE")]
    Opaque,
    /// Pixel components are independent of alpha.
    #[doc(alias = "ANDROID_BITMAP_FLAGS_ALPHA_UNPREMUL")]
    Unpremultiplied,
}

/// Bitfield containing information about the bitmap.
#[cfg(feature = "api-level-30")]
#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct BitmapInfoFlags(u32);

#[cfg(feature = "api-level-30")]
impl std::fmt::Debug for BitmapInfoFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BitmapInfoFlags({:#x}, alpha: {:?}, is_hardware: {})",
            self.0,
            self.alpha(),
            self.is_hardware()
        )
    }
}

#[cfg(feature = "api-level-30")]
impl BitmapInfoFlags {
    /// Returns the alpha value contained in the [`ffi::ANDROID_BITMAP_FLAGS_ALPHA_MASK`] bit range
    #[doc(alias = "ANDROID_BITMAP_FLAGS_ALPHA_MASK")]
    pub fn alpha(self) -> BitmapInfoFlagsAlpha {
        // Note that ffi::ANDROID_BITMAP_FLAGS_ALPHA_SHIFT is 0 and hence irrelevant.
        match self.0 & ffi::ANDROID_BITMAP_FLAGS_ALPHA_MASK {
            ffi::ANDROID_BITMAP_FLAGS_ALPHA_PREMUL => BitmapInfoFlagsAlpha::Premultiplied,
            ffi::ANDROID_BITMAP_FLAGS_ALPHA_OPAQUE => BitmapInfoFlagsAlpha::Opaque,
            ffi::ANDROID_BITMAP_FLAGS_ALPHA_UNPREMUL => BitmapInfoFlagsAlpha::Unpremultiplied,
            3 => todo!("ALPHA_MASK value 3"),
            _ => unreachable!(),
        }
    }

    /// Returns [`true`] when [`ffi::ANDROID_BITMAP_FLAGS_IS_HARDWARE`] is set, meaning this
    /// [`Bitmap`] uses "HARDWARE Config" and its [`HardwareBufferRef`] can be retrieved via
    /// [`Bitmap::hardware_buffer()`].
    #[doc(alias = "ANDROID_BITMAP_FLAGS_IS_HARDWARE")]
    pub fn is_hardware(self) -> bool {
        // This constant is defined in a separate anonymous enum which bindgen treats as i32.
        (self.0 & ffi::ANDROID_BITMAP_FLAGS_IS_HARDWARE as u32) != 0
    }
}

/// A native [`AndroidBitmapInfo`]
///
/// [`AndroidBitmapInfo`]: https://developer.android.com/ndk/reference/struct/android-bitmap-info#struct_android_bitmap_info
#[derive(Clone, Copy)]
#[doc(alias = "AndroidBitmapInfo")]
pub struct BitmapInfo {
    inner: ffi::AndroidBitmapInfo,
}

impl std::fmt::Debug for BitmapInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("BitmapInfo");
        f.field("width", &self.width())
            .field("height", &self.height())
            .field("stride", &self.stride())
            .field("format", &self.try_format());

        #[cfg(feature = "api-level-30")]
        f.field("flags", &self.flags());

        f.finish()
    }
}

impl BitmapInfo {
    /// The bitmap width in pixels.
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    /// The bitmap height in pixels.
    pub fn height(&self) -> u32 {
        self.inner.height
    }

    /// The number of byte per row.
    pub fn stride(&self) -> u32 {
        self.inner.stride
    }

    /// Convert the internal, native [`ffi::AndroidBitmapInfo::format`] into a [`BitmapFormat`].
    ///
    /// # Panics
    ///
    /// This function panics if the underlying value does not have a corresponding variant in
    /// [`BitmapFormat`]. Use [`try_format()`][BitmapInfo::try_format()] for an infallible version
    /// of this function.
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

    /// Bitfield containing information about the bitmap.
    #[cfg(feature = "api-level-30")]
    pub fn flags(&self) -> BitmapInfoFlags {
        BitmapInfoFlags(self.inner.flags)
    }
}
