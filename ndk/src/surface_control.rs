#![cfg(feature = "api-level-29")]
//! Bindings for [`ASurfaceControl`], [`ASurfaceTransaction`], and [`ASurfaceTransactionStats`]
//!
//! See <https://developer.android.com/reference/android/view/SurfaceControl> for an overview of
//! how [`SurfaceControl`] and [`SurfaceTransaction`] operate.
//!
//! [`ASurfaceControl`]: https://developer.android.com/ndk/reference/group/native-activity#asurfacecontrol
//! [`ASurfaceTransaction`]: https://developer.android.com/ndk/reference/group/native-activity#asurfacetransaction
//! [`ASurfaceTransactionStats`]: https://developer.android.com/ndk/reference/group/native-activity#asurfacetransactionstats

use std::{
    ffi::CStr,
    fmt,
    os::fd::{FromRawFd, IntoRawFd, OwnedFd},
    ptr::NonNull,
    time::Duration,
};

#[cfg(feature = "api-level-34")]
use jni_sys::{jobject, JNIEnv};
use num_enum::{FromPrimitive, IntoPrimitive};
#[cfg(feature = "api-level-34")]
use std::mem::ManuallyDrop;

#[cfg(all(doc, feature = "api-level-34"))]
use crate::data_space::DataSpaceRange;
#[cfg(feature = "api-level-31")]
use crate::native_window::ChangeFrameRateStrategy;
#[cfg(feature = "api-level-30")]
use crate::native_window::FrameRateCompatibility;
use crate::{
    data_space::DataSpace,
    hardware_buffer::{HardwareBuffer, Rect},
    native_window::{NativeWindow, NativeWindowTransform},
    utils::abort_on_panic,
};
#[cfg(doc)]
use crate::{hardware_buffer::HardwareBufferUsage, native_activity::NativeActivity};

pub type HdrMetadataSMPTE2086 = ffi::AHdrMetadata_smpte2086;
pub type HdrMetadataCTA861_3 = ffi::AHdrMetadata_cta861_3;

/// The [`SurfaceControl`] API can be used to provide a hierarchy of surfaces for composition to the
/// system compositor. [`SurfaceControl`] represents a content node in this hierarchy.
#[derive(Debug)]
#[doc(alias = "ASurfaceControl")]
pub struct SurfaceControl {
    ptr: NonNull<ffi::ASurfaceControl>,
}

// TODO: What's true here?
unsafe impl Send for SurfaceControl {}
unsafe impl Sync for SurfaceControl {}

impl SurfaceControl {
    /// Assumes ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::ASurfaceControl`].
    pub unsafe fn from_ptr(ptr: NonNull<ffi::ASurfaceControl>) -> Self {
        Self { ptr }
    }

    /// Acquires ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::ASurfaceControl`].
    pub unsafe fn clone_from_ptr(ptr: NonNull<ffi::ASurfaceControl>) -> Self {
        ffi::ASurfaceControl_acquire(ptr.as_ptr());
        Self::from_ptr(ptr)
    }

    pub fn ptr(&self) -> NonNull<ffi::ASurfaceControl> {
        self.ptr
    }

    /// Creates a [`SurfaceControl`] with a [`NativeWindow`] as its parent. `debug_name` is
    /// a debug name associated with this surface. It can be used to identify this surface in
    /// SurfaceFlinger's layer tree.
    ///
    /// By default the [`SurfaceControl`] will be visible and display any buffer
    /// submitted. In addition, the default buffer submission control may release
    /// and not display all buffers that are submitted before receiving a callback
    /// for the previous buffer. See [`SurfaceTransaction::set_visibility()`] and
    /// [`SurfaceTransaction::set_enable_back_pressure()`] to change the default behaviors after
    /// creation.
    ///
    /// Returns [`None`] when you are not allowed to create new [`SurfaceControl`] children for
    /// this [`NativeWindow`].  On the root window from [`NativeActivity`] this is only allowed from
    /// Android 15 onwards [^cite].
    ///
    /// [^cite]: TODO
    #[doc(alias = "ASurfaceControl_createFromWindow")]
    pub fn create_from_window(parent: &NativeWindow, debug_name: &CStr) -> Option<Self> {
        let ptr = unsafe {
            ffi::ASurfaceControl_createFromWindow(parent.ptr().as_ptr(), debug_name.as_ptr())
        };
        let s = NonNull::new(ptr)?;
        Some(unsafe { Self::from_ptr(s) })
    }

    /// Creates a [`SurfaceControl`] with a [`SurfaceControl`] as its parent. `debug_name` is
    /// a debug name associated with this surface. It can be used to identify this surface in
    /// SurfaceFlinger's layer tree.
    #[doc(alias = "ASurfaceControl_create")]
    pub fn create(parent: &Self, debug_name: &CStr) -> Option<Self> {
        let ptr = unsafe { ffi::ASurfaceControl_create(parent.ptr.as_ptr(), debug_name.as_ptr()) };
        let s = NonNull::new(ptr)?;
        Some(unsafe { Self::from_ptr(s) })
    }

    /// Return the [`SurfaceControl`] wrapped by a [Java `SurfaceControl`] object.
    ///
    /// # Safety
    /// `surface_control_obj` must be a non-null instance of [`android.view.SurfaceControl`] and
    /// [`isValid()`] must be [`true`].
    ///
    /// [`android.view.SurfaceControl`]: https://developer.android.com/reference/android/view/SurfaceControl
    /// [`isValid()`]: https://developer.android.com/reference/android/view/SurfaceControl#isValid()
    #[cfg(feature = "api-level-34")]
    #[doc(alias = "ASurfaceControl_fromJava")]
    pub unsafe fn from_java(env: *mut JNIEnv, surface_control_obj: jobject) -> Self {
        let ptr = unsafe { ffi::ASurfaceControl_fromJava(env, surface_control_obj) };
        let ptr = NonNull::new(ptr).expect("ASurfaceControl_fromJava() should never return NULL");
        unsafe { Self::from_ptr(ptr) }
    }
}

impl Drop for SurfaceControl {
    /// The surface and its children may remain on display as long as their parent remains on display.
    // TODO The docs says that this counters an ANativeWindow_acquire? So does create_from_window then take ownership or not?
    #[doc(alias = "ASurfaceControl_release")]
    fn drop(&mut self) {
        unsafe { ffi::ASurfaceControl_release(self.ptr.as_ptr()) }
    }
}

