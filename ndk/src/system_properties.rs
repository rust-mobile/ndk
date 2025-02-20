//! Bindings for [System Properties]
//!
//! [System Properties]: https://source.android.com/docs/core/architecture/configuration/add-system-properties

use std::{
    ffi::{c_char, c_void, CStr, CString, FromBytesWithNulError, FromVecWithNulError},
    fmt,
    mem::MaybeUninit,
    ptr::NonNull,
    str::{FromStr, Utf8Error},
};
#[cfg(feature = "api-level-26")]
use std::{num::NonZeroU32, time::Duration};

use thiserror::Error;

use crate::utils::abort_on_panic;

/// Possible failures returned by [`get_raw()`].
#[derive(Debug, Error)]
pub enum GetRawError {
    #[error("Property is missing or empty")]
    MissingOrEmpty,
    #[error("Property value does not include a terminating NUL")]
    ValueMissingNul(#[source] FromVecWithNulError),
    #[error("`__system_property_get()` or `__system_property_read()` failed")]
    Failed,
}

/// Internal helper to deduplicate the implementation between [`get_raw()`] and
/// [`Property::read_raw()`].
fn process_owned(get: impl FnOnce(*mut c_char) -> i32) -> Result<CString, GetRawError> {
    // Pre-allocate a `Vec` which we can move to the user with the result
    let mut value = Vec::<u8>::with_capacity(ffi::PROP_VALUE_MAX as usize);
    let ret = get(value.as_mut_ptr().cast());
    match ret {
        0 => Err(GetRawError::MissingOrEmpty),
        -1 => Err(GetRawError::Failed),
        1.. => {
            // TODO: This "smart" implementation leaves the user with a 92-byte allocation since
            // set_len() currently doesn't shrink.  Any such operation would likely reallocate,
            // making this have no advantage over the stack-local variant that allocates after the
            // fact.
            unsafe { value.set_len(ret as usize + 1) }
            Ok(CString::from_vec_with_nul(value).map_err(GetRawError::ValueMissingNul)?)
        }
        _ => unreachable!("Status is unexpected integer {ret}"),
    }
}

/// Returns the property value as an owned [`CString`] with possibly invalid UTF-8 [but no interior
/// NULs].  The maximum length can be up to 92 ([`ffi::PROP_VALUE_MAX`]) including NUL terminator.
///
/// [but no interior NULs]: GetRawError::ValueMissingNul
///
/// See [`get()`] for a more convenient API that validates this string for UTF-8 and directly parses
/// it into a [`FromStr`]-compatible type.
///
/// # Deprecation
/// Deprecated since Android O (API level 26), use [`Property::find()`] with
/// [`Property::read_callback()`] instead which does not have a limit on `value` nor `name` length.
#[doc(alias = "__system_property_get")]
pub fn get_raw(name: &CStr) -> Result<CString, GetRawError> {
    process_owned(|value| unsafe { ffi::__system_property_get(name.as_ptr(), value) })
}

/// Possible failures returned by [`get()`].
#[derive(Debug, Error)]
pub enum GetError<T> {
    #[error("Property is missing or empty")]
    MissingOrEmpty,
    #[error("Property value does not include a terminating NUL")]
    ValueMissingNul(#[source] FromBytesWithNulError),
    #[error("Property value does not contain valid UTF-8")]
    Utf8Error(#[from] Utf8Error),
    #[error("`__system_property_get()` or `__system_property_read()` failed")]
    Failed,
    #[error(transparent)]
    ParseError(T),
}

/// Internal helper to deduplicate the implementation between [`get()`] and [`Property::read()`].
fn process_parse<T: FromStr>(get: impl FnOnce(*mut c_char) -> i32) -> Result<T, GetError<T::Err>> {
    let mut value = [0u8; ffi::PROP_VALUE_MAX as usize];
    let ret = get(value.as_mut_ptr().cast());
    match ret {
        0 => Err(GetError::MissingOrEmpty),
        -1 => Err(GetError::Failed),
        1.. => {
            let c_str = CStr::from_bytes_with_nul(&value[..ret as usize + 1])
                .map_err(GetError::ValueMissingNul)?;
            c_str.to_str()?.parse().map_err(GetError::ParseError)
        }
        _ => unreachable!("Status is unexpected integer {ret}"),
    }
}

/// Returns the property value as a [`FromStr`]-parsed type from a source string of at most 92
/// ([`ffi::PROP_VALUE_MAX`]) characters, including NUL terminator.
///
/// # Implementation details
/// This is implemented without any up-front allocations like [`get_raw()`], but requires a trip
/// through [`CStr`] and [`str`] (for calling [`FromStr::from_str()`]) meaning the resulting
/// string has to be compliant with [`CStr`] ([no interior NULs]) and [`str`] ([valid UTF-8]).  In
/// other words, parsing into a [`String`] will never contain interior NULs (and it is unknown and
/// unlikely whether the property API allows for this).
///
/// [no interior NULs]: GetError::ValueMissingNul
/// [valid UTF-8]: GetError::Utf8Error
///
/// # Deprecation
/// Deprecated since Android O (API level 26), use [`Property::find()`] with
/// [`Property::read_callback()`] instead which does not have a limit on `value` nor `name` length.
#[doc(alias = "__system_property_get")]
pub fn get<T: FromStr>(name: &CStr) -> Result<T, GetError<T::Err>> {
    process_parse(|value| unsafe { ffi::__system_property_get(name.as_ptr(), value) })
}

/// Wraps a [`std::io::Error::last_os_error()`] that may not have been updated by a call to
/// [`set()`] and instead contain a stale or bogus value.
#[derive(Error)]
#[error("`__system_property_set()` failed.  The source `io::Error` may not originate from this function")]
pub struct SetError(#[source] std::io::Error);

impl fmt::Debug for SetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`__system_property_set()` failed.  The source `io::Error` may not originate from this function")
    }
}

/// Sets system property `name` to `value`, creating the system property if it doesn't already
/// exist.
///
/// # Stale `errno` result
/// The underlying function, [`ffi::__system_property_set()`], is notorious for [inconsistently
/// setting `errno`] when returning `-1`.  The [`std::io::Error::last_os_error()`] value returned
/// inside [`SetError`] may not have been updated by this call but come from a stale `errno` value.
///
/// [inconsistently setting `errno`]: https://cs.android.com/android/platform/superproject/main/+/main:bionic/libc/bionic/system_property_set.cpp;l=283-286;drc=620eec1f3546523fab4f58fa6733e7faa34e4fd2
#[allow(clippy::result_unit_err)]
#[doc(alias = "__system_property_set")]
pub fn set(name: &CStr, value: &CStr) -> Result<(), SetError> {
    let ret = unsafe { ffi::__system_property_set(name.as_ptr(), value.as_ptr()) };
    match ret {
        0 => Ok(()),
        -1 => Err(SetError(std::io::Error::last_os_error())),
        _ => unreachable!("Status is unexpected integer {ret}"),
    }
}

/// Modern abstraction to [find], cache, [read] and [wait] on properties.
///
/// [find]: Property::find()
/// [read]: Property::read()
/// [wait]: Property::wait()
#[doc(alias = "prop_info")]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Property(NonNull<ffi::prop_info>);

/// The name, value and serial of a property during [`Property::read_callback()`].
#[cfg(feature = "api-level-26")]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PropertyValue<'a> {
    pub name: &'a CStr,
    pub value: &'a CStr,
    pub serial: u32,
}

impl fmt::Debug for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "api-level-26")]
        {
            let mut result = None;
            self.read_callback(|pv: &PropertyValue<'_>| result = Some(pv.fmt(f)));
            result.expect("Read callback never called")
        }
        #[cfg(not(feature = "api-level-26"))]
        {
            f.debug_struct("Property")
                .field("value", &self.read_raw())
                .finish_non_exhaustive()
        }
    }
}

