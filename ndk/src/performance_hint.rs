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
#[cfg(feature = "api-level-36")]
use std::{ffi::CStr, io::Error};
use std::{io::Result, ptr::NonNull, time::Duration};

#[cfg(feature = "api-level-36")]
use num_enum::IntoPrimitive;

use crate::utils::status_to_io_result;

/// An opaque type representing a handle to a performance hint manager.
///
/// To use:
/// 1. Obtain the performance hint manager instance by calling [`PerformanceHintManager::new()`].
/// 2. Create a [`PerformanceHintSession`] with [`PerformanceHintManager::new()`].
/// 3. Get the preferred update rate with [`PerformanceHintManager::preferred_update_rate()`].
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
                    .expect("Supplied duration is too large"),
            )
        })
        .map(|ptr| PerformanceHintSession { ptr })
    }

    /// Get preferred update rate information for this device.
    ///
    /// Returns the preferred update rate supported by device software.
    #[cfg_attr(
        feature = "api-level-36",
        deprecated = "Client-side rate limiting is not necessary, use `is_feature_supported()` for support checking"
    )]
    #[doc(alias = "APerformanceHint_getPreferredUpdateRateNanos")]
    pub fn preferred_update_rate(&self) -> Duration {
        Duration::from_nanos(unsafe {
            ffi::APerformanceHint_getPreferredUpdateRateNanos(self.ptr.as_ptr())
                .try_into()
                .expect("getPreferredUpdateRateNanos should not return negative")
        })
    }

    /// Creates a session using arguments from a corresponding [`SessionCreationConfig`].
    ///
    /// # Returns
    /// - [`ErrorKind::InvalidInput`] if the creation config is in an invalid state.
    /// - [`ErrorKind::BrokenPipe`] if communication failed.
    /// - [`ErrorKind::Unsupported`] if hint sessions are not supported, or if auto timing is
    ///   enabled but unsupported.
    /// - [`ErrorKind::ResourceBusy`] if too many graphics pipeline threads are passed. The NDK
    ///   still creates a session in this case, but with undefined behavior for all graphics
    ///   pipeline sessions, hence it is closed again.
    #[cfg(feature = "api-level-36")]
    #[doc(alias = "APerformanceHint_createSessionUsingConfig")]
    pub fn create_session_using_config(
        &self,
        config: &SessionCreationConfig,
    ) -> Result<PerformanceHintSession> {
        let mut session = std::ptr::null_mut();
        let status = unsafe {
            ffi::APerformanceHint_createSessionUsingConfig(
                self.ptr.as_ptr(),
                config.ptr.as_ptr(),
                &mut session,
            )
        };
        let session = NonNull::new(session).map(|ptr| PerformanceHintSession { ptr });
        errno_to_io_result(status)?;
        Ok(session.expect("createSessionUsingConfig should not return NULL on success"))
    }

    /// Get the maximum number of graphics pipeline threads per app for this device.
    ///
    /// See [`SessionCreationConfig::set_graphics_pipeline()`].
    #[cfg(feature = "api-level-36")]
    #[doc(alias = "APerformanceHint_getMaxGraphicsPipelineThreadsCount")]
    pub fn max_graphics_pipeline_threads_count(&self) -> i32 {
        unsafe { ffi::APerformanceHint_getMaxGraphicsPipelineThreadsCount(self.ptr.as_ptr()) }
    }

    /// Checks whether the device exposes support for a specific [`PerformanceHintFeature`].
    #[cfg(feature = "api-level-36")]
    #[doc(alias = "APerformanceHint_isFeatureSupported")]
    pub fn is_feature_supported(feature: PerformanceHintFeature) -> bool {
        unsafe { ffi::APerformanceHint_isFeatureSupported(feature.into()) }
    }
}

