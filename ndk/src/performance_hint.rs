//! Bindings for [`APerformanceHintManager`] and [`APerformanceHintSession`]
//!
//! `APerformanceHint` allows apps to create performance hint sessions for groups of
//! threads, and provide hints to the system about the workload of those threads, to help the
//! system more accurately allocate power for them. It is the NDK counterpart to the [Java
//! PerformanceHintManager SDK API].
//!
//! [`APerformanceHintManager`]: https://developer.android.com/ndk/reference/group/a-performance-hint#aperformancehintmanager
//! [`APerformanceHintSession`]: https://developer.android.com/ndk/reference/group/a-performance-hint#aperformancehintsession
//! [Java PerformanceHintManager SDK API]: https://developer.android.com/reference/android/os/PerformanceHintManager
#![cfg(feature = "api-level-33")]

#[cfg(doc)]
use std::io::ErrorKind;
use std::{io::Result, ptr::NonNull, time::Duration};

use crate::utils::status_to_io_result;

/// An opaque type representing a handle to a performance hint manager.
///
/// To use:
/// - Obtain the performance hint manager instance by calling [`PerformanceHintManager::new()`].
/// - Create a [`PerformanceHintSession`] with [`PerformanceHintManager::new()`].
/// - Get the preferred update rate with [`PerformanceHintManager::preferred_update_rate()`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[doc(alias = "APerformanceHintManager")]
pub struct PerformanceHintManager {
    ptr: NonNull<ffi::APerformanceHintManager>,
}

// SAFETY: The NDK stores a per-process global singleton that is accessible from any thread, and the
// only public methods perform thread-safe operations (i.e. the underlying AIBinder API to implement
// `IHintManager->createHintSession()` is thread-safe).
unsafe impl Send for PerformanceHintManager {}
unsafe impl Sync for PerformanceHintManager {}

impl PerformanceHintManager {
    /// Retrieve a reference to the performance hint manager.
    ///
    /// Returns [`None`] on failure.
    #[doc(alias = "APerformanceHint_getManager")]
    pub fn new() -> Option<Self> {
        NonNull::new(unsafe { ffi::APerformanceHint_getManager() }).map(|ptr| Self { ptr })
    }

    /// Creates a session for the given set of threads and sets their initial target work duration.
    ///
    /// # Parameters
    /// - `thread_ids`: The list of threads to be associated with this session. They must be part of
    ///   this process' thread group.
    /// - `initial_target_work_duration`: The target duration for the new session. This must be
    ///   positive if using the work duration API, or [`Duration::ZERO`] otherwise.
    #[doc(alias = "APerformanceHint_createSession")]
    pub fn create_session(
        &self,
        thread_ids: &[i32],
        initial_target_work_duration: Duration,
    ) -> Option<PerformanceHintSession> {
        NonNull::new(unsafe {
            ffi::APerformanceHint_createSession(
                self.ptr.as_ptr(),
                thread_ids.as_ptr(),
                thread_ids.len(),
                initial_target_work_duration
                    .as_nanos()
                    .try_into()
                    .expect("Duration should be convertible to i64 nanoseconds"),
            )
        })
        .map(|ptr| PerformanceHintSession { ptr })
    }

    /// Get preferred update rate information for this device.
    ///
    /// Returns the preferred update rate supported by device software.
    #[doc(alias = "APerformanceHint_getPreferredUpdateRateNanos")]
    pub fn preferred_update_rate(&self) -> Duration {
        Duration::from_nanos(unsafe {
            ffi::APerformanceHint_getPreferredUpdateRateNanos(self.ptr.as_ptr())
                .try_into()
                .expect("getPreferredUpdateRateNanos should not return negative")
        })
    }
}

/// An opaque type representing a handle to a performance hint session.  A session can only be
/// acquired from a [`PerformanceHintManager`] with [`PerformanceHintManager::create_session()`].
/// It will be freed automatically on [`drop()`] after use.
///
/// A Session represents a group of threads with an inter-related workload such that hints for their
/// performance should be considered as a unit.  The threads in a given session should be long-lived
/// and not created or destroyed dynamically.
///
/// The work duration API can be used with periodic workloads to dynamically adjust
/// thread performance and keep the work on schedule while optimizing the available
/// power budget.  When using the work duration API, the starting target duration
/// should be specified while creating the session, and can later be adjusted with
/// [`PerformanceHintSession::update_target_work_duration()`].  While using the work duration API,
/// the client is expected to call [`PerformanceHintSession::report_actual_work_duration()`] each
/// cycle to report the actual time taken to complete to the system.
///
/// All timings should be from [`ffi::CLOCK_MONOTONIC`].
#[derive(Debug, PartialEq, Eq, Hash)]
#[doc(alias = "APerformanceHintSession")]
pub struct PerformanceHintSession {
    ptr: NonNull<ffi::APerformanceHintSession>,
}

