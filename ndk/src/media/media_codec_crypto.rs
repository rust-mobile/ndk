//! Bindings for [`AMediaCodecCryptoInfo`]
//!
//! [`AMediaCodecCryptoInfo`]: https://developer.android.com/ndk/reference/group/media#amediacodeccryptoinfo

use std::{mem::MaybeUninit, ptr::NonNull};

use crate::media_error::{MediaError, Result};

/// A native [`AMediaCodecCryptoInfo *`]
///
/// [`AMediaCodecCryptoInfo *`]: https://developer.android.com/ndk/reference/group/media#amediacodeccryptoinfo
#[derive(Debug)]
#[doc(alias = "AMediaCodecCryptoInfo")]
pub struct MediaCodecCryptoInfo {
    inner: NonNull<ffi::AMediaCodecCryptoInfo>,
}

impl MediaCodecCryptoInfo {
    /// Create a [`MediaCodecCryptoInfo`] from scratch. Use this if you need to use custom crypto
    /// info, rather than one obtained from [`super::media_extractor::MediaExtractor`].
    ///
    /// [`MediaCodecCryptoInfo`] describes the structure of an (at least partially) encrypted input
    /// sample.
    ///
    /// A buffer's data is considered to be partitioned into "subsamples", each subsample starts
    /// with a (potentially empty) run of plain, unencrypted bytes followed by a (also potentially
    /// empty) run of encrypted bytes.
    ///
    /// `clearbytes` can be null to indicate that all data is encrypted. This information
    /// encapsulates per-sample metadata as outlined in ISO/IEC FDIS 23001-7:2011 "Common encryption
    /// in ISO base media file format files".
    #[doc(alias = "AMediaCodecCryptoInfo_new")]
    pub fn new(
        num_sub_samples: i32,
        key: &[u8; 16],
        iv: &[u8; 16],
        mode: ffi::cryptoinfo_mode_t,
    ) -> Option<Self> {
        let mut clear_bytes = 0;
        let mut encrypted_bytes = 0;
        NonNull::new(unsafe {
            ffi::AMediaCodecCryptoInfo_new(
                num_sub_samples,
                key.as_ptr().cast_mut(),
                iv.as_ptr().cast_mut(),
                mode,
                // TODO: Set the value and/or store it?
                &mut clear_bytes,
                &mut encrypted_bytes,
            )
        })
        .map(|inner| Self { inner })
    }

    /// Assumes ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::AMediaCodecCryptoInfo`].
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AMediaCodecCryptoInfo>) -> Self {
        Self { inner: ptr }
    }

    pub fn as_ptr(&self) -> *mut ffi::AMediaCodecCryptoInfo {
        self.inner.as_ptr()
    }

    /// Set the crypto pattern on an AMediaCryptoInfo object.
    #[doc(alias = "AMediaCodecCryptoInfo_setPattern")]
    pub fn set_pattern(&self, pattern: &ffi::cryptoinfo_pattern_t) {
        unsafe {
            ffi::AMediaCodecCryptoInfo_setPattern(
                self.inner.as_ptr(),
                <*const _>::cast_mut(pattern),
            )
        }
    }

    /// The number of subsamples that make up the buffer's contents.
    #[doc(alias = "AMediaCodecCryptoInfo_getNumSubSamples")]
    pub fn num_sub_samples(&self) -> usize {
        unsafe { ffi::AMediaCodecCryptoInfo_getNumSubSamples(self.inner.as_ptr()) }
    }

    /// A 16-byte opaque key.
    #[doc(alias = "AMediaCodecCryptoInfo_getKey")]
    pub fn key(&self) -> Result<[u8; 16]> {
        let mut key = [0u8; 16];
        let status =
            unsafe { ffi::AMediaCodecCryptoInfo_getKey(self.inner.as_ptr(), key.as_mut_ptr()) };
        MediaError::from_status(status).map(|()| key)
    }

    /// A 16-byte initialization vector.
    #[doc(alias = "AMediaCodecCryptoInfo_getIV")]
    pub fn iv(&self) -> Result<[u8; 16]> {
        let mut iv = [0u8; 16];
        let status =
            unsafe { ffi::AMediaCodecCryptoInfo_getIV(self.inner.as_ptr(), iv.as_mut_ptr()) };
        MediaError::from_status(status).map(|()| iv)
    }

    /// The type of encryption that has been applied,
    /// one of AMEDIACODECRYPTOINFO_MODE_CLEAR or AMEDIACODECRYPTOINFO_MODE_AES_CTR.
    #[doc(alias = "AMediaCodecCryptoInfo_getMode")]
    pub fn mode(&self) -> ffi::cryptoinfo_mode_t {
        unsafe { ffi::AMediaCodecCryptoInfo_getMode(self.inner.as_ptr()) }
    }

    /// The number of leading unencrypted bytes in each subsample.
    #[doc(alias = "AMediaCodecCryptoInfo_getClearBytes")]
    pub fn clear_bytes(&self) -> Result<usize> {
        let mut clear_bytes = MaybeUninit::uninit();

        let status = unsafe {
            ffi::AMediaCodecCryptoInfo_getClearBytes(self.inner.as_ptr(), clear_bytes.as_mut_ptr())
        };
        MediaError::from_status(status).map(|()| unsafe { clear_bytes.assume_init() })
    }

    /// The number of trailing encrypted bytes in each subsample.
    #[doc(alias = "AMediaCodecCryptoInfo_getEncryptedBytes")]
    pub fn encrypted_bytes(&self) -> Result<usize> {
        let mut encrypted_bytes = MaybeUninit::uninit();
        let status = unsafe {
            ffi::AMediaCodecCryptoInfo_getEncryptedBytes(
                self.inner.as_ptr(),
                encrypted_bytes.as_mut_ptr(),
            )
        };
        MediaError::from_status(status).map(|()| unsafe { encrypted_bytes.assume_init() })
    }
}

impl Drop for MediaCodecCryptoInfo {
    /// Delete a [`MediaCodecCryptoInfo`] created previously with [`MediaCodecCryptoInfo::new()`], or
    /// obtained from [`super::media_extractor::MediaExtractor`].
    #[doc(alias = "AMediaCodecCryptoInfo_delete")]
    fn drop(&mut self) {
        let status = unsafe { ffi::AMediaCodecCryptoInfo_delete(self.inner.as_ptr()) };
        MediaError::from_status(status).unwrap()
    }
}
