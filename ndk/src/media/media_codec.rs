//! Bindings for [`AMediaFormat`] and [`AMediaCodec`]
//!
//! [`AMediaFormat`]: https://developer.android.com/ndk/reference/group/media#amediaformat
//! [`AMediaCodec`]: https://developer.android.com/ndk/reference/group/media#amediacodec

use crate::media_error::{MediaError, Result};
use crate::native_window::NativeWindow;
use crate::utils::abort_on_panic;
use std::{
    ffi::{c_char, c_void, CStr, CString},
    fmt::Display,
    mem::MaybeUninit,
    pin::Pin,
    ptr::{self, NonNull},
    slice,
    time::Duration,
};

#[derive(Debug, PartialEq, Eq)]
pub enum MediaCodecDirection {
    Decoder,
    Encoder,
}

/// A native [`AMediaFormat *`]
///
/// [`AMediaFormat *`]: https://developer.android.com/ndk/reference/group/media#amediaformat
#[derive(Debug)]
pub struct MediaFormat {
    inner: NonNull<ffi::AMediaFormat>,
}

impl Display for MediaFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c_str = unsafe { CStr::from_ptr(ffi::AMediaFormat_toString(self.as_ptr())) };
        f.write_str(c_str.to_str().unwrap())
    }
}

impl Default for MediaFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaFormat {
    /// Assumes ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::AMediaFormat`].
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AMediaFormat>) -> Self {
        Self { inner: ptr }
    }

    fn as_ptr(&self) -> *mut ffi::AMediaFormat {
        self.inner.as_ptr()
    }

    pub fn new() -> Self {
        Self {
            inner: NonNull::new(unsafe { ffi::AMediaFormat_new() }).unwrap(),
        }
    }

    pub fn i32(&self, key: &str) -> Option<i32> {
        let name = CString::new(key).unwrap();
        let mut out = 0;
        if unsafe { ffi::AMediaFormat_getInt32(self.as_ptr(), name.as_ptr(), &mut out) } {
            Some(out)
        } else {
            None
        }
    }

    pub fn i64(&self, key: &str) -> Option<i64> {
        let name = CString::new(key).unwrap();
        let mut out = 0;
        if unsafe { ffi::AMediaFormat_getInt64(self.as_ptr(), name.as_ptr(), &mut out) } {
            Some(out)
        } else {
            None
        }
    }

    pub fn f32(&self, key: &str) -> Option<f32> {
        let name = CString::new(key).unwrap();
        let mut out = 0.0;
        if unsafe { ffi::AMediaFormat_getFloat(self.as_ptr(), name.as_ptr(), &mut out) } {
            Some(out)
        } else {
            None
        }
    }

    pub fn usize(&self, key: &str) -> Option<usize> {
        let name = CString::new(key).unwrap();
        let mut out = 0;
        if unsafe { ffi::AMediaFormat_getSize(self.as_ptr(), name.as_ptr(), &mut out) } {
            Some(out)
        } else {
            None
        }
    }

    pub fn buffer(&self, key: &str) -> Option<&[u8]> {
        let name = CString::new(key).unwrap();
        let mut out_buffer = ptr::null_mut();
        let mut out_size = 0;
        unsafe {
            ffi::AMediaFormat_getBuffer(
                self.as_ptr(),
                name.as_ptr(),
                &mut out_buffer,
                &mut out_size,
            )
        }
        .then(|| unsafe { slice::from_raw_parts(out_buffer.cast(), out_size) })
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        let name = CString::new(key).unwrap();
        let mut out = ptr::null();
        unsafe { ffi::AMediaFormat_getString(self.as_ptr(), name.as_ptr(), &mut out) }
            .then(|| unsafe { CStr::from_ptr(out) }.to_str().unwrap())
    }

    pub fn set_i32(&self, key: &str, value: i32) {
        let name = CString::new(key).unwrap();
        unsafe { ffi::AMediaFormat_setInt32(self.as_ptr(), name.as_ptr(), value) };
    }

    pub fn set_i64(&self, key: &str, value: i64) {
        let name = CString::new(key).unwrap();
        unsafe { ffi::AMediaFormat_setInt64(self.as_ptr(), name.as_ptr(), value) };
    }

    pub fn set_f32(&self, key: &str, value: f32) {
        let name = CString::new(key).unwrap();
        unsafe { ffi::AMediaFormat_setFloat(self.as_ptr(), name.as_ptr(), value) };
    }

    pub fn set_str(&self, key: &str, value: &str) {
        let name = CString::new(key).unwrap();
        let c_string = CString::new(value).unwrap();
        unsafe { ffi::AMediaFormat_setString(self.as_ptr(), name.as_ptr(), c_string.as_ptr()) };
    }

    pub fn set_buffer(&self, key: &str, value: &[u8]) {
        let name = CString::new(key).unwrap();
        unsafe {
            ffi::AMediaFormat_setBuffer(
                self.as_ptr(),
                name.as_ptr(),
                value.as_ptr().cast(),
                value.len(),
            )
        };
    }

    #[cfg(feature = "api-level-28")]
    pub fn f64(&self, key: &str) -> Option<f64> {
        let name = CString::new(key).unwrap();
        let mut out = 0.0;
        if unsafe { ffi::AMediaFormat_getDouble(self.as_ptr(), name.as_ptr(), &mut out) } {
            Some(out)
        } else {
            None
        }
    }

    /// Returns (left, top, right, bottom)
    #[cfg(feature = "api-level-28")]
    pub fn rect(&self, key: &str) -> Option<(i32, i32, i32, i32)> {
        let name = CString::new(key).unwrap();
        let mut left = 0;
        let mut top = 0;
        let mut right = 0;
        let mut bottom = 0;
        if unsafe {
            ffi::AMediaFormat_getRect(
                self.as_ptr(),
                name.as_ptr(),
                &mut left,
                &mut top,
                &mut right,
                &mut bottom,
            )
        } {
            Some((left, top, right, bottom))
        } else {
            None
        }
    }

    #[cfg(feature = "api-level-28")]
    pub fn set_f64(&self, key: &str, value: f64) {
        let name = CString::new(key).unwrap();
        unsafe { ffi::AMediaFormat_setDouble(self.as_ptr(), name.as_ptr(), value) };
    }

    #[cfg(feature = "api-level-28")]
    pub fn set_rect(&self, key: &str, left: i32, top: i32, right: i32, bottom: i32) {
        let name = CString::new(key).unwrap();
        unsafe {
            ffi::AMediaFormat_setRect(self.as_ptr(), name.as_ptr(), left, top, right, bottom)
        };
    }

    #[cfg(feature = "api-level-28")]
    pub fn set_usize(&self, key: &str, value: usize) {
        let name = CString::new(key).unwrap();
        unsafe { ffi::AMediaFormat_setSize(self.as_ptr(), name.as_ptr(), value) };
    }
}

