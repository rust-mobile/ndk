//! Bindings for NDK media status codes.
//!
//! Also used outside of `libmediandk.so` in `libamidi.so` for example.
#![cfg(feature = "media")]
// The cfg(feature) bounds for some pub(crate) fn uses are non-trivial and will become even more
// complex going forward.  Allow them to be unused when compiling with certain feature combinations.
#![allow(dead_code)]

use std::{fmt, mem::MaybeUninit, ptr::NonNull};

use num_enum::{FromPrimitive, IntoPrimitive};

pub type Result<T, E = MediaError> = std::result::Result<T, E>;

/// Media Status codes for [`media_status_t`](https://developer.android.com/ndk/reference/group/media#group___media_1ga009a49041fe39f7bdc6d8b5cddbe760c)
#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[doc(alias = "media_status_t")]
#[non_exhaustive]
pub enum MediaError {
    #[doc(alias = "AMEDIACODEC_ERROR_INSUFFICIENT_RESOURCE")]
    CodecErrorInsufficientResource = ffi::media_status_t::AMEDIACODEC_ERROR_INSUFFICIENT_RESOURCE.0,
    #[doc(alias = "AMEDIACODEC_ERROR_RECLAIMED")]
    CodecErrorReclaimed = ffi::media_status_t::AMEDIACODEC_ERROR_RECLAIMED.0,
    #[doc(alias = "AMEDIA_ERROR_UNKNOWN")]
    ErrorUnknown = ffi::media_status_t::AMEDIA_ERROR_UNKNOWN.0,
    #[doc(alias = "AMEDIA_ERROR_MALFORMED")]
    ErrorMalformed = ffi::media_status_t::AMEDIA_ERROR_MALFORMED.0,
    #[doc(alias = "AMEDIA_ERROR_UNSUPPORTED")]
    ErrorUnsupported = ffi::media_status_t::AMEDIA_ERROR_UNSUPPORTED.0,
    #[doc(alias = "AMEDIA_ERROR_INVALID_OBJECT")]
    ErrorInvalidObject = ffi::media_status_t::AMEDIA_ERROR_INVALID_OBJECT.0,
    #[doc(alias = "AMEDIA_ERROR_INVALID_PARAMETER")]
    ErrorInvalidParameter = ffi::media_status_t::AMEDIA_ERROR_INVALID_PARAMETER.0,
    #[doc(alias = "AMEDIA_ERROR_INVALID_OPERATION")]
    ErrorInvalidOperation = ffi::media_status_t::AMEDIA_ERROR_INVALID_OPERATION.0,
    #[doc(alias = "AMEDIA_ERROR_END_OF_STREAM")]
    ErrorEndOfStream = ffi::media_status_t::AMEDIA_ERROR_END_OF_STREAM.0,
    #[doc(alias = "AMEDIA_ERROR_IO")]
    ErrorIo = ffi::media_status_t::AMEDIA_ERROR_IO.0,
    #[doc(alias = "AMEDIA_ERROR_WOULD_BLOCK")]
    ErrorWouldBlock = ffi::media_status_t::AMEDIA_ERROR_WOULD_BLOCK.0,
    #[doc(alias = "AMEDIA_DRM_ERROR_BASE")]
    DrmErrorBase = ffi::media_status_t::AMEDIA_DRM_ERROR_BASE.0,
    #[doc(alias = "AMEDIA_DRM_NOT_PROVISIONED")]
    DrmNotProvisioned = ffi::media_status_t::AMEDIA_DRM_NOT_PROVISIONED.0,
    #[doc(alias = "AMEDIA_DRM_RESOURCE_BUSY")]
    DrmResourceBusy = ffi::media_status_t::AMEDIA_DRM_RESOURCE_BUSY.0,
    #[doc(alias = "AMEDIA_DRM_DEVICE_REVOKED")]
    DrmDeviceRevoked = ffi::media_status_t::AMEDIA_DRM_DEVICE_REVOKED.0,
    #[doc(alias = "AMEDIA_DRM_SHORT_BUFFER")]
    DrmShortBuffer = ffi::media_status_t::AMEDIA_DRM_SHORT_BUFFER.0,
    #[doc(alias = "AMEDIA_DRM_SESSION_NOT_OPENED")]
    DrmSessionNotOpened = ffi::media_status_t::AMEDIA_DRM_SESSION_NOT_OPENED.0,
    #[doc(alias = "AMEDIA_DRM_TAMPER_DETECTED")]
    DrmTamperDetected = ffi::media_status_t::AMEDIA_DRM_TAMPER_DETECTED.0,
    #[doc(alias = "AMEDIA_DRM_VERIFY_FAILED")]
    DrmVerifyFailed = ffi::media_status_t::AMEDIA_DRM_VERIFY_FAILED.0,
    #[doc(alias = "AMEDIA_DRM_NEED_KEY")]
    DrmNeedKey = ffi::media_status_t::AMEDIA_DRM_NEED_KEY.0,
    #[doc(alias = "AMEDIA_DRM_LICENSE_EXPIRED")]
    DrmLicenseExpired = ffi::media_status_t::AMEDIA_DRM_LICENSE_EXPIRED.0,
    #[doc(alias = "AMEDIA_IMGREADER_ERROR_BASE")]
    ImgreaderErrorBase = ffi::media_status_t::AMEDIA_IMGREADER_ERROR_BASE.0,
    #[doc(alias = "AMEDIA_IMGREADER_NO_BUFFER_AVAILABLE")]
    ImgreaderNoBufferAvailable = ffi::media_status_t::AMEDIA_IMGREADER_NO_BUFFER_AVAILABLE.0,
    #[doc(alias = "AMEDIA_IMGREADER_MAX_IMAGES_ACQUIRED")]
    ImgreaderMaxImagesAcquired = ffi::media_status_t::AMEDIA_IMGREADER_MAX_IMAGES_ACQUIRED.0,
    #[doc(alias = "AMEDIA_IMGREADER_CANNOT_LOCK_IMAGE")]
    ImgreaderCannotLockImage = ffi::media_status_t::AMEDIA_IMGREADER_CANNOT_LOCK_IMAGE.0,
    #[doc(alias = "AMEDIA_IMGREADER_CANNOT_UNLOCK_IMAGE")]
    ImgreaderCannotUnlockImage = ffi::media_status_t::AMEDIA_IMGREADER_CANNOT_UNLOCK_IMAGE.0,
    #[doc(alias = "AMEDIA_IMGREADER_IMAGE_NOT_LOCKED")]
    ImgreaderImageNotLocked = ffi::media_status_t::AMEDIA_IMGREADER_IMAGE_NOT_LOCKED.0,
    /// This error code is unknown to the [`ndk`][crate] crate.  Please report an issue if you
    /// believe this code needs to be added to our mapping.
    // Use the OK discriminant, as no-one will be able to call `as i32` and only has access to the
    // constants via `From` provided by `IntoPrimitive` which reads the contained value.
    #[num_enum(catch_all)]
    Unknown(i32) = ffi::media_status_t::AMEDIA_OK.0,
}

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for MediaError {}