#[cfg(feature = "api-level-31")]
impl Clone for SurfaceControl {
    #[doc(alias = "ASurfaceControl_acquire")]
    fn clone(&self) -> Self {
        unsafe { ffi::ASurfaceControl_acquire(self.ptr.as_ptr()) }
        Self { ptr: self.ptr }
    }
}

/// [`SurfaceTransaction`] is a collection of updates to the surface tree that must be applied
/// atomically.
#[derive(Debug)]
#[doc(alias = "ASurfaceTransaction")]
pub struct SurfaceTransaction {
    ptr: NonNull<ffi::ASurfaceTransaction>,
}

// TODO: What's true here?
unsafe impl Send for SurfaceTransaction {}
unsafe impl Sync for SurfaceTransaction {}

impl SurfaceTransaction {
    /// Assumes ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::ASurfaceControl`].
    pub unsafe fn from_ptr(ptr: NonNull<ffi::ASurfaceTransaction>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> NonNull<ffi::ASurfaceTransaction> {
        self.ptr
    }

    #[doc(alias = "ASurfaceTransaction_create")]
    pub fn new() -> Self {
        let ptr = NonNull::new(unsafe { ffi::ASurfaceTransaction_create() })
            .expect("ASurfaceTransaction_create() should never return NULL");
        unsafe { Self::from_ptr(ptr) }
    }

    /// Return the [`SurfaceTransaction`] wrapped by a [Java `Transaction` object].
    ///
    /// The returned [`SurfaceTransaction`] is still owned by the [Java `Transaction` object]
    /// is only valid while the [Java `Transaction` object] is alive. In particular, the
    /// returned transaction must NOT be deleted with [`drop()`], which is why it is wrapped in
    /// [`ManuallyDrop`].
    ///
    /// # Safety
    /// `transaction_obj` must be a non-null instance of [`android.view.SurfaceControl.Transaction`]
    /// and [`close()`] must not already be called.
    ///
    /// [Java `Transaction` object]: https://developer.android.com/reference/android/view/SurfaceControl.Transaction
    /// [`android.view.SurfaceControl.Transaction`]: https://developer.android.com/reference/android/view/SurfaceControl.Transaction
    /// [`close()`]: https://developer.android.com/reference/android/view/SurfaceControl.Transaction#close()
    #[cfg(feature = "api-level-34")]
    #[doc(alias = "ASurfaceTransaction_fromJava")]
    pub unsafe fn from_java(env: *mut JNIEnv, transaction_obj: jobject) -> ManuallyDrop<Self> {
        let ptr = unsafe { ffi::ASurfaceTransaction_fromJava(env, transaction_obj) };
        let ptr =
            NonNull::new(ptr).expect("ASurfaceTransaction_fromJava() should never return NULL");
        ManuallyDrop::new(unsafe { Self::from_ptr(ptr) })
    }

    /// Applies the updates accumulated in this transaction.
    ///
    /// Note that the transaction is guaranteed to be applied atomically. The transactions which are
    /// applied on the same thread are also guaranteed to be applied in order.
    #[doc(alias = "ASurfaceTransaction_apply")]
    pub fn apply(&self) {
        unsafe { ffi::ASurfaceTransaction_apply(self.ptr.as_ptr()) }
    }

    /// Sets the callback that will be invoked when the updates from this transaction are
    /// presented. For details on the callback semantics and data, see the documentation for
    /// [`OnComplete`].
    ///
    /// Multiple callbacks can be registered, but every callback will only be called once.  You
    /// need to re-register callbacks after every call to [`SurfaceTransaction::apply()`] if reusing
    /// this transaction.  Callbacks may be called after the [`SurfaceTransaction`] object has been
    /// [`Drop`]ped [^1].  This function may leak two [`Box`]es if the transaction isn't
    /// [applied][SurfaceTransaction::apply()], or if Android never invokes the callback.
    ///
    /// [^1]: Hence there is no need to keep the [`SurfaceTransaction`] object around when solely
    /// waiting for the callback.
    // TODO: Why mut, when other things don't grab mut?
    #[doc(alias = "ASurfaceTransaction_setOnComplete")]
    pub fn set_on_complete(&mut self, func: OnComplete) {
        let boxed = Box::new(func);
        unsafe extern "C" fn on_complete(
            context: *mut std::ffi::c_void,
            stats: *mut ffi::ASurfaceTransactionStats,
        ) {
            abort_on_panic(|| {
                let func = Box::<OnComplete>::from_raw(context.cast());
                (func)(&SurfaceTransactionStatsOnComplete(
                    SurfaceTransactionStats {
                        ptr: NonNull::new(stats).unwrap(),
                    },
                ))
            })
        }

        unsafe {
            ffi::ASurfaceTransaction_setOnComplete(
                self.ptr.as_ptr(),
                Box::into_raw(boxed).cast(),
                Some(on_complete),
            )
        }
    }

    /// Sets the callback that will be invoked when the updates from this transaction are applied
    /// and are ready to be presented. This callback will be invoked before the [`OnComplete`]
    /// callback.
    ///
    /// Multiple callbacks can be registered, but every callback will only be called once.  You
    /// need to re-register callbacks after every call to [`SurfaceTransaction::apply()`] if reusing
    /// this transaction.  Callbacks may be called after the [`SurfaceTransaction`] object has been
    /// [`Drop`]ped [^1].  This function may leak two [`Box`]es if the transaction isn't
    /// [applied][SurfaceTransaction::apply()], or if Android never invokes the callback.
    ///
    /// [^1]: Hence there is no need to keep the [`SurfaceTransaction`] object around when solely
    /// waiting for the callback.
    #[cfg(feature = "api-level-31")]
    #[doc(alias = "ASurfaceTransaction_setOnCommit")]
    pub fn set_on_commit(&mut self, func: OnCommit) {
        let boxed = Box::new(func);
        unsafe extern "C" fn on_commit(
            context: *mut std::ffi::c_void,
            stats: *mut ffi::ASurfaceTransactionStats,
        ) {
            abort_on_panic(|| {
                let func = Box::<OnCommit>::from_raw(context.cast());
                (func)(&SurfaceTransactionStats {
                    ptr: NonNull::new(stats).unwrap(),
                })
            })
        }

        unsafe {
            ffi::ASurfaceTransaction_setOnCommit(
                self.ptr.as_ptr(),
                Box::into_raw(boxed).cast(),
                Some(on_commit),
            )
        }
    }

    /// Reparents the `surface_control` from its old parent to the `new_parent` surface control. Any
    /// children of the reparented `surface_control` will remain children of the `surface_control`.
    ///
    /// `new_parent` can be [`None`]. Surface controls without a parent do not appear on the
    /// display.
    #[doc(alias = "ASurfaceTransaction_reparent")]
    pub fn reparent(&self, surface_control: &SurfaceControl, new_parent: Option<&SurfaceControl>) {
        unsafe {
            ffi::ASurfaceTransaction_reparent(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                match new_parent {
                    Some(p) => p.ptr.as_ptr(),
                    None => std::ptr::null_mut(),
                },
            )
        }
    }

    /// Updates the visibility of `surface_control`. If show is set to [`Visibility::Hide`], the
    /// `surface_control` and all surfaces in its subtree will be hidden.
    #[doc(alias = "ASurfaceTransaction_setVisibility")]
    pub fn set_visibility(&self, surface_control: &SurfaceControl, visibility: Visibility) {
        unsafe {
            ffi::ASurfaceTransaction_setVisibility(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                visibility.into(),
            )
        }
    }

    /// Updates the z order index for `surface_control`. Note that the z order for a surface is
    /// relative to other surfaces which are siblings of this surface. The behavior of siblings with
    /// the same z order is undefined.
    ///
    /// Z orders may be any valid [`i32`] value. A layer's default z order index is `0`.
    #[doc(alias = "ASurfaceTransaction_setZOrder")]
    pub fn set_z_order(&self, surface_control: &SurfaceControl, z_order: i32) {
        unsafe {
            ffi::ASurfaceTransaction_setZOrder(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                z_order,
            )
        }
    }

    /// Updates the [`HardwareBuffer`] displayed for `surface_control`. If not [`None`], the
    /// `acquire_fence_fd` should be a file descriptor that is signaled when all pending work for
    /// the buffer is complete and the buffer can be safely read.
    ///
    /// Note that the buffer must be allocated with [`HardwareBufferUsage::GPU_SAMPLED_IMAGE`] as
    /// the surface control might be composited using the GPU.
    #[doc(alias = "ASurfaceTransaction_setBuffer")]
    pub fn set_buffer(
        &self,
        surface_control: &SurfaceControl,
        buffer: &HardwareBuffer,
        acquire_fence_fd: Option<OwnedFd>,
    ) {
        unsafe {
            ffi::ASurfaceTransaction_setBuffer(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                buffer.as_ptr(),
                match acquire_fence_fd {
                    Some(fd) => fd.into_raw_fd(),
                    None => -1,
                },
            )
        }
    }

    /// Updates the color for `surface_control`.  This will make the background color for the
    /// [`SurfaceControl`] visible in transparent regions of the surface.  Colors `r`, `g`, and `b`
    /// must be within the range that is valid for `data_space`.  `data_space` and `alpha` will be
    /// the [`DataSpace`] and alpha set for the background color layer.
    #[doc(alias = "ASurfaceTransaction_setColor")]
    pub fn set_color(
        &self,
        surface_control: &SurfaceControl,
        r: f32,
        g: f32,
        b: f32,
        alpha: f32,
        data_space: DataSpace,
    ) {
        unsafe {
            ffi::ASurfaceTransaction_setColor(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                r,
                g,
                b,
                alpha,
                data_space.into(),
            )
        }
    }

    /// # Parameters
    /// - `source`: The sub-rect within the buffer's content to be rendered inside the surface's
    ///   area The surface's source rect is clipped by the bounds of its current buffer. The source
    ///   rect's width and height must be `> 0`.
    ///
    /// - `destination`: Specifies the rect in the parent's space where this surface will be
    ///   drawn. The post source rect bounds are scaled to fit the destination rect. The surface's
    ///   destination rect is clipped by the bounds of its parent. The destination rect's width and
    ///   height must be `> 0`.
    ///
    /// - `transform`: The transform applied after the source rect is applied to the buffer. This
    ///   parameter should be set to [`NativeWindowTransform::IDENTITY`] for no transform.
    #[deprecated = "Use `set_crop()`, `set_position()`, `set_buffer_transform()`, and `set_scale()` \
                    instead. Those functions provide well defined behavior and allows for more \
                    control by the apps. It also allows the caller to set different properties at \
                    different times, instead of having to specify all the desired properties at \
                    once."]
    #[doc(alias = "ASurfaceTransaction_setGeometry")]
    pub fn set_geometry(
        &self,
        surface_control: &SurfaceControl,
        source: &Rect,
        destination: &Rect,
        transform: NativeWindowTransform,
    ) {
        unsafe {
            ffi::ASurfaceTransaction_setGeometry(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                source,
                destination,
                transform.bits(),
            )
        }
    }

    /// Bounds the surface and its children to the bounds specified. The crop and buffer size will
    /// be used to determine the bounds of the surface. If no crop is specified and the surface has
    /// no buffer, the surface bounds is only constrained by the size of its parent bounds.
    ///
    /// # Parameters
    /// - `crop`: The bounds of the crop to apply.
    #[cfg(feature = "api-level-31")]
    #[doc(alias = "ASurfaceTransaction_setCrop")]
    pub fn set_crop(&self, surface_control: &SurfaceControl, crop: &Rect) {
        unsafe {
            ffi::ASurfaceTransaction_setCrop(self.ptr.as_ptr(), surface_control.ptr.as_ptr(), crop)
        }
    }

    /// Specifies the position in the parent's space where the surface will be drawn.
    ///
    /// # Parameters
    /// - `x`: The x position to render the surface.
    /// - `y`: The y position to render the surface.
    #[cfg(feature = "api-level-31")]
    #[doc(alias = "ASurfaceTransaction_setPosition")]
    pub fn set_position(&self, surface_control: &SurfaceControl, x: i32, y: i32) {
        unsafe {
            ffi::ASurfaceTransaction_setPosition(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                x,
                y,
            )
        }
    }

    /// # Parameters
    /// -`transform`: The transform applied after the source rect is applied to the buffer. This
    ///   parameter should be set to [`NativeWindowTransform::IDENTITY`] for no transform.
    #[cfg(feature = "api-level-31")]
    #[doc(alias = "ASurfaceTransaction_setBufferTransform")]
    pub fn set_buffer_transform(
        &self,
        surface_control: &SurfaceControl,
        transform: NativeWindowTransform,
    ) {
        unsafe {
            ffi::ASurfaceTransaction_setBufferTransform(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                transform.bits(),
            )
        }
    }

    /// Sets an `x` and `y` scale of a surface with `(0, 0)` as the centerpoint of the scale.
    ///
    /// # Parameters
    /// - `x_scale`: The scale in the x direction. Must be greater than `0`.
    /// - `y_scale`: The scale in the y direction. Must be greater than `0`.
    #[cfg(feature = "api-level-31")]
    #[doc(alias = "ASurfaceTransaction_setScale")]
    pub fn set_scale(&self, surface_control: &SurfaceControl, x_scale: f32, y_scale: f32) {
        unsafe {
            ffi::ASurfaceTransaction_setScale(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                x_scale,
                y_scale,
            )
        }
    }

    /// Updates whether the content for the buffer associated with this surface is completely
    /// opaque. If true, every pixel of content inside the buffer must be opaque or visual errors
    /// can occur.
    #[doc(alias = "ASurfaceTransaction_setBufferTransparency")]
    pub fn set_buffer_transparency(
        &self,
        surface_control: &SurfaceControl,
        transparency: Transparency,
    ) {
        unsafe {
            ffi::ASurfaceTransaction_setBufferTransparency(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                transparency.into(),
            )
        }
    }

    /// Updates the region for the content on this surface updated in this transaction. If
    /// unspecified, the complete surface is assumed to be damaged.
    ///
    /// TODO: Provide an empty slice to reset.
    #[doc(alias = "ASurfaceTransaction_setDamageRegion")]
    pub fn set_damage_region(&self, surface_control: &SurfaceControl, rects: &[Rect]) {
        unsafe {
            ffi::ASurfaceTransaction_setDamageRegion(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                if rects.is_empty() {
                    std::ptr::null()
                } else {
                    rects.as_ptr()
                },
                rects.len() as u32,
            )
        }
    }

    /// Specifies a `desired_present_time` for the transaction. The framework will try to present
    /// the transaction at or after the time specified.
    ///
    /// Transactions will not be presented until all of their acquire fences have signaled even if
    /// the app requests an earlier present time.
    ///
    /// If an earlier transaction has a desired present time of `x`, and a later transaction has a
    /// desired present time that is before `x`, the later transaction will not preempt the earlier
    /// transaction.
    #[doc(alias = "ASurfaceTransaction_setDesiredPresentTime")]
    pub fn set_desired_present_time(&self, desired_present_time: Duration) {
        let desired_present_time = desired_present_time
            .as_nanos()
            .try_into()
            .expect("Duration should be convertible to i64 nanoseconds");
        unsafe {
            ffi::ASurfaceTransaction_setDesiredPresentTime(self.ptr.as_ptr(), desired_present_time)
        }
    }

    /// Sets the alpha for the buffer. It uses a premultiplied blending.
    ///
    /// The `alpha` must be between `0.0` and `1.0`.
    #[doc(alias = "ASurfaceTransaction_setBufferAlpha")]
    pub fn set_buffer_alpha(&self, surface_control: &SurfaceControl, alpha: f32) {
        unsafe {
            ffi::ASurfaceTransaction_setBufferAlpha(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                alpha,
            )
        }
    }

    /// Sets the data space of the surface_control's buffers.
    ///
    /// If no data space is set, the surface control defaults to [`DataSpace::Srgb`].
    #[doc(alias = "ASurfaceTransaction_setBufferDataSpace")]
    pub fn set_buffer_data_space(&self, surface_control: &SurfaceControl, data_space: DataSpace) {
        unsafe {
            ffi::ASurfaceTransaction_setBufferDataSpace(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                data_space.into(),
            )
        }
    }

    /// [SMPTE ST 2086 "Mastering Display Color Volume" static metadata]
    ///
    /// When `metadata` is set to [`None`], the framework does not use any SMPTE 2086 metadata when
    /// rendering the surface's buffer.
    ///
    /// [SMPTE ST 2086 "Mastering Display Color Volume" static metadata]: https://ieeexplore.ieee.org/document/8353899
    #[doc(alias = "ASurfaceTransaction_setHdrMetadata_smpte2086")]
    pub fn set_hdr_metadata_smpte2086(
        &self,
        surface_control: &SurfaceControl,
        metadata: Option<&HdrMetadataSMPTE2086>,
    ) {
        unsafe {
            ffi::ASurfaceTransaction_setHdrMetadata_smpte2086(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                // FFI missing const
                match metadata {
                    Some(metadata) => <*const _>::cast_mut(metadata),
                    None => std::ptr::null_mut(),
                },
            )
        }
    }

    /// Sets the CTA 861.3 "HDR Static Metadata Extension" static metadata on a surface.
    ///
    /// When `metadata` is set to [`None`], the framework does not use any CTA 861.3 metadata when
    /// rendering the surface's buffer.
    // TODO Link
    #[doc(alias = "ASurfaceTransaction_setHdrMetadata_cta861_3")]
    pub fn set_hdr_metadata_cta861_3(
        &self,
        surface_control: &SurfaceControl,
        metadata: Option<&HdrMetadataCTA861_3>,
    ) {
        unsafe {
            ffi::ASurfaceTransaction_setHdrMetadata_cta861_3(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                // FFI missing const
                match metadata {
                    Some(metadata) => <*const _>::cast_mut(metadata),
                    None => std::ptr::null_mut(),
                },
            )
        }
    }

    /// Sets the desired extended range brightness for the layer. This only applies
    /// for layers whose [`DataSpace`] has [`DataSpaceRange::Extended`] set on it. See:
    /// [`SurfaceTransaction::set_desired_hdr_headroom()`], prefer using this API for formats that
    /// encode an HDR/ SDR ratio as part of generating the buffer.
    ///
    /// # Parameters
    /// - `surface_control`: The layer whose extended range brightness is being specified
    /// - `current_buffer_ratio`: The current hdr/sdr ratio of the current buffer as represented
    ///   as `peakHdrBrightnessInNits / targetSdrWhitePointInNits`. For example if the buffer
    ///   was rendered with a target SDR whitepoint of `100nits` and a max display brightness of
    ///   `200nits`, this should be set to `2.0`.
    ///
    ///   Default value is `1.0`.
    ///
    ///   Transfer functions that encode their own brightness ranges, such as HLG or PQ, should
    ///   also set this to `1.0` and instead communicate extended content brightness information via
    ///   metadata such as [CTA861_3] or [SMPTE2086].
    ///
    ///   [CTA861_3]: HdrMetadataCTA861_3
    ///   [SMPTE2086]: HdrMetadataSMPTE2086
    ///
    ///   Must be `finite && >= 1.0`.
    ///
    /// - `desired_ratio`: The desired hdr/sdr ratio as represented as `peakHdrBrightnessInNits /
    ///   targetSdrWhitePointInNits`. This can be used to communicate the max desired brightness
    ///   range. This is similar to the "max luminance" value in other HDR metadata formats, but
    ///   represented as a ratio of the target SDR whitepoint to the max display brightness. The
    ///   system may not be able to, or may choose not to, deliver the requested range.
    ///
    ///   While requesting a large desired ratio will result in the most dynamic range, voluntarily
    ///   reducing the requested range can help improve battery life as well as can improve quality
    ///   by ensuring greater bit depth is allocated to the luminance range in use.
    ///
    ///   When called after [`SurfaceTransaction::set_desired_hdr_headroom()`],
    ///   the `desired_ratio` will override the `desired_headroom` provided by
    ///   [`SurfaceTransaction::set_desired_hdr_headroom()`]. Conversely, when called before
    ///   [`SurfaceTransaction::set_desired_hdr_headroom()`], the `desired_headroom` provided by
    ///   [`SurfaceTransaction::set_desired_hdr_headroom()`] will override the `desired_ratio`.
    ///
    ///   Default value is `1.0` and indicates that extended range brightness is not being used, so
    ///   the resulting SDR or HDR behavior will be determined entirely by the dataspace being used
    ///   (ie, typically SDR however PQ or HLG transfer functions will still result in HDR).
    ///
    ///    Must be `finite && >= 1.0`.
    #[cfg(feature = "api-level-34")]
    #[doc(alias = "ASurfaceTransaction_setExtendedRangeBrightness")]
    pub fn set_extended_range_brightness(
        &self,
        surface_control: &SurfaceControl,
        current_buffer_ratio: f32,
        desired_ratio: f32,
    ) {
        unsafe {
            ffi::ASurfaceTransaction_setExtendedRangeBrightness(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                current_buffer_ratio,
                desired_ratio,
            )
        }
    }

    /// Sets the desired HDR headroom for the layer. See:
    /// [`SurfaceTransaction::set_extended_range_brightness()`], prefer using this API for formats
    /// that conform to HDR standards like HLG or HDR10, that do not communicate a HDR/SDR ratio as
    /// part of generating the buffer.
    ///
    /// # Parameters
    /// - `surface_control`: The layer whose desired HDR headroom is being specified
    ///
    /// - `desired_headroom`: The desired hdr/sdr ratio as represented as `peakHdrBrightnessInNits /
    ///   targetSdrWhitePointInNits`. This can be used to communicate the max desired brightness
    ///   range of the panel. The system may not be able to, or may choose not to, deliver the
    ///   requested range.
    ///
    ///   While requesting a large desired ratio will result in the most dynamic range, voluntarily
    ///   reducing the requested range can help improve battery life as well as can improve quality
    ///   by ensuring greater bit depth is allocated to the luminance range in use.
    ///
    ///   Default value is `0.0` and indicates that the system will choose the best headroom for
    ///   this surface control's content. Typically, this means that HLG/PQ encoded content will be
    ///   displayed with some HDR headroom greater than `1.0`.
    ///
    ///   When called after [`SurfaceTransaction::set_extended_range_brightness()`],
    ///   the `desired_headroom` will override the `desired_ratio` provided by
    ///   [`SurfaceTransaction::set_extended_range_brightness()`]. Conversely, when called
    ///   before [`SurfaceTransaction::set_extended_range_brightness()`], the `desired_ratio`
    ///   provided by [`SurfaceTransaction::set_extended_range_brightness()`] will override the
    ///   `desired_headroom`.
    ///
    ///   Must be `finite && >= 1.0` or `0.0` to indicate there is no desired headroom.
    #[cfg(feature = "api-level-35")]
    #[doc(alias = "ASurfaceTransaction_setDesiredHdrHeadroom")]
    pub fn set_desired_hdr_headroom(
        &self,

        surface_control: &SurfaceControl,
        desired_headroom: f32,
    ) {
        unsafe {
            ffi::ASurfaceTransaction_setDesiredHdrHeadroom(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                desired_headroom,
            )
        }
    }

    /// Same as [`set_frame_rate_with_change_strategy(transaction, surface_control,
    /// frame_rate, compatibility, ChangeFrameRateStrategy::OnlyIfSeamless)`][Self::set_frame_rate_with_change_strategy()].
    ///
    #[cfg_attr(
        not(feature = "api-level-31"),
        doc = "[Self::set_frame_rate_with_change_strategy()]: https://developer.android.com/ndk/reference/group/native-activity#asurfacetransaction_setframeratewithchangestrategy"
    )]
    #[cfg(feature = "api-level-30")]
    #[doc(alias = "ASurfaceTransaction_setFrameRate")]
    pub fn set_frame_rate(
        &self,
        surface_control: &SurfaceControl,
        frame_rate: f32,
        compatibility: FrameRateCompatibility,
    ) {
        unsafe {
            ffi::ASurfaceTransaction_setFrameRate(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                frame_rate,
                compatibility as i8,
            )
        }
    }

    /// Sets the intended frame rate for `surface_control`.
    ///
    /// On devices that are capable of running the display at different refresh rates, the system
    /// may choose a display refresh rate to better match this surface's frame rate. Usage of this
    /// API won't directly affect the application's frame production pipeline. However, because the
    /// system may change the display refresh rate, calls to this function may result in changes
    /// to `Choreographer` callback timings, and changes to the time interval at which the system
    /// releases buffers back to the application.
    ///
    /// You can register for changes in the refresh rate using
    /// [`ffi::AChoreographer_registerRefreshRateCallback()`].
    ///
    #[cfg_attr(
        feature = "api-level-34",
        doc = "See [`SurfaceTransaction::clear_frame_rate()`]."
    )]
    ///
    /// # Parameters
    /// - `frame_rate`: The intended frame rate of this surface, in frames per second. `0` is a
    ///   special value that indicates the app will accept the system's choice for the display frame
    ///   rate, which is the default behavior if this function isn't called. The `frame_rate` param
    ///   does _not_ need to be a valid refresh rate for this device's display - e.g., it's fine to
    ///   pass `30`fps to a device that can only run the display at `60`fps.
    ///
    /// - `compatibility`: The frame rate compatibility of this surface. The compatibility value
    ///   may influence the system's choice of display frame rate. This parameter is ignored when
    ///   `frame_rate` is `0`.
    ///
    /// - `change_frame_rate_strategy`: Whether display refresh rate transitions caused by
    ///   this surface should be seamless. A seamless transition is one that doesn't have
    ///   any visual interruptions, such as a black screen for a second or two. See the
    ///   [`ChangeFrameRateStrategy`] values. This parameter is ignored when `frame_rate` is `0`.
    #[cfg(feature = "api-level-31")]
    #[doc(alias = "ASurfaceTransaction_setFrameRateWithChangeStrategy")]
    pub fn set_frame_rate_with_change_strategy(
        &self,
        surface_control: &SurfaceControl,
        frame_rate: f32,
        compatibility: FrameRateCompatibility,
        change_frame_rate_strategy: ChangeFrameRateStrategy,
    ) {
        unsafe {
            ffi::ASurfaceTransaction_setFrameRateWithChangeStrategy(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                frame_rate,
                compatibility as i8,
                change_frame_rate_strategy as i8,
            )
        }
    }

    /// Clears the frame rate which is set for `surface_control`.
    ///
    /// This is equivalent to calling [`set_frame_rate_with_change_strategy(transaction, 0,
    /// compatibility, change_frame_rate_strategy)`][Self::set_frame_rate_with_change_strategy()].
    ///
    /// Usage of this API won't directly affect the application's frame production pipeline.
    /// However, because the system may change the display refresh rate, calls to this function may
    /// result in changes to `Choreographer` callback timings, and changes to the time interval at
    /// which the system releases buffers back to the application.
    ///
    /// You can register for changes in the refresh rate using
    /// [`ffi::AChoreographer_registerRefreshRateCallback()`].
    #[cfg(feature = "api-level-34")]
    #[doc(alias = "ASurfaceTransaction_clearFrameRate")]
    pub fn clear_frame_rate(&self, surface_control: &SurfaceControl) {
        unsafe {
            ffi::ASurfaceTransaction_clearFrameRate(self.ptr.as_ptr(), surface_control.ptr.as_ptr())
        }
    }

    /// Indicate whether to enable backpressure for buffer submission to a given [`SurfaceControl`].
    ///
    /// By default backpressure is disabled, which means submitting a buffer prior to receiving a
    /// callback for the previous buffer could lead to that buffer being "dropped". In cases where
    /// you are selecting for latency, this may be a desirable behavior! We had a new buffer ready,
    /// why shouldn't we show it?
    ///
    /// When back pressure is enabled, each buffer will be required to be presented before it is
    /// released and the callback delivered (absent the whole [`SurfaceControl`] being removed).
    ///
    /// Most apps are likely to have some sort of backpressure internally, e.g. you are waiting
    /// on the callback from frame `N-`2 before starting frame `N`. In high refresh rate scenarios
    /// there may not be much time between `SurfaceFlinger` completing frame `N-1` (and therefore
    /// releasing buffer `N-2`) and beginning frame `N`. This means your app may not have enough
    /// time to respond in the callback. Using this flag and pushing buffers earlier for server side
    /// queuing will be advantageous in such cases.
    ///
    /// # Parameters
    /// - `transaction`: The transaction in which to make the change.
    /// - `surface_control`: The [`SurfaceControl`] on which to control buffer backpressure behavior.
    /// - `enable_back_pressure`: Whether to enable back pressure.
    #[cfg(feature = "api-level-31")]
    #[doc(alias = "ASurfaceTransaction_setEnableBackPressure")]
    pub fn set_enable_back_pressure(
        &self,
        surface_control: &SurfaceControl,
        enable_back_pressure: bool,
    ) {
        unsafe {
            ffi::ASurfaceTransaction_setEnableBackPressure(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
                enable_back_pressure,
            )
        }
    }

    /// Sets the frame timeline to use in `SurfaceFlinger`.
    ///
    /// A frame timeline should be chosen based on the frame deadline the application can meet
    /// when rendering the frame and the application's desired presentation time. By setting a
    /// frame timeline, `SurfaceFlinger` tries to present the frame at the corresponding expected
    /// presentation time.
    ///
    /// To receive frame timelines, a callback must be posted to `Choreographer` using
    /// [`ffi::AChoreographer_postVsyncCallback()`]. The `vsync_id` can then be extracted from the
    /// callback payload using [`ffi::AChoreographerFrameCallbackData_getFrameTimelineVsyncId()`].
    ///
    /// # Parameters
    /// - `vsync_id`: The vsync ID received from [`ffi::AChoreographer`], setting the frame's
    ///   presentation target to the corresponding expected presentation time and deadline from the
    ///   frame to be rendered. A stale or invalid value will be ignored.
    #[cfg(feature = "api-level-33")]
    #[doc(alias = "ASurfaceTransaction_setFrameTimeline")]
    pub fn set_frame_timeline(&self, vsync_id: ffi::AVsyncId) {
        unsafe { ffi::ASurfaceTransaction_setFrameTimeline(self.ptr.as_ptr(), vsync_id) }
    }
}