impl Drop for MediaFormat {
    fn drop(&mut self) {
        let status = unsafe { ffi::AMediaFormat_delete(self.as_ptr()) };
        MediaError::from_status(status).unwrap();
    }
}

/// A native [`AMediaCodec *`]
///
/// [`AMediaCodec *`]: https://developer.android.com/ndk/reference/group/media#amediacodec
#[derive(Debug)]
pub struct MediaCodec {
    inner: NonNull<ffi::AMediaCodec>,
    async_notify_callback: Option<Pin<Box<AsyncNotifyCallback>>>,
}

pub struct AsyncNotifyCallback {
    /// Called when an input buffer becomes available.
    ///
    /// The specified index is the index of the available input buffer.
    pub on_input_available: Option<InputAvailableCallback>,

    /// Called when an output buffer becomes available.
    ///
    /// The specified index is the index of the available output buffer. The specified
    /// [`BufferInfo`] contains information regarding the available output buffer.
    pub on_output_available: Option<OutputAvailableCallback>,

    /// Called when the output format has changed.
    ///
    /// The specified format contains the new output format.
    pub on_format_changed: Option<FormatChangedCallback>,

    /// Called when the [`MediaCodec`] encountered an error.
    ///
    /// The specified [`ActionCode`] indicates the possible actions that client can take, and it can
    /// be checked by calling [`ActionCode::is_recoverable`] or [`ActionCode::is_transient`]. If
    /// both [`ActionCode::is_recoverable`] and [`ActionCode::is_transient`] return [`false`], then
    /// the codec error is fatal and the codec must be deleted. The specified detail string may
    /// contain more detailed messages about this error.
    pub on_error: Option<ErrorCallback>,
}