impl MediaError {
    /// Returns [`Ok`] on [`ffi::media_status_t::AMEDIA_OK`], [`Err`] otherwise (including positive
    /// values).
    ///
    /// Note that some known error codes (currently only for `AMediaCodec`) are positive.
    pub(crate) fn from_status(status: ffi::media_status_t) -> Result<()> {
        match status {
            ffi::media_status_t::AMEDIA_OK => Ok(()),
            x => Err(Self::from(x.0)),
        }
    }

    /// Returns the original value in [`Ok`] if it is not negative, [`Err`] otherwise.
    ///
    /// Note that some [`ffi::media_status_t`] codes are positive but will never be returned as
    /// [`Err`] from this function. As of writing these codes are specific to the `AMediaCodec` API
    /// and should not be handled generically.
    pub(crate) fn from_status_if_negative<T: Into<isize> + Copy>(value: T) -> Result<T> {
        let v = value.into();
        if v >= 0 {
            Ok(value)
        } else {
            Err(Self::from(
                i32::try_from(v).expect("Error code out of bounds"),
            ))
        }
    }
}

/// Calls the `with_ptr` construction function with a pointer to uninitialized stack memory,
/// expecting `with_ptr` to initialize it or otherwise return an error code.
pub(crate) fn construct<T>(with_ptr: impl FnOnce(*mut T) -> ffi::media_status_t) -> Result<T> {
    let mut result = MaybeUninit::uninit();
    let status = with_ptr(result.as_mut_ptr());
    MediaError::from_status(status).map(|()| unsafe { result.assume_init() })
}

/// Calls the `with_ptr` construction function with a pointer to a pointer, and expects `with_ptr`
/// to initialize the second pointer to a valid address.  That address is returned in the form of a
/// [`NonNull`] object.
pub(crate) fn construct_never_null<T>(
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