#[derive(Debug, Error)]
#[error("Property::foreach() failed")]
pub struct ForeachError;

impl Property {
    /// Returns a [`Property`] corresponding to the system property `name`, or [`None`] if it
    /// doesn't exist.  Use [`Property::read()`] or [`Property::read_callback()`] to query the
    /// current value.
    ///
    /// Property lookup is expensive, so it can be useful to cache the result of this function.
    #[doc(alias = "__system_property_find")]
    pub fn find(name: &CStr) -> Option<Self> {
        let prop = unsafe { ffi::__system_property_find(name.as_ptr()) };
        NonNull::new(prop.cast_mut()).map(Self)
    }

    /// Calls the `callback` for every system property with a [`Property`] handle. Use in
    /// conjunction with [`Property::read_callback()`] to get its name and value.
    ///
    /// This method is for inspecting and debugging the property system, and not generally useful.
    #[doc(alias = "__system_property_foreach")]
    pub fn foreach<F: FnMut(Self)>(mut callback: F) -> Result<(), ForeachError> {
        unsafe extern "C" fn ffi_callback<F: FnMut(Property)>(
            pi: *const ffi::prop_info,
            cookie: *mut c_void,
        ) {
            abort_on_panic(|| {
                let callback = cookie.cast::<F>();
                (*callback)(Property(NonNull::new(pi.cast_mut()).unwrap()))
            })
        }

        let ret = unsafe {
            ffi::__system_property_foreach(Some(ffi_callback::<F>), <*mut _>::cast(&mut callback))
        };

        match ret {
            0 => Ok(()),
            // Only seems to fail if the property service failed to initialize:
            // https://cs.android.com/android/platform/superproject/main/+/main:bionic/libc/system_properties/system_properties.cpp;drc=622b6aeeefb11efcd338338586564368bc47750a
            -1 => Err(ForeachError),
            _ => unreachable!("Status is unexpected integer {ret}"),
        }
    }