impl std::fmt::Debug for AsyncNotifyCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncNotifyCallback")
            .field(
                "on_input_available",
                match &self.on_input_available {
                    Some(_) => &"Some(_)",
                    None => &"None",
                },
            )
            .field(
                "on_output_available",
                match &self.on_output_available {
                    Some(_) => &"Some(_)",
                    None => &"None",
                },
            )
            .field(
                "on_format_changed",
                match &self.on_format_changed {
                    Some(_) => &"Some(_)",
                    None => &"None",
                },
            )
            .field(
                "on_error",
                match &self.on_error {
                    Some(_) => &"Some(_)",
                    None => &"None",
                },
            )
            .finish()
    }
}

pub type InputAvailableCallback = Box<dyn FnMut(usize)>;
pub type OutputAvailableCallback = Box<dyn FnMut(usize, &BufferInfo)>;
pub type FormatChangedCallback = Box<dyn FnMut(&MediaFormat)>;
pub type ErrorCallback = Box<dyn FnMut(MediaError, ActionCode, &CStr)>;

impl MediaCodec {
    fn as_ptr(&self) -> *mut ffi::AMediaCodec {
        self.inner.as_ptr()
    }

    pub fn from_codec_name(name: &str) -> Option<Self> {
        let c_string = CString::new(name).unwrap();
        Some(Self {
            inner: NonNull::new(unsafe { ffi::AMediaCodec_createCodecByName(c_string.as_ptr()) })?,
            async_notify_callback: None,
        })
    }

    pub fn from_decoder_type(mime_type: &str) -> Option<Self> {
        let c_string = CString::new(mime_type).unwrap();
        Some(Self {
            inner: NonNull::new(unsafe {
                ffi::AMediaCodec_createDecoderByType(c_string.as_ptr())
            })?,
            async_notify_callback: None,
        })
    }

    pub fn from_encoder_type(mime_type: &str) -> Option<Self> {
        let c_string = CString::new(mime_type).unwrap();
        Some(Self {
            inner: NonNull::new(unsafe {
                ffi::AMediaCodec_createEncoderByType(c_string.as_ptr())
            })?,
            async_notify_callback: None,
        })
    }

