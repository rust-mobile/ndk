#![cfg(feature = "api-level-24")]
//! Bindings for [`AChoreographer`], [`AVsyncId`], and [`AChoreographerFrameCallbackData`]
//!
//! See <https://developer.android.com/ndk/reference/group/choreographer> for an overview of
//! how these types operate.
//!
//! [`AChoreographer`]: https://developer.android.com/ndk/reference/group/choreographer#achoreographer
//! [`AVsyncId`]: https://developer.android.com/ndk/reference/group/choreographer#avsyncid
//! [`AChoreographerFrameCallbackData`]: https://developer.android.com/ndk/reference/group/choreographer#achoreographerframecallbackdata
//!
//! ---
//!
//! Choreographer coordinates the timing of frame rendering. This is the C version of the
//! [`android.view.Choreographer`] object in Java.
//!
//! [`android.view.Choreographer`]: https://developer.android.com/reference/android/view/Choreographer
//!
//! As of API level 33, apps can follow proper frame pacing and even choose a future frame to render.
//! The API is used as follows:
//! 1. The app posts an [`Choreographer::post_vsync_callback()`] to Choreographer to run on the
//!    next frame.
//! 2. The callback is called when it is the time to start the frame with an
//!    [`ChoreographerFrameCallbackData`] payload: information about multiple possible frame
//!    timelines.
//! 3. Apps can choose a frame timeline from the {@link
//!    AChoreographerFrameCallbackData} payload, depending on the frame deadline they can meet when
//!    rendering the frame and their desired presentation time, and subsequently
//!    [notify `SurfaceFlinger`][SurfaceTransaction::set_frame_timeline()]
//!    of the choice. Alternatively, for apps that do not choose a frame timeline, their frame would be
//!    presented at the earliest possible timeline.
//!    TODO: Continuation?
//!    - The preferred frame timeline is the default frame
//!      timeline that the platform scheduled for the app, based on device configuration.
//! 4. SurfaceFlinger attempts to follow the chosen frame timeline, by not applying transactions or
//!    latching buffers before the desired presentation time.

// TODO: Remove "the data pointer" references

use std::{fmt, os::raw::c_void, ptr::NonNull, time::Duration};

#[cfg(doc)]
use crate::looper::ThreadLooper;
#[cfg(all(doc, feature = "api-level-33"))]
use crate::surface_control::SurfaceTransaction;
use crate::utils::abort_on_panic;

/// Prototype of the function that is called when a new frame is being rendered. It's passed the
/// time that the frame is being rendered in the `CLOCK_MONOTONIC` time base. All callbacks that
/// run as part of rendering a frame will observe the same frame time, so it should be used whenever
/// events need to be synchronized (e.g. animations).
///
/// This time is the same value as that of [`ChoreographerFrameCallbackData::frame_time()`], i.e.
/// the "time at which the frame started being rendered".
///
/// A [`Choreographer`] instance is associated with the [`ThreadLooper`] local to the current thread
/// that registers this callback.  The callback will be called, once, on the current thread when
/// that looper is polled for callbacks.
#[doc(alias = "AChoreographer_frameCallback")]
#[doc(alias = "AChoreographer_frameCallback64")]
// TODO: SystemTime because of CLOCK_MONOTONIC?
pub type FrameCallback = Box<dyn FnOnce(Duration)>;

/// Prototype of the function that is called when a new frame is being rendered. It is called with
/// [`ChoreographerFrameCallbackData`] describing multiple frame timelines.
///
/// A [`Choreographer`] instance is associated with the [`ThreadLooper`] local to the current thread
/// that registers this callback.  The callback will be called, once, on the current thread when
/// that looper is polled for callbacks.
#[cfg(feature = "api-level-33")]
#[doc(alias = "AChoreographer_vsyncCallback")]
pub type VsyncCallback = Box<dyn FnOnce(&ChoreographerFrameCallbackData)>;

/// Prototype of the function that is called when the display refresh rate changes. It's passed
/// the new vsync period, as well as the data pointer provided by the application that registered
/// a callback.
///
/// A [`Choreographer`] instance is associated with the [`ThreadLooper`] local to the current
/// thread.  The callback will be called on the current thread when that looper is polled for
/// callbacks.
#[cfg(feature = "api-level-30")]
#[doc(alias = "AChoreographer_refreshRateCallback")]
pub type RefreshRateCallback = Box<dyn FnMut(Duration)>;

/// Opaque type that provides access to a [`Choreographer`] object.
///
/// An instance can be obtained using [`Choreographer::instance()].  This instance is specific to
/// the current thread and [`ThreadLooper`] and cannot be [shared][Send] with other threads.
#[derive(Debug)]
#[doc(alias = "AChoreographer")]
pub struct Choreographer {
    ptr: NonNull<ffi::AChoreographer>,
}

