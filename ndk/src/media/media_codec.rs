use super::{construct, construct_never_null, NdkMediaError, Result};
use crate::native_window::NativeWindow;
use std::{
    convert::TryInto,
    ffi::{c_void, CStr, CString},
    fmt::Display,
    os::raw::c_char,
    ptr::{self, NonNull},
    slice,
    time::Duration,
};

#[derive(Debug)]
pub enum MediaCodecDirection {
    Decoder,
    Encoder,
}

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
    fn as_ptr(&self) -> *mut ffi::AMediaFormat {
        self.inner.as_ptr()
    }

    pub fn new() -> Self {
        Self {
            inner: unsafe { NonNull::new(ffi::AMediaFormat_new()).unwrap() },
        }
    }

    pub fn get_i32(&self, key: &str) -> Option<i32> {
        let name = CString::new(key).unwrap();
        let mut out = 0;
        unsafe { ffi::AMediaFormat_getInt32(self.as_ptr(), name.as_ptr(), &mut out as _) }
            .then(|| out)
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        let name = CString::new(key).unwrap();
        let mut out = 0;
        unsafe { ffi::AMediaFormat_getInt64(self.as_ptr(), name.as_ptr(), &mut out as _) }
            .then(|| out)
    }

    pub fn get_f32(&self, key: &str) -> Option<f32> {
        let name = CString::new(key).unwrap();
        let mut out = 0.0;
        unsafe { ffi::AMediaFormat_getFloat(self.as_ptr(), name.as_ptr(), &mut out as _) }
            .then(|| out)
    }

    pub fn get_usize(&self, key: &str) -> Option<usize> {
        let name = CString::new(key).unwrap();
        let mut out: ffi::size_t = 0;
        unsafe { ffi::AMediaFormat_getSize(self.as_ptr(), name.as_ptr(), &mut out as _) }
            .then(|| out as _)
    }

    pub fn get_buffer(&self, key: &str) -> Option<&[u8]> {
        let name = CString::new(key).unwrap();
        let mut out_buffer: *mut c_void = ptr::null_mut();
        let mut out_size: ffi::size_t = 0;
        unsafe {
            ffi::AMediaFormat_getBuffer(
                self.as_ptr(),
                name.as_ptr(),
                &mut out_buffer as _,
                &mut out_size as _,
            )
        }
        .then(|| unsafe { slice::from_raw_parts(out_buffer as _, out_size as _) })
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        let name = CString::new(key).unwrap();
        let mut out: *const c_char = ptr::null();
        unsafe { ffi::AMediaFormat_getString(self.as_ptr(), name.as_ptr(), &mut out as _) }
            .then(|| unsafe { CStr::from_ptr(out).to_str().unwrap() })
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
                value.as_ptr() as _,
                value.len() as _,
            )
        };
    }

    #[cfg(feature = "api-level-28")]
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        let name = CString::new(key).unwrap();
        let mut out = 0.0;
        unsafe { ffi::AMediaFormat_getDouble(self.as_ptr(), name.as_ptr(), &mut out as _) }
            .then(|| out)
    }

    /// Returns (left, top, right, bottom)
    #[cfg(feature = "api-level-28")]
    pub fn get_rect(&self, key: &str) -> Option<(i32, i32, i32, i32)> {
        let name = CString::new(key).unwrap();
        let mut left = 0;
        let mut top = 0;
        let mut right = 0;
        let mut bottom = 0;
        unsafe {
            ffi::AMediaFormat_getRect(
                self.as_ptr(),
                name.as_ptr(),
                &mut left as _,
                &mut top as _,
                &mut right as _,
                &mut bottom as _,
            )
        }
        .then(|| (left, top, right, bottom))
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
        unsafe { ffi::AMediaFormat_setSize(self.as_ptr(), name.as_ptr(), value as _) };
    }
}

impl Drop for MediaFormat {
    fn drop(&mut self) {
        let status = unsafe { ffi::AMediaFormat_delete(self.as_ptr()) };
        NdkMediaError::from_status(status, || ()).unwrap();
    }
}

#[derive(Debug)]
pub struct MediaCodec {
    inner: NonNull<ffi::AMediaCodec>,
}

impl MediaCodec {
    fn as_ptr(&self) -> *mut ffi::AMediaCodec {
        self.inner.as_ptr()
    }

    pub fn from_codec_name(name: &str) -> Option<Self> {
        let c_string = CString::new(name).unwrap();
        Some(Self {
            inner: unsafe { NonNull::new(ffi::AMediaCodec_createCodecByName(c_string.as_ptr()))? },
        })
    }