    /// Set an asynchronous callback for actionable [`MediaCodec`] events.
    ///
    /// When asynchronous callback is enabled, it is an error for the client to call
    /// [`MediaCodec::dequeue_input_buffer()`] or [`MediaCodec::dequeue_output_buffer()`].
    ///
    /// [`MediaCodec::flush()`] behaves differently in asynchronous mode. After calling
    /// [`MediaCodec::flush()`], the client must call [`MediaCodec::start()`] to "resume" receiving
    /// input buffers. Even if the client does not receive
    /// [`AsyncNotifyCallback::on_input_available`] callbacks from video encoders configured with an
    /// input surface, the client still needs to call [`MediaCodec::start()`] to resume the input
    /// surface to send buffers to the encoders.
    ///
    /// When called with [`None`] callback, this method unregisters any previously set callback.
    ///
    /// Refer to the definition of [`AsyncNotifyCallback`] on how each callback function is called
    /// and what are specified.
    ///
    /// Once the callback is unregistered or the codec is reset / released, the previously
    /// registered callback will not be called.
    ///
    /// All callbacks are fired on one NDK internal thread.
    /// [`MediaCodec::set_async_notify_callback()`] should not be called on the callback thread. No
    /// heavy duty task should be performed on callback thread.
    #[cfg(feature = "api-level-28")]
    pub fn set_async_notify_callback(
        &mut self,
        callback: Option<AsyncNotifyCallback>,
    ) -> Result<()> {
        unsafe extern "C" fn ffi_on_input_available(
            _codec: *mut ffi::AMediaCodec,
            user_data: *mut c_void,
            index: i32,
        ) {
            abort_on_panic(|| {
                let callback = &mut *(user_data as *mut AsyncNotifyCallback);
                if let Some(f) = callback.on_input_available.as_mut() {
                    f(index as usize);
                }
            })
        }

        unsafe extern "C" fn ffi_on_output_available(
            _codec: *mut ffi::AMediaCodec,
            user_data: *mut c_void,
            index: i32,
            buffer_info: *mut ffi::AMediaCodecBufferInfo,
        ) {
            abort_on_panic(|| {
                let callback = &mut *(user_data as *mut AsyncNotifyCallback);
                if let Some(f) = callback.on_output_available.as_mut() {
                    let buffer_info = BufferInfo {
                        inner: *buffer_info,
                    };
                    f(index as usize, &buffer_info);
                }
            })
        }

        unsafe extern "C" fn ffi_on_format_changed(
            _codec: *mut ffi::AMediaCodec,
            user_data: *mut c_void,
            format: *mut ffi::AMediaFormat,
        ) {
            abort_on_panic(|| {
                // Ownership of the format is not documented, but the implementation allocates a new instance and does
                // not free it, so assume it is ok for us to do so
                // https://cs.android.com/android/platform/superproject/main/+/refs/heads/main:frameworks/av/media/ndk/NdkMediaCodec.cpp;l=248-254;drc=5e15c3e22f3fa32d64e57302201123ce41589adf
                let format = MediaFormat::from_ptr(NonNull::new_unchecked(format));

                let callback = &mut *(user_data as *mut AsyncNotifyCallback);
                if let Some(f) = callback.on_format_changed.as_mut() {
                    f(&format);
                }
            })
        }

        unsafe extern "C" fn ffi_on_error(
            _codec: *mut ffi::AMediaCodec,
            user_data: *mut c_void,
            error: ffi::media_status_t,
            action_code: i32,
            detail: *const c_char,
        ) {
            abort_on_panic(|| {
                let callback = &mut *(user_data as *mut AsyncNotifyCallback);
                if let Some(f) = callback.on_error.as_mut() {
                    f(
                        MediaError::from_status(error).unwrap_err(),
                        ActionCode(action_code),
                        CStr::from_ptr(detail),
                    );
                }
            })
        }

        let (callback, ffi_callback, user_data) = if let Some(callback) = callback {
            // On Android 12 and earlier, due to faulty null checks, if a callback is not set, but at least one other
            // callback *is* set, then it will segfault in when trying to invoke the unset callback. See for example:
            // https://cs.android.com/android/platform/superproject/+/android-12.0.0_r34:frameworks/av/media/ndk/NdkMediaCodec.cpp;l=161-162;drc=ef058464777739e2d9ffad5f00d0e57b186d9a13
            // To work around this we just enable all callbacks and do nothing if the corresponding callback is not set
            // in AsyncNotifyCallback
            let ffi_callback = ffi::AMediaCodecOnAsyncNotifyCallback {
                onAsyncInputAvailable: Some(ffi_on_input_available),
                onAsyncOutputAvailable: Some(ffi_on_output_available),
                onAsyncFormatChanged: Some(ffi_on_format_changed),
                onAsyncError: Some(ffi_on_error),
            };

            let mut boxed = Box::pin(callback);
            let ptr: *mut AsyncNotifyCallback = &mut *boxed;

            (Some(boxed), ffi_callback, ptr as *mut c_void)
        } else {
            let ffi_callback = ffi::AMediaCodecOnAsyncNotifyCallback {
                onAsyncInputAvailable: None,
                onAsyncOutputAvailable: None,
                onAsyncFormatChanged: None,
                onAsyncError: None,
            };

            (None, ffi_callback, ptr::null_mut())
        };

        let status = unsafe {
            ffi::AMediaCodec_setAsyncNotifyCallback(self.as_ptr(), ffi_callback, user_data)
        };
        let result = MediaError::from_status(status);

        // This behavior is not documented, but the implementation always clears the callback on failure, so we must
        // clear any callback that may have been previously registered
        // https://cs.android.com/android/platform/superproject/main/+/main:frameworks/av/media/ndk/NdkMediaCodec.cpp;l=581-584;drc=8c4e619c7461ac1a8c20c55364643662e9185e4d
        if result.is_ok() {
            self.async_notify_callback = callback;
        } else {
            self.async_notify_callback = None;
        }

        result
    }