impl Choreographer {
    /// Assumes ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::AChoreographer`].
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AChoreographer>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> NonNull<ffi::AChoreographer> {
        self.ptr
    }

    /// Get the [`Choreographer`] instance for the current thread. This must be called on a
    /// [`ThreadLooper`] thread, otherwise [`None`] is returned.
    // TODO: Document that:
    // 1. Choreographer is not SendSync because its callbacks operate on the looper thread;
    // 2. The user must poll the current looper to actually receive these callbacks;
    // 3. If there's enough synchronization when _(un?)registering_ callbacks, we could create
    //    a `ForeignChoreographer` where callbacks are registered _with_ Send bound.
    pub fn instance() -> Option<Self> {
        let ptr = unsafe { ffi::AChoreographer_getInstance() };
        let ptr = NonNull::new(ptr)?;
        Some(unsafe { Self::from_ptr(ptr) })
    }

    /// Post a callback to be run on the next frame.
    ///
    /// This function automatically uses [`ffi::AChoreographer_postFrameCallback64()`] when compiled
    /// with `api-level-29` or higher.
    // TODO: Perhaps a user wants to support <29, but automatically use the non-deprecated function
    // based on runtime version?
    #[doc(alias = "AChoreographer_postFrameCallback")]
    #[doc(alias = "AChoreographer_postFrameCallback64")]
    pub fn post_frame_callback(&self, callback: FrameCallback) {
        let boxed = Box::new(callback);

        // TODO: i64 != c_long everywhere!
        unsafe extern "C" fn frame_callback(frame_time_nanos: i64, data: *mut c_void) {
            abort_on_panic(|| {
                let boxed = Box::<FrameCallback>::from_raw(data.cast());
                let frame_time = Duration::from_nanos(
                    frame_time_nanos
                        .try_into()
                        .expect("frame_time_nanos should not be negative"),
                );
                (boxed)(frame_time)
            })
        }

        if cfg!(feature = "api-level-29") {
            unsafe {
                ffi::AChoreographer_postFrameCallback64(
                    self.ptr.as_ptr(),
                    Some(frame_callback),
                    Box::into_raw(boxed).cast(),
                )
            }
        } else {
            unsafe {
                ffi::AChoreographer_postFrameCallback(
                    self.ptr.as_ptr(),
                    Some(frame_callback),
                    Box::into_raw(boxed).cast(),
                )
            }
        }
    }

    /// Post a callback to be run on the frame following the specified delay.  The
    /// data pointer provided will be passed to the callback function when it's
    /// called.
    ///
    /// This function automatically uses [`ffi::AChoreographer_postFrameCallbackDelayed64()`] when compiled
    /// with `api-level-29` or higher.
    // TODO: Perhaps a user wants to support <29, but automatically use the non-deprecated function
    // based on runtime version?
    #[doc(alias = "AChoreographer_postFrameCallbackDelayed")]
    #[doc(alias = "AChoreographer_postFrameCallbackDelayed64")]
    pub fn post_frame_callback_delayed(&self, callback: FrameCallback, delay: Duration) {
        let boxed = Box::new(callback);

        // TODO: i64 != c_long everywhere!
        unsafe extern "C" fn frame_callback(frame_time_nanos: i64, data: *mut c_void) {
            abort_on_panic(|| {
                let boxed = Box::<FrameCallback>::from_raw(data.cast());
                let frame_time = Duration::from_nanos(
                    frame_time_nanos
                        .try_into()
                        .expect("frame_time_nanos should not be negative"),
                );
                (boxed)(frame_time)
            })
        }

        let delay = delay.as_millis();

        if cfg!(feature = "api-level-29") {
            let delay = delay
                .try_into()
                .expect("delay milliseconds should fit in u32");
            unsafe {
                ffi::AChoreographer_postFrameCallbackDelayed64(
                    self.ptr.as_ptr(),
                    Some(frame_callback),
                    Box::into_raw(boxed).cast(),
                    delay,
                )
            }
        } else {
            let delay = delay
                .try_into()
                .expect("delay milliseconds should fit in c_long");
            unsafe {
                ffi::AChoreographer_postFrameCallbackDelayed(
                    self.ptr.as_ptr(),
                    Some(frame_callback),
                    Box::into_raw(boxed).cast(),
                    delay,
                )
            }
        }
    }