/// Different aspects of performance hint functionality, which can be passed to
/// [`PerformanceHintManager::is_feature_supported()`] to determine whether the device exposes
/// support for that feature.
///
/// Some of these features will not expose failure to the client if used when unsupported, to
/// prevent the client from needing to worry about handling different logic for each possible
/// support configuration. The exception to this is features with important user-facing side
/// effects, such as [`PerformanceHintFeature::AutoCpu`] and [`PerformanceHintFeature::AutoGpu`]
/// modes which expect the client not to report durations while they are active.
#[cfg(feature = "api-level-36")]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive)]
#[non_exhaustive]
#[doc(alias = "APerformanceHintFeature")]
pub enum PerformanceHintFeature {
    /// All [`PerformanceHintSession`] functionality. Using the Performance Hint API at all if this
    /// is not enabled will likely result in either [`PerformanceHintManager`] or
    /// [`PerformanceHintSession`] failing to create, or the session having little to no benefit
    /// even if creation succeeds.
    #[doc(alias = "APERF_HINT_SESSIONS")]
    Sessions = ffi::APerformanceHintFeature_APERF_HINT_SESSIONS,
    /// The power efficiency mode, as exposed by
    /// [`SessionCreationConfig::set_prefer_power_efficiency()`] and
    /// [`PerformanceHintSession::set_prefer_power_efficiency()`].
    #[doc(alias = "APERF_HINT_POWER_EFFICIENCY")]
    PowerEfficiency = ffi::APerformanceHintFeature_APERF_HINT_POWER_EFFICIENCY,
    /// The ability for sessions to bind to surfaces using `APerformanceHint_setNativeSurfaces()`
    /// or `ASessionCreationConfig_setNativeSurfaces()`.
    #[doc(alias = "APERF_HINT_SURFACE_BINDING")]
    SurfaceBinding = ffi::APerformanceHintFeature_APERF_HINT_SURFACE_BINDING,
    /// The "graphics pipeline" mode, as exposed by
    /// [`SessionCreationConfig::set_graphics_pipeline()`].
    #[doc(alias = "APERF_HINT_GRAPHICS_PIPELINE")]
    GraphicsPipeline = ffi::APerformanceHintFeature_APERF_HINT_GRAPHICS_PIPELINE,
    /// The automatic CPU timing feature, as exposed by
    /// [`SessionCreationConfig::set_use_auto_timing()`].
    #[doc(alias = "APERF_HINT_AUTO_CPU")]
    AutoCpu = ffi::APerformanceHintFeature_APERF_HINT_AUTO_CPU,
    /// The automatic GPU timing feature, as exposed by
    /// [`SessionCreationConfig::set_use_auto_timing()`].
    #[doc(alias = "APERF_HINT_AUTO_GPU")]
    AutoGpu = ffi::APerformanceHintFeature_APERF_HINT_AUTO_GPU,
}