impl Drop for SurfaceTransaction {
    #[doc(alias = "ASurfaceTransaction_delete")]
    fn drop(&mut self) {
        unsafe { ffi::ASurfaceTransaction_delete(self.ptr.as_ptr()) }
    }
}

/// Since the transactions are applied asynchronously, the [`OnComplete`] callback can be used to be
/// notified when a frame including the updates in a transaction was presented.
///
/// Buffers which are replaced or removed from the scene in the transaction invoking this callback
/// may be reused after this point.
///
/// # Parameters
/// - `stats`: [`SurfaceTransactionStats`] handle to query information about the transaction.
#[doc(alias = "ASurfaceTransaction_OnComplete")]
pub type OnComplete = Box<dyn FnOnce(&SurfaceTransactionStatsOnComplete) + Send>;

/// The [`OnCommit`] callback is invoked when this transaction is applied and the updates are ready
/// to be presented.  This callback will be invoked before the [`OnComplete`] callback.
///
/// This callback does not mean buffers have been released! It simply means that any new
/// transactions applied will not overwrite the transaction for which we are receiving a callback
/// and instead will be included in the next frame. If you are trying to avoid dropping frames
/// (overwriting transactions), and unable to use timestamps (which provide a more efficient
/// solution), then this method provides a method to pace your transaction application.
///
/// # Parameters
/// - `stats`: [`SurfaceTransactionStats`] handle to query information about the
///   transaction. Present and release fences are not available for this callback.
#[cfg(feature = "api-level-31")]
#[doc(alias = "ASurfaceTransaction_OnCommit")]
pub type OnCommit = Box<dyn FnOnce(&SurfaceTransactionStats) + Send>;