    pub fn from_decoder_type(mime_type: &str) -> Option<Self> {
        let c_string = CString::new(mime_type).unwrap();
        Some(Self {
            inner: unsafe {
                NonNull::new(ffi::AMediaCodec_createDecoderByType(c_string.as_ptr()))?
            },
        })
    }

    pub fn from_encoder_type(mime_type: &str) -> Option<Self> {
        let c_string = CString::new(mime_type).unwrap();
        Some(Self {
            inner: unsafe {
                NonNull::new(ffi::AMediaCodec_createEncoderByType(c_string.as_ptr()))?
            },
        })
    }

    pub fn configure(
        &self,
        format: &MediaFormat,
        surface: &NativeWindow,
        direction: MediaCodecDirection,
    ) -> Result<()> {
        let status = unsafe {
            ffi::AMediaCodec_configure(
                self.as_ptr(),
                format.as_ptr(),
                surface.ptr().as_ptr(),
                ptr::null_mut(),
                if matches!(direction, MediaCodecDirection::Encoder) {
                    1
                } else {
                    0
                },
            )
        };
        NdkMediaError::from_status(status, || ())
    }

    #[cfg(feature = "api-level-26")]
    pub fn create_input_surface(&self) -> Result<NativeWindow> {
        unsafe {
            let ptr = construct_never_null(|res| {
                ffi::AMediaCodec_createInputSurface(self.as_ptr(), res)
            })?;
            Ok(NativeWindow::from_ptr(ptr))
        }
    }

    #[cfg(feature = "api-level-26")]
    pub fn create_persistent_input_surface() -> Result<NativeWindow> {
        unsafe {
            let ptr =
                construct_never_null(|res| ffi::AMediaCodec_createPersistentInputSurface(res))?;
            Ok(NativeWindow::from_ptr(ptr))
        }
    }

    // Returns `None` if timeout is reached.
    pub fn dequeue_input_buffer(&self, timeout: Duration) -> Result<Option<InputBuffer>> {
        let result = unsafe {
            ffi::AMediaCodec_dequeueInputBuffer(
                self.as_ptr(),
                timeout
                    .as_micros()
                    .try_into()
                    .expect("Supplied timeout is too large"),
            )
        };

        if result == ffi::AMEDIACODEC_INFO_TRY_AGAIN_LATER as _ {
            Ok(None)
        } else if result >= 0 {
            Ok(Some(InputBuffer {
                codec: self,
                index: result as _,
            }))
        } else {
            NdkMediaError::from_status(result as _, || None)
        }
    }

    // Returns `None` if timeout is reached.
    pub fn dequeue_output_buffer(&self, timeout: Duration) -> Result<Option<OutputBuffer>> {
        let mut info: ffi::AMediaCodecBufferInfo = unsafe { std::mem::zeroed() };

        let result = unsafe {
            ffi::AMediaCodec_dequeueOutputBuffer(
                self.as_ptr(),
                &mut info as _,
                timeout
                    .as_micros()
                    .try_into()
                    .expect("Supplied timeout is too large"),
            )
        };

        if result == ffi::AMEDIACODEC_INFO_TRY_AGAIN_LATER as _ {
            Ok(None)
        } else if result >= 0 {
            Ok(Some(OutputBuffer {
                codec: self,
                index: result as _,
                info,
            }))
        } else {
            NdkMediaError::from_status(result as _, || None)
        }
    }

    pub fn flush(&self) -> Result<()> {
        let status = unsafe { ffi::AMediaCodec_flush(self.as_ptr()) };
        NdkMediaError::from_status(status, || ())
    }

    #[cfg(feature = "api-level-28")]
    pub fn get_input_format(&self) -> MediaFormat {
        unsafe {
            let inner = construct_never_null(|res| {
                *res = ffi::AMediaCodec_getInputFormat(self.as_ptr());
                0
            })
            .unwrap();
            MediaFormat { inner }
        }
    }

    pub fn get_output_format(&self) -> MediaFormat {
        unsafe {
            let inner = construct_never_null(|res| {
                *res = ffi::AMediaCodec_getOutputFormat(self.as_ptr());
                0
            })
            .unwrap();
            MediaFormat { inner }
        }
    }

    #[cfg(feature = "api-level-28")]
    pub fn get_name(&self) -> Result<String> {
        unsafe {
            let name_ptr =
                construct(|name: *mut *mut c_char| ffi::AMediaCodec_getName(self.as_ptr(), name))?;
            let name = CStr::from_ptr(name_ptr).to_str().unwrap().to_owned();
            ffi::AMediaCodec_releaseName(self.as_ptr(), name_ptr);

            Ok(name)
        }
    }