    pub fn configure(
        &self,
        format: &MediaFormat,
        surface: Option<&NativeWindow>,
        direction: MediaCodecDirection,
    ) -> Result<()> {
        let status = unsafe {
            ffi::AMediaCodec_configure(
                self.as_ptr(),
                format.as_ptr(),
                surface.map_or(ptr::null_mut(), |s| s.ptr().as_ptr()),
                ptr::null_mut(),
                if direction == MediaCodecDirection::Encoder {
                    ffi::AMEDIACODEC_CONFIGURE_FLAG_ENCODE as u32
                } else {
                    0
                },
            )
        };
        MediaError::from_status(status)
    }

    #[cfg(feature = "api-level-26")]
    pub fn create_input_surface(&self) -> Result<NativeWindow> {
        use crate::media_error::construct_never_null;
        unsafe {
            let ptr = construct_never_null(|res| {
                ffi::AMediaCodec_createInputSurface(self.as_ptr(), res)
            })?;
            Ok(NativeWindow::from_ptr(ptr))
        }
    }

    #[cfg(feature = "api-level-26")]
    pub fn create_persistent_input_surface() -> Result<NativeWindow> {
        use crate::media_error::construct_never_null;
        unsafe {
            let ptr =
                construct_never_null(|res| ffi::AMediaCodec_createPersistentInputSurface(res))?;
            Ok(NativeWindow::from_ptr(ptr))
        }
    }

    pub fn dequeue_input_buffer(&self, timeout: Duration) -> Result<DequeuedInputBufferResult<'_>> {
        let result = unsafe {
            ffi::AMediaCodec_dequeueInputBuffer(
                self.as_ptr(),
                timeout
                    .as_micros()
                    .try_into()
                    .expect("Supplied timeout is too large"),
            )
        };

