//! Bindings for [`AMediaCodec`]
//!
//! [`AMediaCodec`]: https://developer.android.com/ndk/reference/group/media#amediacodec

#[deprecated = "MediaFormat should be referenced directly from the media_format module"]
pub use super::media_format::MediaFormat;
use crate::media_error::{MediaError, Result};
use crate::native_window::NativeWindow;
use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    ptr::{self, NonNull},
    slice,
    time::Duration,
};

#[derive(Debug, PartialEq, Eq)]
pub enum MediaCodecDirection {
    Decoder,
    Encoder,
}

/// A native [`AMediaCodec *`]
///
/// [`AMediaCodec *`]: https://developer.android.com/ndk/reference/group/media#amediacodec
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
            inner: NonNull::new(unsafe { ffi::AMediaCodec_createCodecByName(c_string.as_ptr()) })?,
        })
    }

    pub fn from_decoder_type(mime_type: &str) -> Option<Self> {
        let c_string = CString::new(mime_type).unwrap();
        Some(Self {
            inner: NonNull::new(unsafe {
                ffi::AMediaCodec_createDecoderByType(c_string.as_ptr())
            })?,
        })
    }

    pub fn from_encoder_type(mime_type: &str) -> Option<Self> {
        let c_string = CString::new(mime_type).unwrap();
        Some(Self {
            inner: NonNull::new(unsafe {
                ffi::AMediaCodec_createEncoderByType(c_string.as_ptr())
            })?,
        })
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
                info: unsafe { info.assume_init() },
            }))
        }
    }

    pub fn flush(&self) -> Result<()> {
        let status = unsafe { ffi::AMediaCodec_flush(self.as_ptr()) };
        MediaError::from_status(status)
    }

    #[cfg(feature = "api-level-28")]
    pub fn input_format(&self) -> MediaFormat {
        let inner = NonNull::new(unsafe { ffi::AMediaCodec_getInputFormat(self.as_ptr()) })
            .expect("AMediaCodec_getInputFormat returned NULL");
        unsafe { MediaFormat::from_ptr(inner) }
    }

    pub fn output_format(&self) -> MediaFormat {
        let inner = NonNull::new(unsafe { ffi::AMediaCodec_getOutputFormat(self.as_ptr()) })
            .expect("AMediaCodec_getOutputFormat returned NULL");
        unsafe { MediaFormat::from_ptr(inner) }
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
        let status = unsafe {
            ffi::AMediaCodec_queueInputBuffer(
                self.as_ptr(),
                buffer.index,
                offset as ffi::off_t,
                size,
                time,
                flags,
            )
        };
        MediaError::from_status(status)
    }

    pub fn release_output_buffer(&self, buffer: OutputBuffer<'_>, render: bool) -> Result<()> {
        let status =
            unsafe { ffi::AMediaCodec_releaseOutputBuffer(self.as_ptr(), buffer.index, render) };
        MediaError::from_status(status)
    }

    pub fn release_output_buffer_at_time(
        &self,
        buffer: OutputBuffer<'_>,
        timestamp_ns: i64,
    ) -> Result<()> {
        let status = unsafe {
            ffi::AMediaCodec_releaseOutputBufferAtTime(self.as_ptr(), buffer.index, timestamp_ns)
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
        unsafe {
            let mut out_size = 0;
            let buffer_ptr =
                ffi::AMediaCodec_getInputBuffer(self.codec.as_ptr(), self.index, &mut out_size);
            assert!(
                !buffer_ptr.is_null(),
                "AMediaCodec_getInputBuffer returned NULL for index {}",
                self.index
            );
            slice::from_raw_parts_mut(buffer_ptr.cast(), out_size)
        }
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
    info: ffi::AMediaCodecBufferInfo,
}

impl OutputBuffer<'_> {
    pub fn buffer(&self) -> &[u8] {
        unsafe {
            let mut _out_size = 0;
            let buffer_ptr =
                ffi::AMediaCodec_getOutputBuffer(self.codec.as_ptr(), self.index, &mut _out_size);
            assert!(
                !buffer_ptr.is_null(),
                "AMediaCodec_getOutputBuffer returned NULL for index {}",
                self.index
            );
            slice::from_raw_parts(
                buffer_ptr.add(self.info.offset as usize),
                self.info.size as usize,
            )
        }
    }

    #[cfg(feature = "api-level-28")]
    pub fn format(&self) -> MediaFormat {
        let inner = NonNull::new(unsafe {
            ffi::AMediaCodec_getBufferFormat(self.codec.as_ptr(), self.index)
        })
        .expect("AMediaCodec_getBufferFormat returned NULL");
        unsafe { MediaFormat::from_ptr(inner) }
    }

    pub fn flags(&self) -> u32 {
        self.info.flags
    }

    pub fn presentation_time_us(&self) -> i64 {
        self.info.presentationTimeUs
    }
}

#[derive(Debug)]
pub enum DequeuedOutputBufferInfoResult<'a> {
    Buffer(OutputBuffer<'a>),
    TryAgainLater,
    OutputFormatChanged,
    OutputBuffersChanged,
}