/// An opaque handle returned during a callback that can be used to query general stats and stats
/// for surfaces which were either removed or for which buffers were updated after this transaction
/// was applied.
#[doc(alias = "ASurfaceTransactionStats")]
pub struct SurfaceTransactionStats {
    ptr: NonNull<ffi::ASurfaceTransactionStats>,
}

// TODO: Document extra getters
/// [`SurfaceTransactionStats`] with extra getters that are only valid to call in the [`OnComplete`]
/// callback, but not in the [`OnCommit`] callback.
///
#[cfg_attr(
    not(feature = "api-level-31"),
    doc = "[`OnCommit`]: https://developer.android.com/ndk/reference/group/native-activity#group___native_activity_1ga3465b02096890aba8011a71efe0241f2"
)]
#[doc(alias = "ASurfaceTransactionStats")]
#[doc(alias = "SurfaceTransactionStats")]
pub struct SurfaceTransactionStatsOnComplete(SurfaceTransactionStats);

impl std::ops::Deref for SurfaceTransactionStatsOnComplete {
    type Target = SurfaceTransactionStats;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for SurfaceTransactionStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugSurfaceControl<'a>(&'a SurfaceTransactionStats, &'a SurfaceControl);
        impl<'a> fmt::Debug for DebugSurfaceControl<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("SurfaceControl Stats")
                    .field("surface_control", &self.1)
                    .field("acquire_time", &self.0.acquire_time(self.1))
                    .finish()
            }
        }
        struct DebugSurfaceControls<'a>(&'a SurfaceTransactionStats);
        impl<'a> fmt::Debug for DebugSurfaceControls<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list()
                    .entries(
                        self.0
                            .surface_controls()
                            .as_mut()
                            .iter()
                            .map(|sc| DebugSurfaceControl(self.0, sc)),
                    )
                    .finish()
            }
        }
        f.debug_struct("SurfaceTransactionStats")
            .field("latch_time", &self.latch_time())
            .field("surface_controls", &DebugSurfaceControls(self))
            .finish()
    }
}

