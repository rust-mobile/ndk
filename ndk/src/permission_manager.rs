//! Bindings for permission checks
//!
//! <https://developer.android.com/ndk/reference/group/permission>
#![cfg(feature = "api-level-31")]

use std::{ffi::CStr, mem::MaybeUninit};

use num_enum::{FromPrimitive, IntoPrimitive};

/// Permission check return status values.
#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[non_exhaustive]
pub enum PermissionManagerStatus {
    /// This is returned if the permission check encountered an unspecified error.
    ErrorUnknown = ffi::PERMISSION_MANAGER_STATUS_ERROR_UNKNOWN,
    /// This is returned if the permission check failed because the service is unavailable.
    ServiceUnavailable = ffi::PERMISSION_MANAGER_STATUS_SERVICE_UNAVAILABLE,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(i32) = 0,
}

/// Checks whether the package with the given pid/uid has been granted a permission.
///
/// Note that the Java API of [`Context#checkPermission()`] is usually faster due to caching,
/// thus is preferred over this API wherever possible.
///
/// [`Context#checkPermission()`]: https://developer.android.com/reference/android/content/Context#checkPermission(java.lang.String,%20int,%20int)
///
/// # Parameters
/// - `permission`: the permission to be checked.
/// - `pid`: the process id of the package to be checked.
/// - `uid`: the uid of the package to be checked.
#[doc(alias = "APermissionManager_checkPermission")]
pub fn check_permission(
    permission: &CStr,
    pid: i32,
    uid: u32,
) -> Result<bool, PermissionManagerStatus> {
    let mut result = MaybeUninit::uninit();
    match unsafe {
        ffi::APermissionManager_checkPermission(permission.as_ptr(), pid, uid, result.as_mut_ptr())
    } {
        ffi::PERMISSION_MANAGER_STATUS_OK => Ok(match unsafe { result.assume_init() } {
            ffi::PERMISSION_MANAGER_PERMISSION_GRANTED => true,
            ffi::PERMISSION_MANAGER_PERMISSION_DENIED => false,
            x => unreachable!("Unexpected `PERMISSION` result output {x}"),
        }),
        x => Err(x.into()),
    }
}