    /**
     * Posts a callback to be run on the next frame. The data pointer provided will
     * be passed to the callback function when it's called.
     *
     * Available since API level 33.
     */
    #[cfg(feature = "api-level-33")]
    #[doc(alias = "AChoreographer_postVsyncCallback")]
    pub fn post_vsync_callback(&self, callback: VsyncCallback) {
        let boxed = Box::new(callback);

        unsafe extern "C" fn vsync_callback(
            callback_data: *const ffi::AChoreographerFrameCallbackData,
            data: *mut c_void,
        ) {
            abort_on_panic(|| {
                let boxed = Box::<VsyncCallback>::from_raw(data.cast());
                let ptr = NonNull::new(callback_data.cast_mut())
                    .expect("callback_data should not be NULL");
                (boxed)(&ChoreographerFrameCallbackData { ptr })
            })
        }

        unsafe {
            ffi::AChoreographer_postVsyncCallback(
                self.ptr.as_ptr(),
                Some(vsync_callback),
                Box::into_raw(boxed).cast(),
            )
        };
    }

    /**
     * Registers a callback to be run when the display refresh rate changes. The
     * data pointer provided will be passed to the callback function when it's
     * called. The same callback may be registered multiple times, provided that a
     * different data pointer is provided each time.
     *
     * If an application registers a callback for this choreographer instance when
     * no new callbacks were previously registered, that callback is guaranteed to
     * be dispatched. However, if the callback and associated data pointer are
     * unregistered prior to running the callback, then the callback may be silently
     * dropped.
     *
     * This api is thread-safe. Any thread is allowed to register a new refresh
     * rate callback for the choreographer instance.
     *
     * Note that in API level 30, this api is not guaranteed to be atomic with
     * DisplayManager. That is, calling Display#getRefreshRate very soon after
     * a refresh rate callback is invoked may return a stale refresh rate. If any
     * Display properties would be required by this callback, then it is recommended
     * to listen directly to DisplayManager.DisplayListener#onDisplayChanged events
     * instead.
     *
     * As of API level 31, this api is guaranteed to have a consistent view with DisplayManager;
     * Display#getRefreshRate is guaranteed to not return a stale refresh rate when invoked from this
     * callback.
     */
    /// TODO: The returned box should be kept alive TODO make dropping impossible!
    #[cfg(feature = "api-level-30")]
    #[doc(alias = "AChoreographer_registerRefreshRateCallback")]
    pub fn register_refresh_rate_callback(&self, callback: RefreshRateCallback) {
        let mut boxed = Box::new(callback);
        // This box is only freed when unregister() is called
        let data = Box::into_raw(boxed);

        unsafe extern "C" fn refresh_rate_callback(vsync_period_nanos: i64, data: *mut c_void) {
            abort_on_panic(|| {
                let func: *mut RefreshRateCallback = data.cast();
                let vsync_period = Duration::from_nanos(
                    vsync_period_nanos
                        .try_into()
                        .expect("vsync_period_nanos should not be negative"),
                );
                (*func)(vsync_period)
            })
        }
        unsafe {
            ffi::AChoreographer_registerRefreshRateCallback(
                self.ptr.as_ptr(),
                Some(refresh_rate_callback),
                data.cast(),
            )
        }
        // // TODO: Return a handle through which unregister() can be called!
        // boxed
    }

    /**
     * Unregisters a callback to be run when the display refresh rate changes, along
     * with the data pointer previously provided when registering the callback. The
     * callback is only unregistered when the data pointer matches one that was
     * previously registered.
     *
     * This api is thread-safe. Any thread is allowed to unregister an existing
     * refresh rate callback for the choreographer instance. When a refresh rate
     * callback and associated data pointer are unregistered, then there is a
     * guarantee that when the unregistration completes that that callback will not
     * be run with the data pointer passed.
     */
    #[cfg(feature = "api-level-30")]
    #[doc(alias = "AChoreographer_unregisterRefreshRateCallback")]
    pub fn unregister_refresh_rate_callback(&self, og_callback: Box<RefreshRateCallback>) {
        unsafe {
            ffi::AChoreographer_unregisterRefreshRateCallback(
                self.ptr.as_ptr(),
                // Some(refresh_rate_callback),
                Some(todo!()),
                <*mut _>::cast(&mut *og_callback),
            )
        }
    }
}

#[cfg(feature = "api-level-33")]
/// Opaque type that provides access to a [`ChoreographerFrameCallbackData`] object, which contains
/// various methods to extract frame information.
#[doc(alias = "AChoreographerFrameCallbackData")]
pub struct ChoreographerFrameCallbackData {
    ptr: NonNull<ffi::AChoreographerFrameCallbackData>,
}

#[cfg(feature = "api-level-33")]
impl fmt::Debug for ChoreographerFrameCallbackData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChoreographerFrameCallbackData")
            .field("frame_time()", &self.frame_time())
            .field(
                "frame_timelines()",
                &self.frame_timelines().collect::<Vec<_>>(),
            )
            .field(
                "preferred_frame_timeline()",
                &self.preferred_frame_timeline(),
            )
            .finish()
    }
}