        if result == ffi::AMEDIACODEC_INFO_TRY_AGAIN_LATER as isize {
            Ok(DequeuedInputBufferResult::TryAgainLater)
        } else {
            let index = MediaError::from_status_if_negative(result)? as usize;
            Ok(DequeuedInputBufferResult::Buffer(InputBuffer {
                codec: self,
                index,
            }))
        }
    }

    pub fn dequeue_output_buffer(
        &self,
        timeout: Duration,
    ) -> Result<DequeuedOutputBufferInfoResult<'_>> {
        let mut info = MaybeUninit::uninit();

        let result = unsafe {
            ffi::AMediaCodec_dequeueOutputBuffer(
                self.as_ptr(),
                info.as_mut_ptr(),
                timeout
                    .as_micros()
                    .try_into()
                    .expect("Supplied timeout is too large"),
            )
        };

        if result == ffi::AMEDIACODEC_INFO_TRY_AGAIN_LATER as isize {
            Ok(DequeuedOutputBufferInfoResult::TryAgainLater)
        } else if result == ffi::AMEDIACODEC_INFO_OUTPUT_FORMAT_CHANGED as isize {
            Ok(DequeuedOutputBufferInfoResult::OutputFormatChanged)
        } else if result == ffi::AMEDIACODEC_INFO_OUTPUT_BUFFERS_CHANGED as isize {
            Ok(DequeuedOutputBufferInfoResult::OutputBuffersChanged)
        } else {
            let index = MediaError::from_status_if_negative(result)? as usize;
            Ok(DequeuedOutputBufferInfoResult::Buffer(OutputBuffer {
                codec: self,
                index,
                info: BufferInfo {
                    inner: unsafe { info.assume_init() },
                },
            }))
        }
    }

    pub fn flush(&self) -> Result<()> {
        let status = unsafe { ffi::AMediaCodec_flush(self.as_ptr()) };
        MediaError::from_status(status)
    }

    pub fn input_buffer(&self, index: usize) -> Option<&mut [MaybeUninit<u8>]> {
        unsafe {
            let mut out_size = 0;
            let buffer_ptr = ffi::AMediaCodec_getInputBuffer(self.as_ptr(), index, &mut out_size);
            if buffer_ptr.is_null() {
                return None;
            }
            Some(slice::from_raw_parts_mut(buffer_ptr.cast(), out_size))
        }
    }

    pub fn output_buffer(&self, index: usize) -> Option<&[u8]> {
        unsafe {
            let mut out_size = 0;
            let buffer_ptr = ffi::AMediaCodec_getOutputBuffer(self.as_ptr(), index, &mut out_size);
            if buffer_ptr.is_null() {
                return None;
            }
            Some(slice::from_raw_parts(buffer_ptr, out_size))
        }
    }

    #[cfg(feature = "api-level-28")]
    pub fn input_format(&self) -> MediaFormat {
        let inner = NonNull::new(unsafe { ffi::AMediaCodec_getInputFormat(self.as_ptr()) })
            .expect("AMediaCodec_getInputFormat returned NULL");
        MediaFormat { inner }
    }

    pub fn output_format(&self) -> MediaFormat {
        let inner = NonNull::new(unsafe { ffi::AMediaCodec_getOutputFormat(self.as_ptr()) })
            .expect("AMediaCodec_getOutputFormat returned NULL");
        MediaFormat { inner }
    }

    #[cfg(feature = "api-level-28")]
    pub fn name(&self) -> Result<String> {
        use crate::media_error::construct;
        unsafe {
            let name_ptr = construct(|name| ffi::AMediaCodec_getName(self.as_ptr(), name))?;
            let name = CStr::from_ptr(name_ptr).to_str().unwrap().to_owned();
            ffi::AMediaCodec_releaseName(self.as_ptr(), name_ptr);

            Ok(name)
        }
    }

    pub fn queue_input_buffer(
        &self,
        buffer: InputBuffer<'_>,
        offset: usize,
        size: usize,
        time: u64,
        flags: u32,
    ) -> Result<()> {
        debug_assert!(ptr::eq(self, buffer.codec));
        self.queue_input_buffer_by_index(buffer.index, offset, size, time, flags)
    }

    pub fn queue_input_buffer_by_index(
        &self,
        buffer_index: usize,
        offset: usize,
        size: usize,
        time: u64,
        flags: u32,
    ) -> Result<()> {
        let status = unsafe {
            ffi::AMediaCodec_queueInputBuffer(
                self.as_ptr(),
                buffer_index,
                offset as ffi::off_t,
                size,
                time,
                flags,
            )
        };
        MediaError::from_status(status)
    }

    pub fn release_output_buffer(&self, buffer: OutputBuffer<'_>, render: bool) -> Result<()> {
        debug_assert!(ptr::eq(self, buffer.codec));
        self.release_output_buffer_by_index(buffer.index, render)
    }

    pub fn release_output_buffer_by_index(&self, buffer_index: usize, render: bool) -> Result<()> {
        let status =
            unsafe { ffi::AMediaCodec_releaseOutputBuffer(self.as_ptr(), buffer_index, render) };
        MediaError::from_status(status)
    }

    pub fn release_output_buffer_at_time(
        &self,
        buffer: OutputBuffer<'_>,
        timestamp_ns: i64,
    ) -> Result<()> {
        debug_assert!(ptr::eq(self, buffer.codec));
        self.release_output_buffer_at_time_by_index(buffer.index, timestamp_ns)
    }

    pub fn release_output_buffer_at_time_by_index(
        &self,
        buffer_index: usize,
        timestamp_ns: i64,
    ) -> Result<()> {
        let status = unsafe {
            ffi::AMediaCodec_releaseOutputBufferAtTime(self.as_ptr(), buffer_index, timestamp_ns)
        };
        MediaError::from_status(status)
    }

    #[cfg(feature = "api-level-26")]
    pub fn set_input_surface(&self, surface: &NativeWindow) -> Result<()> {
        let status =
            unsafe { ffi::AMediaCodec_setInputSurface(self.as_ptr(), surface.ptr().as_ptr()) };
        MediaError::from_status(status)
    }

    pub fn set_output_surface(&self, surface: &NativeWindow) -> Result<()> {
        let status =
            unsafe { ffi::AMediaCodec_setOutputSurface(self.as_ptr(), surface.ptr().as_ptr()) };
        MediaError::from_status(status)
    }

    #[cfg(feature = "api-level-26")]
    pub fn set_parameters(&self, params: MediaFormat) -> Result<()> {
        let status = unsafe { ffi::AMediaCodec_setParameters(self.as_ptr(), params.as_ptr()) };
        MediaError::from_status(status)
    }

    #[cfg(feature = "api-level-26")]
    pub fn set_signal_end_of_input_stream(&self) -> Result<()> {
        let status = unsafe { ffi::AMediaCodec_signalEndOfInputStream(self.as_ptr()) };
        MediaError::from_status(status)
    }

    pub fn start(&self) -> Result<()> {
        let status = unsafe { ffi::AMediaCodec_start(self.as_ptr()) };
        MediaError::from_status(status)
    }

    pub fn stop(&self) -> Result<()> {
        let status = unsafe { ffi::AMediaCodec_stop(self.as_ptr()) };
        MediaError::from_status(status)
    }
}

