//! Bindings for [`AThermalManager`]
//!
//! Structures and functions to access thermal status and register/unregister thermal status
//! listener in native code.
//!
//! [`AThermalManager`]: https://developer.android.com/ndk/reference/group/thermal#athermalmanager
#![cfg(feature = "api-level-30")]

#[cfg(doc)]
use std::io::ErrorKind;
use std::{io::Result, os::raw::c_void, ptr::NonNull};

use num_enum::{FromPrimitive, IntoPrimitive};

use crate::utils::abort_on_panic;

/// Workaround for <https://issuetracker.google.com/issues/358664965>.  `status_t` should only
/// contain negative error codes, but the underlying `AThermal` implementation freely passes
/// positive error codes around. At least the expected return codes are "implicitly" documented to
/// be positive.
fn status_to_io_result(status: i32) -> Result<()> {
    // Intentionally not imported in scope (and an identically-named function) to prevent
    // accidentally calling this function without negation.
    crate::utils::status_to_io_result(-status)
}

/// Thermal status used in function [`ThermalManager::current_thermal_status()`] and
/// [`ThermalStatusCallback`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(i32)]
#[doc(alias = "AThermalStatus")]
#[non_exhaustive]
pub enum ThermalStatus {
    /// Error in thermal status.
    // TODO: Move to a Result?
    #[doc(alias = "ATHERMAL_STATUS_ERROR")]
    Error = ffi::AThermalStatus::ATHERMAL_STATUS_ERROR.0,
    /// Not under throttling.
    #[doc(alias = "ATHERMAL_STATUS_NONE")]
    None = ffi::AThermalStatus::ATHERMAL_STATUS_NONE.0,
    /// Light throttling where UX is not impacted.
    #[doc(alias = "ATHERMAL_STATUS_LIGHT")]
    Light = ffi::AThermalStatus::ATHERMAL_STATUS_LIGHT.0,
    /// Moderate throttling where UX is not largely impacted.
    #[doc(alias = "ATHERMAL_STATUS_MODERATE")]
    Moderate = ffi::AThermalStatus::ATHERMAL_STATUS_MODERATE.0,
    /// Severe throttling where UX is largely impacted.
    #[doc(alias = "ATHERMAL_STATUS_SEVERE")]
    Severe = ffi::AThermalStatus::ATHERMAL_STATUS_SEVERE.0,
    /// Platform has done everything to reduce power.
    #[doc(alias = "ATHERMAL_STATUS_CRITICAL")]
    Critical = ffi::AThermalStatus::ATHERMAL_STATUS_CRITICAL.0,
    /// Key components in platform are shutting down due to thermal condition. Device
    /// functionalities will be limited.
    #[doc(alias = "ATHERMAL_STATUS_EMERGENCY")]
    Emergency = ffi::AThermalStatus::ATHERMAL_STATUS_EMERGENCY.0,
    /// Need shutdown immediately.
    #[doc(alias = "ATHERMAL_STATUS_SHUTDOWN")]
    Shutdown = ffi::AThermalStatus::ATHERMAL_STATUS_SHUTDOWN.0,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(i32),
}

impl From<ffi::AThermalStatus> for ThermalStatus {
    fn from(value: ffi::AThermalStatus) -> Self {
        value.0.into()
    }
}

/// Prototype of the function that is called when thermal status changes. It's passed the updated
/// thermal status as parameter.
///
/// # Warning
/// [`ThermalManager`] is synchronized internally, and its lock is held while this callback is
/// called.  Interacting with [`ThermalManager`] inside this closure *will* result in a deadlock.
#[doc(alias = "AThermal_StatusCallback")]
pub type ThermalStatusCallback = Box<dyn FnMut(ThermalStatus) + Send>;