impl fmt::Debug for SurfaceTransactionStatsOnComplete {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugSurfaceControl<'a>(&'a SurfaceTransactionStatsOnComplete, &'a SurfaceControl);
        impl<'a> fmt::Debug for DebugSurfaceControl<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("SurfaceControl Stats")
                    .field("surface_control", &self.1)
                    .field("acquire_time", &self.0.acquire_time(self.1))
                    .field(
                        "previous_release_fence_fd",
                        &self.0.previous_release_fence_fd(self.1),
                    )
                    .finish()
            }
        }
        struct DebugSurfaceControls<'a>(&'a SurfaceTransactionStatsOnComplete);
        impl<'a> fmt::Debug for DebugSurfaceControls<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list()
                    .entries(
                        self.0
                            .surface_controls()
                            .as_mut()
                            .iter()
                            .map(|sc| DebugSurfaceControl(self.0, sc)),
                    )
                    .finish()
            }
        }
        f.debug_struct("SurfaceTransactionStats")
            .field("latch_time", &self.latch_time())
            .field("present_fence_fd", &self.present_fence_fd())
            .field("surface_controls", &DebugSurfaceControls(self))
            .finish()
    }
}

impl SurfaceTransactionStats {
    /// Returns the timestamp of when the frame was latched by the framework. Once a frame is
    /// latched by the framework, it is presented at the following hardware vsync.
    #[doc(alias = "ASurfaceTransactionStats_getLatchTime")]
    pub fn latch_time(&self) -> Duration {
        let ts = unsafe { ffi::ASurfaceTransactionStats_getLatchTime(self.ptr.as_ptr()) };
        Duration::from_nanos(
            ts.try_into()
                .expect("getLatchTime() should not return a negative timestamp"),
        )
    }