    /// Returns a tuple of the property `(name, value)` as owned [`CString`]s with possibly
    /// invalid UTF-8 [but no interior NULs].  The maximum length can be up to 32 bytes
    /// ([`ffi::PROP_NAME_MAX`]) for the name and 92 bytes ([`ffi::PROP_VALUE_MAX`]) for the value
    /// including NUL terminator.
    ///
    /// [but no interior NULs]: GetRawError::ValueMissingNul
    ///
    /// See [`Property::read()`] for a more convenient API that validates this string for UTF-8 and
    /// directly parses it into a [`FromStr`]-compatible type.
    ///
    /// # Deprecation
    /// Deprecated since Android O (API level 26), use [`Self::read_callback()`] instead which does
    /// not have a limit on `value` nor `name` length.
    #[doc(alias = "__system_property_read")]
    pub fn read_raw(&self) -> Result<(CString, CString), GetRawError> {
        let mut name = [MaybeUninit::<c_char>::uninit(); ffi::PROP_NAME_MAX as usize];
        let value = process_owned(|value| unsafe {
            ffi::__system_property_read(self.0.as_ptr(), name.as_mut_ptr().cast(), value)
        })?;
        // TODO: Switch to CStr::from_bytes_until_nul (with c_char -> u8 transmute) since MSRV 1.69
        // SAFETY: __system_property_read() has initialized all bytes until and including a NUL
        // terminator, which it is guaranteed to write within the length limit.
        let name = unsafe { CStr::from_ptr(name.as_ptr().cast()) }.to_owned();
        Ok((name, value))
    }

    /// Returns the property name as owned [`CString`] of at most 32 bytes ([`ffi::PROP_NAME_MAX`]),
    /// and its value as a [`FromStr`]-parsed type from a source string of at most 92 bytes
    /// ([`ffi::PROP_VALUE_MAX`]), both including NUL terminator.
    ///
    /// # Implementation details
    /// This is implemented without any up-front allocations like [`get_raw()`], but requires a trip
    /// through [`CStr`] and [`str`] (for calling [`FromStr::from_str()`]) meaning the resulting
    /// string has to be compliant with [`CStr`] ([no interior NULs]) and [`str`] ([valid UTF-8]).
    /// In other words, parsing into a [`String`] will never contain interior NULs (and it is
    /// unknown and unlikely whether the property API allows for this).
    ///
    /// [no interior NULs]: GetError::ValueMissingNul
    /// [valid UTF-8]: GetError::Utf8Error
    ///
    /// # Deprecation
    /// Deprecated since Android O (API level 26), use [`Self::read_callback()`] instead which does
    /// not have a limit on `value` nor `name` length.
    #[doc(alias = "__system_property_read")]
    pub fn read<T: FromStr>(&self) -> Result<(CString, T), GetError<T::Err>> {
        let mut name = [MaybeUninit::<c_char>::uninit(); ffi::PROP_NAME_MAX as usize];
        let value = process_parse(|value| unsafe {
            ffi::__system_property_read(self.0.as_ptr(), name.as_mut_ptr().cast(), value)
        })?;
        // TODO: Switch to CStr::from_bytes_until_nul (with c_char -> u8 transmute) since MSRV 1.69
        // SAFETY: __system_property_read() has initialized all bytes until and including a NUL
        // terminator, which it is guaranteed to write within the length limit.
        let name = unsafe { CStr::from_ptr(name.as_ptr().cast()) }.to_owned();
        Ok((name, value))
    }

