//! Bindings for [System Properties]
//!
//! [System Properties]: https://source.android.com/docs/core/architecture/configuration/add-system-properties

use std::{
    ffi::{c_char, c_void, CStr, CString, FromBytesWithNulError, FromVecWithNulError},
    fmt,
    io::Error,
    ptr::NonNull,
    str::{FromStr, Utf8Error},
};
#[cfg(feature = "api-level-26")]
use std::{mem::MaybeUninit, num::NonZeroU32, time::Duration};

use thiserror::Error;

use crate::utils::{abort_on_panic, status_to_io_result};

/// Possible failures returned by [`get_raw()`].
#[derive(Debug, Error)]
pub enum GetRawError {
    #[error("Property is missing or empty")]
    MissingOrEmpty,
    #[error(transparent)]
    NulError(#[from] FromVecWithNulError),
    #[error(transparent)]
    Io(#[from] Error),
}

/// Internal helper to deduplicate the implementation between [`get_raw()`] and
/// [`Property::read_raw()`].
fn process_owned(get: impl FnOnce(*mut c_char) -> i32) -> Result<CString, GetRawError> {
    // Pre-allocate a `Vec` which we can move to the user with the result
    let mut value = Vec::with_capacity(ffi::PROP_VALUE_MAX as usize);
    let ret = get(value.as_mut_ptr());
    match ret {
        0 => Err(GetRawError::MissingOrEmpty),
        ..=-1 => Err(Error::from_raw_os_error(-ret).into()),
        1.. => {
            // TODO: This "smart" implementation leaves the user with a 92-byte allocation since
            // set_len() currently doesn't shrink.  Any such operation would likely reallocate,
            // making this have no advantage over the stack-local variant that allocates after the
            // fact.
            unsafe { value.set_len(ret as usize + 1) }
            Ok(CString::from_vec_with_nul(value)?)
        }
    }
}

/// Returns the property value as an owned [`CString`] with possibly invalid UTF-8 [but no interor
/// NULs].  The maximum length can be up to 92 ([`ffi::PROP_VALUE_MAX`]) including NUL terminator.
///
/// [but no interor NULs]: GetRawError::NulError
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
#[allow(missing_debug_implementations)] // Our MSRV is too low for derive(Debug) to emit bounds on T.
#[derive(Error)]
pub enum GetError<T> {
    #[error("Property is missing or empty")]
    MissingOrEmpty,
    #[error(transparent)]
    NulError(#[from] FromBytesWithNulError),
    #[error("Property does not contain valid UTF-8")]
    Utf8Error(#[from] Utf8Error),
    #[error(transparent)]
    Io(#[from] Error),
    #[error(transparent)]
    ParseError(T),
}

/// Internal helper to deduplicate the implementation between [`get()`] and [`Property::read()`].
fn process_parse<T: FromStr>(get: impl FnOnce(*mut c_char) -> i32) -> Result<T, GetError<T::Err>> {
    let mut value = [0u8; ffi::PROP_VALUE_MAX as usize];
    let ret = get(value.as_mut_ptr());
    match ret {
        0 => Err(GetError::MissingOrEmpty),
        ..=-1 => Err(Error::from_raw_os_error(-ret).into()),
        1.. => {
            let c_str = CStr::from_bytes_with_nul(&value[..ret as usize + 1])?;
            c_str.to_str()?.parse().map_err(GetError::ParseError)
        }
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
/// [no interior NULs]: GetError::NulError
/// [valid UTF-8]: GetError::Utf8Error
///
/// # Deprecation
/// Deprecated since Android O (API level 26), use [`Property::find()`] with
/// [`Property::read_callback()`] instead which does not have a limit on `value` nor `name` length.
#[doc(alias = "__system_property_get")]
pub fn get<T: FromStr>(name: &CStr) -> Result<T, GetError<T::Err>> {
    process_parse(|value| unsafe { ffi::__system_property_get(name.as_ptr(), value) })
}

/// Sets system property `name` to `value`, creating the system property if it doesn't already
/// exist.
#[doc(alias = "__system_property_set")]
pub fn set(name: &CStr, value: &CStr) -> std::io::Result<()> {
    let ret = unsafe { ffi::__system_property_set(name.as_ptr(), value.as_ptr()) };
    match ret {
        0 => Ok(()),
        ..=-1 => Err(Error::from_raw_os_error(-ret)),
        1.. => panic!("Unexpected non-zero non-negative return value `{ret}`"),
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

impl Property {
    /// Returns a [`Property`] corresponding to the system property `name`, or [`None`] if it
    /// doesn't exist.  Use [`Property::read()`] or [`Property::read_callback()`] to query the
    /// current value.
    ///
    /// Property lookup is expensive, so it can be useful to cache the result of this function.
    #[doc(alias = "__system_property_find")]
    pub fn find(name: &CStr) -> Option<Self> {
        let prop = unsafe { ffi::__system_property_find(name.as_ptr()) };
        // TODO: No lifetime information available for this pointer
        NonNull::new(prop.cast_mut()).map(Self)
    }

    /// Calls the `callback` for every system property with a [`Property`] handle. Use in
    /// conjunction with [`Property::read_callback()`] to get its name and value.
    ///
    /// This method is for inspecting and debugging the property system, and not generally useful.
    #[doc(alias = "__system_property_foreach")]
    pub fn foreach<F: FnMut(Self)>(mut callback: F) -> std::io::Result<()> {
        unsafe extern "C" fn ffi_callback<F: FnMut(Property)>(
            pi: *const ffi::prop_info,
            cookie: *mut c_void,
        ) {
            abort_on_panic(|| {
                let callback = cookie as *mut F;

                // TODO: No lifetime information available for this pointer
                (*callback)(Property(NonNull::new(pi.cast_mut()).unwrap()))
            })
        }

        let ret = unsafe {
            ffi::__system_property_foreach(Some(ffi_callback::<F>), <*mut _>::cast(&mut callback))
        };

        status_to_io_result(ret)
    }

    /// Returns an owned [`CString`] with possibly invalid UTF-8 [but no interor NULs].  The maximum
    /// length can be up to 92 ([`ffi::PROP_VALUE_MAX`]) including NUL terminator.
    ///
    /// [but no interor NULs]: GetRawError::NulError
    ///
    /// See [`Property::read()`] for a more convenient API that validates this string for UTF-8 and
    /// directly parses it into a [`FromStr`]-compatible type.
    ///
    /// # Deprecation
    /// Deprecated since Android O (API level 26), use [`Self::read_callback()`] instead which does
    /// not have a limit on `value` nor `name` length.
    #[doc(alias = "__system_property_read")]
    pub fn read_raw(&self) -> Result<CString, GetRawError> {
        process_owned(|value| unsafe {
            // TODO: should we return the name of ffi::PROP_NAME_MAX?
            ffi::__system_property_read(self.0.as_ptr(), std::ptr::null_mut(), value)
        })
    }

    /// Returns the property value as a [`FromStr`]-parsed type from a source string of at most 92
    /// ([`ffi::PROP_VALUE_MAX`]) characters, including NUL terminator.
    ///
    /// # Implementation details
    /// This is implemented without any up-front allocations like [`get_raw()`], but requires a trip
    /// through [`CStr`] and [`str`] (for calling [`FromStr::from_str()`]) meaning the resulting
    /// string has to be compliant with [`CStr`] ([no interior NULs]) and [`str`] ([valid UTF-8]).
    /// In other words, parsing into a [`String`] will never contain interior NULs (and it is
    /// unknown and unlikely whether the property API allows for this).
    ///
    /// [no interior NULs]: GetError::NulError
    /// [valid UTF-8]: GetError::Utf8Error
    ///
    /// # Deprecation
    /// Deprecated since Android O (API level 26), use [`Self::read_callback()`] instead which does
    /// not have a limit on `value` nor `name` length.
    #[doc(alias = "__system_property_read")]
    pub fn read<T: FromStr>(&self) -> Result<T, GetError<T::Err>> {
        process_parse(|value| unsafe {
            // TODO: should we return the name of ffi::PROP_NAME_MAX?
            ffi::__system_property_read(self.0.as_ptr(), std::ptr::null_mut(), value)
        })
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