    /// Returns an array of [`SurfaceControl`] pointers that were updated during the transaction.
    /// Stats for the surfaces can be queried through [`SurfaceTransactionStats`] functions.
    #[doc(alias = "ASurfaceTransactionStats_getASurfaceControls")]
    pub fn surface_controls(&self) -> SurfaceControls {
        let mut array = std::mem::MaybeUninit::uninit();
        let mut count = std::mem::MaybeUninit::uninit();
        unsafe {
            ffi::ASurfaceTransactionStats_getASurfaceControls(
                self.ptr.as_ptr(),
                array.as_mut_ptr(),
                count.as_mut_ptr(),
            )
        };
        SurfaceControls {
            array: unsafe { array.assume_init() },
            count: unsafe { count.assume_init() },
        }
    }

    /// Returns the timestamp of when the *CURRENT* buffer was acquired. A buffer is considered
    /// acquired when its `acquire_fence_fd` has signaled. A buffer cannot be latched or presented
    /// until it is acquired. If no `acquire_fence_fd` was provided, this timestamp will be set
    /// to [`None`].
    #[doc(alias = "ASurfaceTransactionStats_getAcquireTime")]
    pub fn acquire_time(&self, surface_control: &SurfaceControl) -> Option<Duration> {
        let ts = unsafe {
            ffi::ASurfaceTransactionStats_getAcquireTime(
                self.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
            )
        };
        match ts {
            -1 => None,
            ts => Some(Duration::from_nanos(
                ts.try_into()
                    .expect("getAcquireTime() should not return a negative timestamp"),
            )),
        }
    }
}