/// Token returned by [`ThermalManager::register_thermal_status_listener()`] for a given
/// [`ThermalStatusCallback`].
///
/// Pass this to [`ThermalManager::unregister_thermal_status_listener()`] when you no longer wish to
/// receive the callback.
#[derive(Debug, PartialEq, Eq, Hash)]
#[must_use = "Without this token the callback can no longer be unregistered and will leak Boxes"]
pub struct ThermalStatusListenerToken {
    func: ffi::AThermal_StatusCallback,
    data: *mut ThermalStatusCallback,
}

// SAFETY: (un)register_thermal_status_listener() is internally synchronized
unsafe impl Send for ThermalStatusListenerToken {}
unsafe impl Sync for ThermalStatusListenerToken {}

/// An opaque type representing a handle to a thermal manager. An instance of thermal manager must
/// be acquired prior to using thermal status APIs.  It will be freed automatically on [`drop()`]
/// after use.
///
/// To use:
/// - Create a new thermal manager instance by calling the [`ThermalManager::new()`] function.
/// - Get current thermal status with [`ThermalManager::current_thermal_status()`].
/// - Register a thermal status listener with [`ThermalManager::register_thermal_status_listener()`].
/// - Unregister a thermal status listener with
///   [`ThermalManager::unregister_thermal_status_listener()`].
/// - Release the thermal manager instance with [`drop()`].
#[derive(Debug, PartialEq, Eq, Hash)]
#[doc(alias = "AThermalManager")]
pub struct ThermalManager {
    ptr: NonNull<ffi::AThermalManager>,
}

// SAFETY: All AThermalManager methods are internally synchronized
unsafe impl Send for ThermalManager {}
unsafe impl Sync for ThermalManager {}

impl ThermalManager {
    /// Acquire an instance of the thermal manager.
    ///
    /// Returns [`None`] on failure.
    #[doc(alias = "AThermal_acquireManager")]
    pub fn new() -> Option<Self> {
        NonNull::new(unsafe { ffi::AThermal_acquireManager() }).map(|ptr| Self { ptr })
    }

    /// Gets the current thermal status.
    ///
    /// Returns current thermal status, [`ThermalStatus::Error`] on failure.
    // TODO: Result?
    #[doc(alias = "AThermal_getCurrentThermalStatus")]
    pub fn current_thermal_status(&self) -> ThermalStatus {
        unsafe { ffi::AThermal_getCurrentThermalStatus(self.ptr.as_ptr()) }.into()
    }

    /// Register the thermal status listener for thermal status change.
    ///
    /// Will leak [`Box`]es unless [`ThermalManager::unregister_thermal_status_listener()`] is
    /// called.
    // TODO: This API properly mutex-syncs the callbacks with the destructor!  Meaning we can track
    // `Box`es in `self` and trivially `drop()` them _after_ calling `AThermal_releaseManager()`!
    ///
    /// # Returns
    /// - [`ErrorKind::InvalidInput`] if the listener and data pointer were previously added and not removed.
    /// - [`ErrorKind::PermissionDenied`] if the required permission is not held.
    /// - [`ErrorKind::BrokenPipe`] if communication with the system service has failed.
    #[doc(alias = "AThermal_registerThermalStatusListener")]
    pub fn register_thermal_status_listener(
        &self,
        callback: ThermalStatusCallback,
    ) -> Result<ThermalStatusListenerToken> {
        let boxed = Box::new(callback);
        // This box is only freed when unregister() is called
        let data = Box::into_raw(boxed);

        unsafe extern "C" fn thermal_status_callback(
            data: *mut c_void,
            status: ffi::AThermalStatus,
        ) {
            abort_on_panic(|| {
                let func: *mut ThermalStatusCallback = data.cast();
                (*func)(status.into())
            })
        }

        status_to_io_result(unsafe {
            ffi::AThermal_registerThermalStatusListener(
                self.ptr.as_ptr(),
                Some(thermal_status_callback),
                data.cast(),
            )
        })
        .map(|()| ThermalStatusListenerToken {
            func: Some(thermal_status_callback),
            data,
        })
    }