impl PerformanceHintSession {
    /// Updates this session's target duration for each cycle of work.
    ///
    /// `target_duration` is the new desired duration.
    ///
    /// # Returns
    /// - [`ErrorKind::InvalidInput`] if `target_duration` is not positive.
    /// - [`ErrorKind::BrokenPipe`] if communication with the system service has failed.
    #[doc(alias = "APerformanceHint_updateTargetWorkDuration")]
    pub fn update_target_work_duration(&self, target_duration: Duration) -> Result<()> {
        status_to_io_result(unsafe {
            ffi::APerformanceHint_updateTargetWorkDuration(
                self.ptr.as_ptr(),
                target_duration
                    .as_nanos()
                    .try_into()
                    .expect("Duration should be convertible to i64 nanoseconds"),
            )
        })
    }

    /// Reports the actual duration for the last cycle of work.
    ///
    /// The system will attempt to adjust the scheduling and performance of the threads within the
    /// thread group to bring the actual duration close to the target duration.
    ///
    /// `actual_duration` is the duration of time the thread group took to complete its last
    ///     task.
    ///
    /// # Returns
    /// - [`ErrorKind::InvalidInput`] if `actual_duration` is not positive.
    /// - [`ErrorKind::BrokenPipe`] if communication with the system service has failed.
    #[doc(alias = "APerformanceHint_reportActualWorkDuration")]
    pub fn report_actual_work_duration(&self, actual_duration: Duration) -> Result<()> {
        status_to_io_result(unsafe {
            ffi::APerformanceHint_reportActualWorkDuration(
                self.ptr.as_ptr(),
                actual_duration
                    .as_nanos()
                    .try_into()
                    .expect("Duration should be convertible to i64 nanoseconds"),
            )
        })
    }

    /// Set a list of threads to the performance hint session.  This operation will replace the
    /// current list of threads with the given list of threads.
    ///
    /// `thread_ids` is the list of threads to be associated with this session. They must be part of
    /// this app's thread group.
    ///
    /// # Returns
    /// - [`ErrorKind::InvalidInput`] if the list of thread ids is empty or if any of the thread ids
    ///   are not part of the thread group.
    /// - [`ErrorKind::BrokenPipe`] if communication with the system service has failed.
    /// - [`ErrorKind::PermissionDenied`] if any thread id doesn't belong to the application.
    #[cfg(feature = "api-level-34")]
    #[doc(alias = "APerformanceHint_setThreads")]
    pub fn set_threads(&self, thread_ids: &[i32]) -> Result<()> {
        status_to_io_result(unsafe {
            ffi::APerformanceHint_setThreads(
                self.ptr.as_ptr(),
                thread_ids.as_ptr(),
                thread_ids.len(),
            )
        })
    }

    /// This tells the session that these threads can be safely scheduled to prefer power efficiency
    /// over performance.
    ///
    /// `enabled` is the flag which sets whether this session will use power-efficient scheduling.
    ///
    /// # Returns
    /// - [`ErrorKind::BrokenPipe`] if communication with the system service has failed.
    #[cfg(feature = "api-level-35")]
    #[doc(alias = "APerformanceHint_setPreferPowerEfficiency")]
    pub fn set_prefer_power_efficiency(&self, enabled: bool) -> Result<()> {
        status_to_io_result(unsafe {
            ffi::APerformanceHint_setPreferPowerEfficiency(self.ptr.as_ptr(), enabled)
        })
    }

    /// Reports the durations for the last cycle of work.
    ///
    /// The system will attempt to adjust the scheduling and performance of the threads within the
    /// thread group to bring the actual duration close to the target duration.
    ///
    /// # Parameters
    /// - `work_duration` is the [`WorkDuration`] structure of times the thread group took to
    ///   complete its last task breaking down into different components.
    ///
    ///   The work period start timestamp and actual total duration must be greater than zero.
    ///
    ///   The actual CPU and GPU durations must be greater than or equal to [`Duration::ZERO`], and
    ///   at least one of them must be greater than [`Duration::ZERO`]. When one of them is equal to
    ///   [`Duration::ZERO`], it means that type of work was not measured for this workload.
    ///
    /// # Returns
    /// - [`ErrorKind::BrokenPipe`] if communication with the system service has failed.
    #[cfg(feature = "api-level-35")]
    #[doc(alias = "APerformanceHint_reportActualWorkDuration2")]
    pub fn report_actual_work_duration2(&self, work_duration: &WorkDuration) -> Result<()> {
        status_to_io_result(unsafe {
            ffi::APerformanceHint_reportActualWorkDuration2(
                self.ptr.as_ptr(),
                work_duration.ptr.as_ptr(),
            )
        })
    }
}

