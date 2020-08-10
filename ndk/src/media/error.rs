use super::Result;
use thiserror::Error;

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MediaErrorResult {
    CodecErrorInsufficientResource = ffi::media_status_t_AMEDIACODEC_ERROR_INSUFFICIENT_RESOURCE,
    CodecErrorReclaimed = ffi::media_status_t_AMEDIACODEC_ERROR_RECLAIMED,
    ErrorUnknown = ffi::media_status_t_AMEDIA_ERROR_UNKNOWN,
    ErrorMalformed = ffi::media_status_t_AMEDIA_ERROR_MALFORMED,
    ErrorUnsupported = ffi::media_status_t_AMEDIA_ERROR_UNSUPPORTED,
    ErrorInvalidObject = ffi::media_status_t_AMEDIA_ERROR_INVALID_OBJECT,
    ErrorInvalidParameter = ffi::media_status_t_AMEDIA_ERROR_INVALID_PARAMETER,
    ErrorInvalidOperation = ffi::media_status_t_AMEDIA_ERROR_INVALID_OPERATION,
    ErrorEndOfStream = ffi::media_status_t_AMEDIA_ERROR_END_OF_STREAM,
    ErrorIo = ffi::media_status_t_AMEDIA_ERROR_IO,
    ErrorWouldBlock = ffi::media_status_t_AMEDIA_ERROR_WOULD_BLOCK,
    DrmErrorBase = ffi::media_status_t_AMEDIA_DRM_ERROR_BASE,
    DrmNotProvisioned = ffi::media_status_t_AMEDIA_DRM_NOT_PROVISIONED,
    DrmResourceBusy = ffi::media_status_t_AMEDIA_DRM_RESOURCE_BUSY,
    DrmDeviceRevoked = ffi::media_status_t_AMEDIA_DRM_DEVICE_REVOKED,
    DrmShortBuffer = ffi::media_status_t_AMEDIA_DRM_SHORT_BUFFER,
    DrmSessionNotOpened = ffi::media_status_t_AMEDIA_DRM_SESSION_NOT_OPENED,
    DrmTamperDetected = ffi::media_status_t_AMEDIA_DRM_TAMPER_DETECTED,
    DrmVerifyFailed = ffi::media_status_t_AMEDIA_DRM_VERIFY_FAILED,
    DrmNeedKey = ffi::media_status_t_AMEDIA_DRM_NEED_KEY,
    DrmLicenseExpired = ffi::media_status_t_AMEDIA_DRM_LICENSE_EXPIRED,
    ImgreaderErrorBase = ffi::media_status_t_AMEDIA_IMGREADER_ERROR_BASE,
    ImgreaderNoBufferAvailable = ffi::media_status_t_AMEDIA_IMGREADER_NO_BUFFER_AVAILABLE,
    ImgreaderMaxImagesAcquired = ffi::media_status_t_AMEDIA_IMGREADER_MAX_IMAGES_ACQUIRED,
    ImgreaderCannotLockImage = ffi::media_status_t_AMEDIA_IMGREADER_CANNOT_LOCK_IMAGE,
    ImgreaderCannotUnlockImage = ffi::media_status_t_AMEDIA_IMGREADER_CANNOT_UNLOCK_IMAGE,
    ImgreaderImageNotLocked = ffi::media_status_t_AMEDIA_IMGREADER_IMAGE_NOT_LOCKED,
}

#[derive(Debug, Error)]
pub enum NdkMediaError {
    #[error("error Media result ({0:?})")]
    ErrorResult(MediaErrorResult),
    #[error("unknown Media error result ({0})")]
    UnknownResult(i32),
}

impl NdkMediaError {
    pub(crate) fn from_status<T>(
        status: ffi::media_status_t,
        on_success: impl FnOnce() -> T,
    ) -> Result<T> {
        use MediaErrorResult::*;
        let result = match status {
            ffi::media_status_t_AMEDIA_OK => return Ok(on_success()),
            ffi::media_status_t_AMEDIACODEC_ERROR_INSUFFICIENT_RESOURCE => {
                CodecErrorInsufficientResource
            }
            ffi::media_status_t_AMEDIACODEC_ERROR_RECLAIMED => CodecErrorReclaimed,
            ffi::media_status_t_AMEDIA_ERROR_UNKNOWN => ErrorUnknown,
            ffi::media_status_t_AMEDIA_ERROR_MALFORMED => ErrorMalformed,
            ffi::media_status_t_AMEDIA_ERROR_UNSUPPORTED => ErrorUnsupported,
            ffi::media_status_t_AMEDIA_ERROR_INVALID_OBJECT => ErrorInvalidObject,
            ffi::media_status_t_AMEDIA_ERROR_INVALID_PARAMETER => ErrorInvalidParameter,
            ffi::media_status_t_AMEDIA_ERROR_INVALID_OPERATION => ErrorInvalidOperation,
            ffi::media_status_t_AMEDIA_ERROR_END_OF_STREAM => ErrorEndOfStream,
            ffi::media_status_t_AMEDIA_ERROR_IO => ErrorIo,
            ffi::media_status_t_AMEDIA_ERROR_WOULD_BLOCK => ErrorWouldBlock,
            ffi::media_status_t_AMEDIA_DRM_ERROR_BASE => DrmErrorBase,
            ffi::media_status_t_AMEDIA_DRM_NOT_PROVISIONED => DrmNotProvisioned,
            ffi::media_status_t_AMEDIA_DRM_RESOURCE_BUSY => DrmResourceBusy,
            ffi::media_status_t_AMEDIA_DRM_DEVICE_REVOKED => DrmDeviceRevoked,
            ffi::media_status_t_AMEDIA_DRM_SHORT_BUFFER => DrmShortBuffer,
            ffi::media_status_t_AMEDIA_DRM_SESSION_NOT_OPENED => DrmSessionNotOpened,
            ffi::media_status_t_AMEDIA_DRM_TAMPER_DETECTED => DrmTamperDetected,
            ffi::media_status_t_AMEDIA_DRM_VERIFY_FAILED => DrmVerifyFailed,
            ffi::media_status_t_AMEDIA_DRM_NEED_KEY => DrmNeedKey,
            ffi::media_status_t_AMEDIA_DRM_LICENSE_EXPIRED => DrmLicenseExpired,
            ffi::media_status_t_AMEDIA_IMGREADER_ERROR_BASE => ImgreaderErrorBase,
            ffi::media_status_t_AMEDIA_IMGREADER_NO_BUFFER_AVAILABLE => ImgreaderNoBufferAvailable,
            ffi::media_status_t_AMEDIA_IMGREADER_MAX_IMAGES_ACQUIRED => ImgreaderMaxImagesAcquired,
            ffi::media_status_t_AMEDIA_IMGREADER_CANNOT_LOCK_IMAGE => ImgreaderCannotLockImage,
            ffi::media_status_t_AMEDIA_IMGREADER_CANNOT_UNLOCK_IMAGE => ImgreaderCannotUnlockImage,
            ffi::media_status_t_AMEDIA_IMGREADER_IMAGE_NOT_LOCKED => ImgreaderImageNotLocked,
            _ => return Err(NdkMediaError::UnknownResult(status)),
        };
        Err(NdkMediaError::ErrorResult(result))
    }
}