    /// Unregister the thermal status listener previously resgistered.
    ///
    /// # Returns
    /// - [`ErrorKind::InvalidInput`] if the listener and data pointer were not previously added.
    /// - [`ErrorKind::PermissionDenied`] if the required permission is not held.
    /// - [`ErrorKind::BrokenPipe`] if communication with the system service has failed.
    #[doc(alias = "AThermal_unregisterThermalStatusListener")]
    pub fn unregister_thermal_status_listener(
        &self,
        token: ThermalStatusListenerToken,
    ) -> Result<()> {
        status_to_io_result(unsafe {
            ffi::AThermal_unregisterThermalStatusListener(
                self.ptr.as_ptr(),
                token.func,
                token.data.cast(),
            )
        })?;
        let _ = unsafe { Box::from_raw(token.data) };
        Ok(())
    }

    /// Provides an estimate of how much thermal headroom the device currently has before hitting
    /// severe throttling.
    ///
    /// Note that this only attempts to track the headroom of slow-moving sensors, such as the
    /// skin temperature sensor.  This means that there is no benefit to calling this function more
    /// frequently than about once per second, and attempted to call significantly more frequently
    /// may result in the function returning [`f32::NAN`].
    ///
    /// In addition, in order to be able to provide an accurate forecast, the system does not
    /// attempt to forecast until it has multiple temperature samples from which to extrapolate.
    /// This should only take a few seconds from the time of the first call, but during this time,
    /// no forecasting will occur, and the current headroom will be returned regardless of the value
    /// of `forecast_seconds`.
    ///
    /// The value returned is a non-negative float that represents how much of the thermal envelope
    /// is in use (or is forecasted to be in use).  A value of `1.0` indicates that the device is
    /// (or will be) throttled at [`ThermalStatus::Severe`].  Such throttling can affect the CPU,
    /// GPU, and other subsystems.  Values may exceed `1.0`, but there is no implied mapping to
    /// specific thermal levels beyond that point.  This means that values greater than `1.0` may
    /// correspond to [`ThermalStatus::Severe`], but may also represent heavier throttling.
    ///
    /// A value of `0.0` corresponds to a fixed distance from `1.0`, but does not correspond to any
    /// particular thermal status or temperature.  Values on `(0.0, 1.0]` may be expected to scale
    /// linearly with temperature, though temperature changes over time are typically not linear.
    /// Negative values will be clamped to `0.0` before returning.
    ///
    /// `forecast_seconds` specifies how many seconds into the future to forecast. Given that device
    /// conditions may change at any time, forecasts from further in the
    /// future will likely be less accurate than forecasts in the near future.
    ////
    /// # Returns
    /// A value greater than equal to `0.0`, where `1.0` indicates the SEVERE throttling threshold,
    /// as described above. Returns [`f32::NAN`] if the device does not support this functionality
    /// or if this function is called significantly faster than once per second.
    #[cfg(feature = "api-level-31")]
    #[doc(alias = "AThermal_getThermalHeadroom")]
    pub fn thermal_headroom(
        &self,
        // TODO: Duration, even though it has a granularity of seconds?
        forecast_seconds: i32,
    ) -> f32 {
        unsafe { ffi::AThermal_getThermalHeadroom(self.ptr.as_ptr(), forecast_seconds) }
    }