impl Drop for PerformanceHintSession {
    /// Release the performance hint manager pointer acquired via
    /// [`PerformanceHintManager::create_session()`].
    #[doc(alias = "APerformanceHint_closeSession")]
    fn drop(&mut self) {
        unsafe { ffi::APerformanceHint_closeSession(self.ptr.as_ptr()) }
    }
}

#[cfg(feature = "api-level-35")]
#[derive(Debug, PartialEq, Eq, Hash)]
#[doc(alias = "AWorkDuration")]
pub struct WorkDuration {
    ptr: NonNull<ffi::AWorkDuration>,
}

#[cfg(feature = "api-level-35")]
impl WorkDuration {
    /// Creates a new [`WorkDuration`]. When the client finishes using [`WorkDuration`], it will
    /// automatically be released on [`drop()`].
    #[doc(alias = "AWorkDuration_create")]
    pub fn new() -> Self {
        Self {
            ptr: NonNull::new(unsafe { ffi::AWorkDuration_create() })
                .expect("AWorkDuration_create should not return NULL"),
        }
    }

    /// Sets the work period start timestamp in nanoseconds.
    ///
    /// `work_period_start_timestamp` is the work period start timestamp based on
    /// [`ffi::CLOCK_MONOTONIC`] about when the work starts.  This timestamp must be greater than
    /// [`Duration::ZERO`].
    #[doc(alias = "AWorkDuration_setWorkPeriodStartTimestampNanos")]
    pub fn set_work_period_start_timestamp(&self, work_period_start_timestamp: Duration) {
        unsafe {
            ffi::AWorkDuration_setWorkPeriodStartTimestampNanos(
                self.ptr.as_ptr(),
                work_period_start_timestamp
                    .as_nanos()
                    .try_into()
                    .expect("Duration should be convertible to i64 nanoseconds"),
            )
        }
    }

    /// Sets the actual total work duration in nanoseconds.
    ///
    /// `actual_total_duration` is the actual total work duration.  This number must be greater
    /// than [`Duration::ZERO`].
    #[doc(alias = "AWorkDuration_setActualTotalDurationNanos")]
    pub fn set_actual_total_duration(&self, actual_total_duration: Duration) {
        unsafe {
            ffi::AWorkDuration_setActualTotalDurationNanos(
                self.ptr.as_ptr(),
                actual_total_duration
                    .as_nanos()
                    .try_into()
                    .expect("Duration should be convertible to i64 nanoseconds"),
            )
        }
    }

    /// Sets the actual CPU work duration in nanoseconds.
    ///
    /// `actual_cpu_duration` is the actual CPU work duration.  If it is equal to
    /// [`Duration::ZERO`], that means the CPU was not measured.
    #[doc(alias = "AWorkDuration_setActualCpuDurationNanos")]
    pub fn set_actual_cpu_duration(&self, actual_cpu_duration: Duration) {
        unsafe {
            ffi::AWorkDuration_setActualCpuDurationNanos(
                self.ptr.as_ptr(),
                actual_cpu_duration
                    .as_nanos()
                    .try_into()
                    .expect("Duration should be convertible to i64 nanoseconds"),
            )
        }
    }

    /// Sets the actual GPU work duration in nanoseconds.
    ///
    /// `actual_gpu_duration` is the actual GPU work duration.  If it is equal to
    /// [`Duration::ZERO`], that means the GPU was not measured.
    #[doc(alias = "AWorkDuration_setActualGpuDurationNanos")]
    pub fn set_actual_gpu_duration(&self, actual_gpu_duration: Duration) {
        unsafe {
            ffi::AWorkDuration_setActualGpuDurationNanos(
                self.ptr.as_ptr(),
                actual_gpu_duration
                    .as_nanos()
                    .try_into()
                    .expect("Duration should be convertible to i64 nanoseconds"),
            )
        }
    }
}

impl Default for WorkDuration {
    #[doc(alias = "AWorkDuration_create")]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WorkDuration {
    /// Destroys [`WorkDuration`] and free all resources associated to it.
    #[doc(alias = "AWorkDuration_release")]
    fn drop(&mut self) {
        unsafe { ffi::AWorkDuration_release(self.ptr.as_ptr()) }
    }
}