impl SurfaceTransactionStatsOnComplete {
    /// Returns a sync fence that signals when the transaction has been presented. If a device does
    /// not support present fences, [`None`] will be returned.
    ///
    /// This query is not valid for the [`OnCommit`] callback.
    ///
    #[cfg_attr(
        not(feature = "api-level-31"),
        doc = "[`OnCommit`]: https://developer.android.com/ndk/reference/group/native-activity#group___native_activity_1ga3465b02096890aba8011a71efe0241f2"
    )]
    #[doc(alias = "ASurfaceTransactionStats_getPresentFenceFd")]
    pub fn present_fence_fd(&self) -> Option<OwnedFd> {
        let fd = unsafe { ffi::ASurfaceTransactionStats_getPresentFenceFd(self.0.ptr.as_ptr()) };
        match fd {
            -1 => None,
            fd => Some(unsafe { OwnedFd::from_raw_fd(fd) }),
        }
    }

    /// Returns the fence used to signal the release of the PREVIOUS buffer set on this surface.
    /// If this fence is valid ([`Some`]), the PREVIOUS buffer has not yet been released and the
    /// fence will signal when the PREVIOUS buffer has been released. If the fence is [`None`], the
    /// PREVIOUS buffer is already released.
    ///
    /// Each time a buffer is set through [`SurfaceTransaction::set_buffer()`] on a transaction
    /// which is applied, the framework takes a ref on this buffer. The framework treats the
    /// addition of a buffer to a particular surface as a unique ref. When a transaction updates
    /// or removes a buffer from a surface, or removes the surface itself from the tree, this
    /// ref is guaranteed to be released in the [`OnComplete`] callback for this transaction.
    /// The [`SurfaceTransactionStats`] provided in the callback for this surface may contain an
    /// optional fence which must be signaled before the ref is assumed to be released.
    ///
    /// The client must ensure that all pending refs on a buffer are released before attempting to
    /// reuse this buffer, otherwise synchronization errors may occur.
    ///
    /// This query is not valid for the [`OnCommit`] callback.
    ///
    #[cfg_attr(
        not(feature = "api-level-31"),
        doc = "[`OnCommit`]: https://developer.android.com/ndk/reference/group/native-activity#group___native_activity_1ga3465b02096890aba8011a71efe0241f2"
    )]
    #[doc(alias = "ASurfaceTransactionStats_getPreviousReleaseFenceFd")]
    pub fn previous_release_fence_fd(&self, surface_control: &SurfaceControl) -> Option<OwnedFd> {
        let fd = unsafe {
            ffi::ASurfaceTransactionStats_getPreviousReleaseFenceFd(
                self.0.ptr.as_ptr(),
                surface_control.ptr.as_ptr(),
            )
        };
        match fd {
            -1 => None,
            fd => Some(unsafe { OwnedFd::from_raw_fd(fd) }),
        }
    }
}