    /// Gets the thermal headroom thresholds for all available thermal status.
    ///
    /// A thermal status will only exist in output if the device manufacturer has the corresponding
    /// threshold defined for at least one of its slow-moving skin temperature sensors.  If it's
    /// set, one should also expect to get it from [`ThermalManager::current_thermal_status()`] or
    /// [`ThermalStatusCallback`].
    ///
    /// The headroom threshold is used to interpret the possible thermal throttling status
    /// based on the headroom prediction.  For example, if the headroom threshold for
    /// [`ThermalStatus::Light`] is `0.7`, and a headroom prediction in `10s` returns `0.75` (or
    /// [`ThermalManager::thermal_headroom(10)`] = `0.75`), one can expect that in `10` seconds the
    /// system could be in lightly throttled state if the workload remains the same.  The app can
    /// consider taking actions according to the nearest throttling status the difference between
    /// the headroom and the threshold.
    ///
    /// For new devices it's guaranteed to have a single sensor, but for older devices with
    /// multiple sensors reporting different threshold values, the minimum threshold is taken to
    /// be conservative on predictions.  Thus, when reading real-time headroom, it's not guaranteed
    /// that a real-time value of `0.75` (or [`ThermalManager::thermal_headroom(0)`] = `0.75`)
    /// exceeding the threshold of `0.7` above will always come with lightly throttled state (or
    /// [`ThermalManager::current_thermal_status()`] = [`ThermalStatus::Light`]) but it can be lower
    /// (or [`ThermalManager::current_thermal_status()`] = [`ThermalStatus::None`]). While it's
    /// always guaranteed that the device won't be throttled heavier than the unmet threshold's
    /// state, so a real-time headroom of `0.75` will never come with [`ThermalStatus::Moderate`]
    /// but always lower, and `0.65` will never come with [`ThermalStatus::Light`] but
    /// [`ThermalStatus::None`].
    ///
    /// The returned list of thresholds is cached on first successful query and owned by the thermal
    /// manager, which will not change between calls to this function. The caller should only need
    /// to free the manager with [`drop()`].
    ///
    /// # Returns
    /// - [`ErrorKind::InvalidInput`] if outThresholds or size_t is nullptr, or *outThresholds is not nullptr.
    /// - [`ErrorKind::BrokenPipe`] if communication with the system service has failed.
    /// - [`ErrorKind::Unsupported`] if the feature is disabled by the current system.
    #[cfg(feature = "api-level-35")]
    #[doc(alias = "AThermal_getThermalHeadroomThresholds")]
    pub fn thermal_headroom_thresholds(
        &self,
    ) -> Result<Option<impl ExactSizeIterator<Item = ThermalHeadroomThreshold> + '_>> {
        let mut out_thresholds = std::ptr::null();
        let mut out_size = 0;
        status_to_io_result(unsafe {
            ffi::AThermal_getThermalHeadroomThresholds(
                self.ptr.as_ptr(),
                &mut out_thresholds,
                &mut out_size,
            )
        })?;
        if out_thresholds.is_null() {
            return Ok(None);
        }
        Ok(Some(
            unsafe { std::slice::from_raw_parts(out_thresholds, out_size) }
                .iter()
                .map(|t| ThermalHeadroomThreshold {
                    headroom: t.headroom,
                    thermal_status: t.thermalStatus.into(),
                }),
        ))
    }
}

impl Drop for ThermalManager {
    /// Release the thermal manager pointer acquired via [`ThermalManager::new()`].
    #[doc(alias = "AThermal_releaseManager")]
    fn drop(&mut self) {
        unsafe { ffi::AThermal_releaseManager(self.ptr.as_ptr()) }
    }
}

/// This struct defines an instance of headroom threshold value and its status.
///
/// The value should be monotonically non-decreasing as the thermal status increases.  For
/// [`ThermalStatus::Severe`], its headroom threshold is guaranteed to be `1.0`.  For status below
/// severe status, the value should be lower or equal to `1.0`, and for status above severe, the
/// value should be larger or equal to `1.0`.
///
/// Also see [`ThermalManager::thermal_headroom()`] for explanation on headroom, and
/// [`ThermalManager::thermal_headroom_thresholds()`] for how to use this.
#[cfg(feature = "api-level-35")]
#[derive(Clone, Copy, Debug, PartialEq)]
#[doc(alias = "AThermalHeadroomThreshold")]
pub struct ThermalHeadroomThreshold {
    headroom: f32,
    thermal_status: ThermalStatus,
}