#[cfg(feature = "api-level-33")]
impl ChoreographerFrameCallbackData {
    /// The time at which the frame started being rendered.
    ///
    /// Note that this time should **not** be used to advance animation clocks.
    /// Instead, see [`ChoreographerFrameCallbackData::frame_timeline_expected_presentation_time()`].
    #[doc(alias = "AChoreographerFrameCallbackData_getFrameTimeNanos")]
    pub fn frame_time(&self) -> Duration {
        let nanos =
            unsafe { ffi::AChoreographerFrameCallbackData_getFrameTimeNanos(self.ptr.as_ptr()) };
        Duration::from_nanos(
            nanos
                .try_into()
                .expect("frame_time_nanos should not be negative"),
        )
    }

    /// Returns an iterator over possible frame timelines.
    #[doc(alias = "AChoreographerFrameCallbackData_getFrameTimelinesLength")]
    pub fn frame_timelines(&self) -> impl Iterator<Item = FrameTimeline<'_>> {
        let length = unsafe {
            ffi::AChoreographerFrameCallbackData_getFrameTimelinesLength(self.ptr.as_ptr())
        };
        (0..length).map(|i| FrameTimeline(i, self))
    }

    /// Gets the platform-preferred frame timeline.
    ///
    /// The preferred frame timeline is the default by which the platform scheduled the app, based
    /// on the device configuration.
    #[doc(alias = "AChoreographerFrameCallbackData_getPreferredFrameTimelineIndex")]
    pub fn preferred_frame_timeline(&self) -> FrameTimeline<'_> {
        let index = unsafe {
            ffi::AChoreographerFrameCallbackData_getPreferredFrameTimelineIndex(self.ptr.as_ptr())
        };
        FrameTimeline(index, self)
    }
}

#[cfg(feature = "api-level-33")]
pub struct FrameTimeline<'a>(usize, &'a ChoreographerFrameCallbackData);
#[cfg(feature = "api-level-33")]
impl fmt::Debug for FrameTimeline<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrameTimeline")
            .field("index", &self.0)
            .field("vsync_id()", &self.vsync_id())
            .field(
                "expected_presentation_time()",
                &self.expected_presentation_time(),
            )
            .field("deadline()", &self.deadline())
            .finish()
    }
}

#[cfg(feature = "api-level-33")]
impl FrameTimeline<'_> {
    /// Gets the token used by the platform to identify the frame timeline at the given \c index.
    ///
    /// # Parameters
    /// - `index`: index of a frame timeline, in `[0, frame_timelines_length)`. See
    ///   [`ChoreographerFrameCallbackData::frame_timelines_length()`].
    #[doc(alias = "AChoreographerFrameCallbackData_getFrameTimelineVsyncId")]
    pub fn vsync_id(&self) -> ffi::AVsyncId {
        unsafe {
            ffi::AChoreographerFrameCallbackData_getFrameTimelineVsyncId(
                self.1.ptr.as_ptr(),
                self.0,
            )
        }
    }
    /// Gets the time at which the frame described at the given `index` is expected to be presented.
    /// This time should be used to advance any animation clocks.
    ///
    /// # Parameters
    /// - `index`: index of a frame timeline, in `[0, frame_timelines_length)`. See
    ///   [`ChoreographerFrameCallbackData::frame_timelines_length()`].
    #[doc(alias = "AChoreographerFrameCallbackData_getFrameTimelineExpectedPresentationTimeNanos")]
    pub fn expected_presentation_time(&self) -> Duration {
        let nanos = unsafe {
            ffi::AChoreographerFrameCallbackData_getFrameTimelineExpectedPresentationTimeNanos(
                self.1.ptr.as_ptr(),
                self.0,
            )
        };
        Duration::from_nanos(
            nanos
                .try_into()
                .expect("frame_time_nanos should not be negative"),
        )
    }

    /// Gets the time at which the frame described at the given `index` needs to be ready by in
    /// order to be presented on time.
    ///
    /// # Parameters
    /// - `index`: index of a frame timeline, in `[0, frame_timelines_length)`. See
    ///   [`ChoreographerFrameCallbackData::frame_timelines_length()`].
    #[doc(alias = "AChoreographerFrameCallbackData_getFrameTimelineDeadlineNanos")]
    pub fn deadline(&self) -> Duration {
        let nanos = unsafe {
            ffi::AChoreographerFrameCallbackData_getFrameTimelineDeadlineNanos(
                self.1.ptr.as_ptr(),
                self.0,
            )
        };
        Duration::from_nanos(
            nanos
                .try_into()
                .expect("frame_time_nanos should not be negative"),
        )
    }
}