    pub fn queue_input_buffer(
        &self,
        buffer: InputBuffer,
        offset: usize,
        size: usize,
        time: u64,
        flags: u32,
    ) -> Result<()> {
        let status = unsafe {
            ffi::AMediaCodec_queueInputBuffer(
                self.as_ptr(),
                buffer.index as _,
                offset as _,
                size as _,
                time,
                flags,
            )
        };
        NdkMediaError::from_status(status, || ())
    }

    pub fn release_output_buffer(&self, buffer: OutputBuffer, render: bool) -> Result<()> {
        let status = unsafe {
            ffi::AMediaCodec_releaseOutputBuffer(self.as_ptr(), buffer.index as _, render)
        };
        NdkMediaError::from_status(status, || ())
    }

    pub fn release_output_buffer_at_time(
        &self,
        buffer: OutputBuffer,
        timestamp_ns: i64,
    ) -> Result<()> {
        let status = unsafe {
            ffi::AMediaCodec_releaseOutputBufferAtTime(
                self.as_ptr(),
                buffer.index as _,
                timestamp_ns,
            )
        };
        NdkMediaError::from_status(status, || ())
    }

    #[cfg(feature = "api-level-26")]
    pub fn set_input_surface(&self, surface: NativeWindow) -> Result<()> {
        let status =
            unsafe { ffi::AMediaCodec_setInputSurface(self.as_ptr(), surface.ptr().as_ptr()) };
        NdkMediaError::from_status(status, || ())
    }

    pub fn set_output_surface(&self, surface: NativeWindow) -> Result<()> {
        let status =
            unsafe { ffi::AMediaCodec_setOutputSurface(self.as_ptr(), surface.ptr().as_ptr()) };
        NdkMediaError::from_status(status, || ())
    }

    #[cfg(feature = "api-level-26")]
    pub fn set_parameters(&self, params: MediaFormat) -> Result<()> {
        let status = unsafe { ffi::AMediaCodec_setParameters(self.as_ptr(), params.as_ptr()) };
        NdkMediaError::from_status(status, || ())
    }

    #[cfg(feature = "api-level-26")]
    pub fn set_signal_end_of_input_stream(&self) -> Result<()> {
        let status = unsafe { ffi::AMediaCodec_signalEndOfInputStream(self.as_ptr()) };
        NdkMediaError::from_status(status, || ())
    }

    pub fn start(&self) -> Result<()> {
        let status = unsafe { ffi::AMediaCodec_start(self.as_ptr()) };
        NdkMediaError::from_status(status, || ())
    }

    pub fn stop(&self) -> Result<()> {
        let status = unsafe { ffi::AMediaCodec_stop(self.as_ptr()) };
        NdkMediaError::from_status(status, || ())
    }
}

impl Drop for MediaCodec {
    fn drop(&mut self) {
        let status = unsafe { ffi::AMediaCodec_delete(self.as_ptr()) };
        NdkMediaError::from_status(status, || ()).unwrap();
    }
}

#[derive(Debug)]
pub struct InputBuffer<'a> {
    codec: &'a MediaCodec,
    index: usize,
}

impl InputBuffer<'_> {
    pub fn get_mut(&mut self) -> &mut [u8] {
        unsafe {
            let mut out_size: ffi::size_t = 0;
            let buffer_ptr = ffi::AMediaCodec_getInputBuffer(
                self.codec.as_ptr(),
                self.index as _,
                &mut out_size as _,
            );
            slice::from_raw_parts_mut(buffer_ptr, out_size as _)
        }
    }
}

#[derive(Debug)]
pub struct OutputBuffer<'a> {
    codec: &'a MediaCodec,
    index: ffi::size_t,
    info: ffi::AMediaCodecBufferInfo,
}

impl OutputBuffer<'_> {
    pub fn get_buffer(&self) -> &[u8] {
        unsafe {
            let mut _out_size: ffi::size_t = 0;
            let buffer_ptr = ffi::AMediaCodec_getOutputBuffer(
                self.codec.as_ptr(),
                self.index as _,
                &mut _out_size as _,
            );
            slice::from_raw_parts(buffer_ptr.add(self.info.offset as _), self.info.size as _)
        }
    }

    #[cfg(feature = "api-level-28")]
    pub fn get_format(&self) -> MediaFormat {
        unsafe {
            let inner = construct_never_null(|res| {
                *res = ffi::AMediaCodec_getBufferFormat(self.codec.as_ptr(), self.index as _);
                0
            })
            .unwrap();
            MediaFormat { inner }
        }
    }

    pub fn flags(&self) -> u32 {
        self.info.flags
    }

    pub fn presentation_time_us(&self) -> i64 {
        self.info.presentationTimeUs
    }
}