/// Converts the positive `errno` values returned by the performance hint API.
#[cfg(feature = "api-level-36")]
fn errno_to_io_result(status: i32) -> Result<()> {
    match status {
        0 => Ok(()),
        r => Err(Error::from_raw_os_error(r.abs())),
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
/// All timings should be from [`libc::CLOCK_MONOTONIC`].
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
                    .expect("Supplied duration is too large"),
            )
        })
    }

    /// Reports the actual duration for the last cycle of work.
    ///
    /// The system will attempt to adjust the scheduling and performance of the threads within the
    /// thread group to bring the actual duration close to the target duration.
    ///
    /// `actual_duration` is the duration of time the thread group took to complete its last task.
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
                    .expect("Supplied duration is too large"),
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

    /// Informs the framework of an upcoming increase in the workload of this session.
    ///
    /// The hint can be specified to affect the `cpu`, `gpu`, or both. It should be sent shortly
    /// before the start of the cycle where the workload is going to change, or as early as possible
    /// during that cycle for maximum effect. Unsupported hints are silently dropped, and sending a
    /// hint for both CPU and GPU counts as two separate hints for the rate limiter. These hints
    /// should not be sent repeatedly for an ongoing expensive workload, as workload time reporting
    /// is intended to handle this.
    ///
    /// `identifier` is a required string used to distinguish this specific hint.
    ///
    /// # Returns
    /// - [`ErrorKind::ResourceBusy`] if the hint was rate limited.
    /// - [`ErrorKind::BrokenPipe`] if communication with the system service has failed.
    #[cfg(feature = "api-level-36")]
    #[doc(alias = "APerformanceHint_notifyWorkloadIncrease")]
    pub fn notify_workload_increase(&self, cpu: bool, gpu: bool, identifier: &CStr) -> Result<()> {
        errno_to_io_result(unsafe {
            ffi::APerformanceHint_notifyWorkloadIncrease(
                self.ptr.as_ptr(),
                cpu,
                gpu,
                identifier.as_ptr(),
            )
        })
    }

    /// Informs the framework that the workload associated with this session is about to start, or
    /// that it is about to completely change, and that the system should discard any assumptions
    /// about its characteristics inferred from previous activity.
    ///
    /// The hint can be specified to affect the `cpu`, `gpu`, or both. It should be sent shortly
    /// before the start of the cycle where the workload is going to change, or as early as possible
    /// during that cycle for maximum effect. Unsupported hints are silently dropped, and sending a
    /// hint for both CPU and GPU counts as two separate hints for the rate limiter. These hints
    /// should not be sent repeatedly for an ongoing expensive workload, as workload time reporting
    /// is intended to handle this.
    ///
    /// `identifier` is a required string used to distinguish this specific hint.
    ///
    /// # Returns
    /// - [`ErrorKind::ResourceBusy`] if the hint was rate limited.
    /// - [`ErrorKind::BrokenPipe`] if communication with the system service has failed.
    #[cfg(feature = "api-level-36")]
    #[doc(alias = "APerformanceHint_notifyWorkloadReset")]
    pub fn notify_workload_reset(&self, cpu: bool, gpu: bool, identifier: &CStr) -> Result<()> {
        errno_to_io_result(unsafe {
            ffi::APerformanceHint_notifyWorkloadReset(
                self.ptr.as_ptr(),
                cpu,
                gpu,
                identifier.as_ptr(),
            )
        })
    }

    /// Informs the framework of an upcoming one-off expensive workload cycle for this session.
    ///
    /// This cycle will be treated as not representative of the workload as a whole, and it will be
    /// discarded for the purposes of load tracking.
    ///
    /// The hint can be specified to affect the `cpu`, `gpu`, or both. It should be sent shortly
    /// before the start of the cycle where the workload is going to change, or as early as possible
    /// during that cycle for maximum effect. Unsupported hints are silently dropped, and sending a
    /// hint for both CPU and GPU counts as two separate hints for the rate limiter. These hints
    /// should not be sent repeatedly for an ongoing expensive workload, as workload time reporting
    /// is intended to handle this.
    ///
    /// `identifier` is a required string used to distinguish this specific hint.
    ///
    /// # Returns
    /// - [`ErrorKind::ResourceBusy`] if the hint was rate limited.
    /// - [`ErrorKind::BrokenPipe`] if communication with the system service has failed.
    #[cfg(feature = "api-level-36")]
    #[doc(alias = "APerformanceHint_notifyWorkloadSpike")]
    pub fn notify_workload_spike(&self, cpu: bool, gpu: bool, identifier: &CStr) -> Result<()> {
        errno_to_io_result(unsafe {
            ffi::APerformanceHint_notifyWorkloadSpike(
                self.ptr.as_ptr(),
                cpu,
                gpu,
                identifier.as_ptr(),
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
    /// [`libc::CLOCK_MONOTONIC`] about when the work starts.  This timestamp must be greater than
    /// [`Duration::ZERO`].
    #[doc(alias = "AWorkDuration_setWorkPeriodStartTimestampNanos")]
    pub fn set_work_period_start_timestamp(&self, work_period_start_timestamp: Duration) {
        unsafe {
            ffi::AWorkDuration_setWorkPeriodStartTimestampNanos(
                self.ptr.as_ptr(),
                work_period_start_timestamp
                    .as_nanos()
                    .try_into()
                    .expect("Supplied timestamp is too large"),
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
                    .expect("Supplied duration is too large"),
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
                    .expect("Supplied duration is too large"),
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
                    .expect("Supplied duration is too large"),
            )
        }
    }
}

#[cfg(feature = "api-level-35")]
impl Default for WorkDuration {
    #[doc(alias = "AWorkDuration_create")]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "api-level-35")]
impl Drop for WorkDuration {
    /// Destroys [`WorkDuration`] and free all resources associated to it.
    #[doc(alias = "AWorkDuration_release")]
    fn drop(&mut self) {
        unsafe { ffi::AWorkDuration_release(self.ptr.as_ptr()) }
    }
}

/// Configuration for [`PerformanceHintManager::create_session_using_config()`].
#[cfg(feature = "api-level-36")]
#[derive(Debug, PartialEq, Eq, Hash)]
#[doc(alias = "ASessionCreationConfig")]
pub struct SessionCreationConfig {
    ptr: NonNull<ffi::ASessionCreationConfig>,
}

#[cfg(feature = "api-level-36")]
impl SessionCreationConfig {
    /// Creates a new [`SessionCreationConfig`].
    #[doc(alias = "ASessionCreationConfig_create")]
    pub fn new() -> Self {
        Self {
            ptr: NonNull::new(unsafe { ffi::ASessionCreationConfig_create() })
                .expect("ASessionCreationConfig_create should not return NULL"),
        }
    }

    /// Sets the threads to be associated with the session to be created. They must be part of this
    /// process' thread group.
    #[doc(alias = "ASessionCreationConfig_setTids")]
    pub fn set_threads(&self, thread_ids: &[i32]) {
        unsafe {
            ffi::ASessionCreationConfig_setTids(
                self.ptr.as_ptr(),
                thread_ids.as_ptr(),
                thread_ids.len(),
            )
        }
    }

    /// Sets the initial target work duration for the session to be created.
    ///
    /// This must be positive to use the work duration API, and may be [`Duration::ZERO`]
    /// otherwise.
    #[doc(alias = "ASessionCreationConfig_setTargetWorkDurationNanos")]
    pub fn set_target_work_duration(&self, target_work_duration: Duration) {
        unsafe {
            ffi::ASessionCreationConfig_setTargetWorkDurationNanos(
                self.ptr.as_ptr(),
                target_work_duration
                    .as_nanos()
                    .try_into()
                    .expect("Supplied duration is too large"),
            )
        }
    }

    /// Sets whether power efficiency mode will be enabled for the session, telling it that its
    /// threads can be safely scheduled to prefer power efficiency over performance.
    #[doc(alias = "ASessionCreationConfig_setPreferPowerEfficiency")]
    pub fn set_prefer_power_efficiency(&self, enabled: bool) {
        unsafe { ffi::ASessionCreationConfig_setPreferPowerEfficiency(self.ptr.as_ptr(), enabled) }
    }

    /// Sets whether this session manages a graphics pipeline's critical path.
    ///
    /// Such sessions are expected to time the critical path of the graphics pipeline from end to
    /// end, with the total work duration representing the time from the start of frame production
    /// until the buffer is fully finished drawing. It should include any threads on the critical
    /// path of that pipeline, up to a limit accessible from
    /// [`PerformanceHintManager::max_graphics_pipeline_threads_count()`].
    #[doc(alias = "ASessionCreationConfig_setGraphicsPipeline")]
    pub fn set_graphics_pipeline(&self, enabled: bool) {
        unsafe { ffi::ASessionCreationConfig_setGraphicsPipeline(self.ptr.as_ptr(), enabled) }
    }

    /// Enables automatic timing mode for the `cpu`, `gpu`, or both, for sessions using
    /// [graphics pipeline mode][Self::set_graphics_pipeline()] with an attached surface. In this
    /// mode, sessions do not need to report timing data for the respective units.
    ///
    /// Check [`PerformanceHintFeature::AutoCpu`] and [`PerformanceHintFeature::AutoGpu`] with
    /// [`PerformanceHintManager::is_feature_supported()`] and fall back to manual timing if they
    /// are not supported, as session creation will fail otherwise.
    #[doc(alias = "ASessionCreationConfig_setUseAutoTiming")]
    pub fn set_use_auto_timing(&self, cpu: bool, gpu: bool) {
        unsafe { ffi::ASessionCreationConfig_setUseAutoTiming(self.ptr.as_ptr(), cpu, gpu) }
    }
}

#[cfg(feature = "api-level-36")]
impl Default for SessionCreationConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "api-level-36")]
impl Drop for SessionCreationConfig {
    #[doc(alias = "ASessionCreationConfig_release")]
    fn drop(&mut self) {
        unsafe { ffi::ASessionCreationConfig_release(self.ptr.as_ptr()) }
    }
}