impl Drop for MediaCodec {
    fn drop(&mut self) {
        let status = unsafe { ffi::AMediaCodec_delete(self.as_ptr()) };
        MediaError::from_status(status).unwrap();
    }
}

#[derive(Debug)]
pub struct InputBuffer<'a> {
    codec: &'a MediaCodec,
    index: usize,
}

impl InputBuffer<'_> {
    pub fn buffer_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.codec.input_buffer(self.index).unwrap_or_else(|| {
            panic!(
                "AMediaCodec_getInputBuffer returned NULL for index {}",
                self.index
            )
        })
    }
}

#[derive(Debug)]
pub enum DequeuedInputBufferResult<'a> {
    Buffer(InputBuffer<'a>),
    TryAgainLater,
}

#[derive(Debug)]
pub struct OutputBuffer<'a> {
    codec: &'a MediaCodec,
    index: usize,
    info: BufferInfo,
}

impl OutputBuffer<'_> {
    pub fn buffer(&self) -> &[u8] {
        self.codec.output_buffer(self.index).unwrap_or_else(|| {
            panic!(
                "AMediaCodec_getOutputBuffer returned NULL for index {}",
                self.index
            )
        })
    }

    #[cfg(feature = "api-level-28")]
    pub fn format(&self) -> MediaFormat {
        let inner = NonNull::new(unsafe {
            ffi::AMediaCodec_getBufferFormat(self.codec.as_ptr(), self.index)
        })
        .expect("AMediaCodec_getBufferFormat returned NULL");
        MediaFormat { inner }
    }

    pub fn info(&self) -> &BufferInfo {
        &self.info
    }
}

#[derive(Debug)]
pub enum DequeuedOutputBufferInfoResult<'a> {
    Buffer(OutputBuffer<'a>),
    TryAgainLater,
    OutputFormatChanged,
    OutputBuffersChanged,
}

#[derive(Copy, Clone, Debug)]
pub struct BufferInfo {
    inner: ffi::AMediaCodecBufferInfo,
}

impl BufferInfo {
    pub fn offset(&self) -> i32 {
        self.inner.offset
    }

    pub fn size(&self) -> i32 {
        self.inner.size
    }

    pub fn presentation_time_us(&self) -> i64 {
        self.inner.presentationTimeUs
    }

    pub fn flags(&self) -> u32 {
        self.inner.flags
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ActionCode(pub i32);

impl ActionCode {
    pub fn is_recoverable(self) -> bool {
        unsafe { ffi::AMediaCodecActionCode_isRecoverable(self.0) }
    }

    pub fn is_transient(self) -> bool {
        unsafe { ffi::AMediaCodecActionCode_isTransient(self.0) }
    }
}