/// A list of [`SurfaceControl`]s returned by [`SurfaceTransactionStats::surface_controls()`].
#[derive(Debug)]
pub struct SurfaceControls {
    array: *mut *mut ffi::ASurfaceControl,
    count: usize,
}

impl AsRef<[SurfaceControl]> for SurfaceControls {
    fn as_ref(&self) -> &[SurfaceControl] {
        unsafe { std::slice::from_raw_parts(self.array.cast(), self.count) }
    }
}

impl AsMut<[SurfaceControl]> for SurfaceControls {
    fn as_mut(&mut self) -> &mut [SurfaceControl] {
        unsafe { std::slice::from_raw_parts_mut(self.array.cast(), self.count) }
    }
}

impl std::ops::Deref for SurfaceControls {
    type Target = [SurfaceControl];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl std::ops::DerefMut for SurfaceControls {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl Drop for SurfaceControls {
    /// Releases the array of [`SurfaceControl`]s that were returned by
    /// [`SurfaceTransactionStats::surface_controls()`].
    #[doc(alias = "ASurfaceTransactionStats_releaseASurfaceControls")]
    fn drop(&mut self) {
        unsafe { ffi::ASurfaceTransactionStats_releaseASurfaceControls(self.array) }
    }
}

/// Parameter for [`SurfaceTransaction::set_visibility()`].
#[repr(i8)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[doc(alias = "ASURFACE_TRANSACTION_VISIBILITY")]
#[non_exhaustive]
pub enum Visibility {
    #[doc(alias = "ASURFACE_TRANSACTION_VISIBILITY_HIDE")]
    Hide = ffi::ASurfaceTransactionVisibility::ASURFACE_TRANSACTION_VISIBILITY_HIDE.0,
    #[doc(alias = "ASURFACE_TRANSACTION_VISIBILITY_SHOW")]
    Show = ffi::ASurfaceTransactionVisibility::ASURFACE_TRANSACTION_VISIBILITY_SHOW.0,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(i8),
}

impl From<Visibility> for ffi::ASurfaceTransactionVisibility {
    fn from(val: Visibility) -> Self {
        Self(val.into())
    }
}

impl From<ffi::ASurfaceTransactionVisibility> for Visibility {
    fn from(val: ffi::ASurfaceTransactionVisibility) -> Self {
        Self::from(val.0)
    }
}

/// Parameter for [`SurfaceTransaction::set_buffer_transparency()`].
#[repr(i8)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[doc(alias = "ASURFACE_TRANSACTION_TRANSPARENCY")]
#[non_exhaustive]
pub enum Transparency {
    #[doc(alias = "ASURFACE_TRANSACTION_TRANSPARENCY_TRANSPARENT")]
    Transparent =
        ffi::ASurfaceTransactionTransparency::ASURFACE_TRANSACTION_TRANSPARENCY_TRANSPARENT.0,
    #[doc(alias = "ASURFACE_TRANSACTION_TRANSPARENCY_TRANSLUCENT")]
    Translucent =
        ffi::ASurfaceTransactionTransparency::ASURFACE_TRANSACTION_TRANSPARENCY_TRANSLUCENT.0,
    #[doc(alias = "ASURFACE_TRANSACTION_TRANSPARENCY_OPAQUE")]
    Opaque = ffi::ASurfaceTransactionTransparency::ASURFACE_TRANSACTION_TRANSPARENCY_OPAQUE.0,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(i8),
}

impl From<Transparency> for ffi::ASurfaceTransactionTransparency {
    fn from(val: Transparency) -> Self {
        Self(val.into())
    }
}

impl From<ffi::ASurfaceTransactionTransparency> for Transparency {
    fn from(val: ffi::ASurfaceTransactionTransparency) -> Self {
        Self::from(val.0)
    }
}