    /// Calls `callback` with a consistent trio of `name`, `value` and `serial` number (stored in
    /// [`PropertyValue`]) for this [`Property`].
    #[cfg(feature = "api-level-26")]
    #[doc(alias = "__system_property_read_callback")]
    pub fn read_callback<F: FnOnce(&PropertyValue<'_>)>(&self, callback: F) {
        // Wrap the callback in a MaybeUninit so that ffi_callback() can "copy from" a pointer to it
        // and consume the FnOnce, leaving the original callback "invalid" but inaccessible without
        // unsafe.
        let mut callback = MaybeUninit::new(callback);
        unsafe extern "C" fn ffi_callback<F: FnOnce(&PropertyValue<'_>)>(
            cookie: *mut c_void,
            name: *const c_char,
            value: *const c_char,
            serial: u32,
        ) {
            abort_on_panic(|| {
                let callback: F = std::ptr::read(cookie.cast());
                let name = CStr::from_ptr(name);
                let value = CStr::from_ptr(value);

                callback(&PropertyValue {
                    name,
                    value,
                    serial,
                })
            })
        }

        unsafe {
            ffi::__system_property_read_callback(
                self.0.as_ptr(),
                Some(ffi_callback::<F>),
                callback.as_mut_ptr().cast(),
            )
        }
    }

    /// Waits for this specific system property to be updated past `old_serial`. Waits no longer
    /// than `timeout`, or forever if `timeout` is [`None`].
    ///
    /// Returns the new serial in [`Some`], or [`None`] on timeout.
    #[cfg(feature = "api-level-26")]
    #[doc(alias = "__system_property_wait")]
    pub fn wait(&self, old_serial: Option<NonZeroU32>, timeout: Option<Duration>) -> Option<u32> {
        wait_optional_prop(Some(self), old_serial, timeout)
    }
}

/// Internal helper to deduplicate the implementation between [`wait()`] and [`Property::wait()`].
#[cfg(feature = "api-level-26")]
fn wait_optional_prop(
    prop: Option<&Property>,
    old_serial: Option<NonZeroU32>,
    timeout: Option<Duration>,
) -> Option<u32> {
    let mut new_serial = MaybeUninit::uninit();
    let timeout = timeout.map_or(std::ptr::null(), |t| &ffi::timespec {
        tv_sec: t.as_secs() as i64,
        tv_nsec: t.subsec_nanos() as i64,
    });
    unsafe {
        // XXX: The C wrapper API converts a possible -1 error to a boolean (true):
        // https://cs.android.com/android/platform/superproject/main/+/main:bionic/libc/system_properties/system_properties.cpp;l=428;drc=622b6aeeefb11efcd338338586564368bc47750a
        // https://cs.android.com/android/platform/superproject/main/+/main:bionic/libc/bionic/system_property_api.cpp;l=118-120;drc=620eec1f3546523fab4f58fa6733e7faa34e4fd2
        ffi::__system_property_wait(
            prop.map_or(std::ptr::null(), |p| p.0.as_ptr()),
            old_serial.map_or(0, NonZeroU32::get),
            new_serial.as_mut_ptr(),
            timeout,
        )
    }
    .then(|| unsafe { new_serial.assume_init() })
}

/// Waits for this specific system property to be updated past `old_serial`. Waits no longer than
/// `timeout`, or forever if `timeout` is [`None`].
///
/// Same as [`Property::wait()`], but for the global serial number.
///
/// Returns the new serial in [`Some`], or [`None`] on timeout.
#[cfg(feature = "api-level-26")]
#[doc(alias = "__system_property_wait")]
pub fn wait(old_serial: Option<NonZeroU32>, timeout: Option<Duration>) -> Option<u32> {
    wait_optional_prop(None, old_serial, timeout)
}
