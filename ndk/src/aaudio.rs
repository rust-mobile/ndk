//! Bindings for the NDK AAudio Android audio API.
//!
//! See also [the NDK docs](https://developer.android.com/ndk/guides/audio/aaudio/aaudio)
//! and [the NDK API reference](https://developer.android.com/ndk/reference/group/audio)
#![cfg(feature = "aaudio")]

use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{
    borrow::Cow,
    convert::TryFrom,
    ffi::{c_void, CStr},
    fmt,
    mem::MaybeUninit,
    num::NonZeroI32,
    ptr::NonNull,
};
use thiserror::Error;

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum AAudioAllowedCapturePolicy {
    AllowCaptureByAll = ffi::AAUDIO_ALLOW_CAPTURE_BY_ALL,
    AllowCaptureBySystem = ffi::AAUDIO_ALLOW_CAPTURE_BY_SYSTEM,
    AllowCaptureByNone = ffi::AAUDIO_ALLOW_CAPTURE_BY_NONE,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum AAudioContentType {
    Speech = ffi::AAUDIO_CONTENT_TYPE_SPEECH,
    Music = ffi::AAUDIO_CONTENT_TYPE_MUSIC,
    Movie = ffi::AAUDIO_CONTENT_TYPE_MOVIE,
    Sonification = ffi::AAUDIO_CONTENT_TYPE_SONIFICATION,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum AAudioDirection {
    Input = ffi::AAUDIO_DIRECTION_INPUT,
    Output = ffi::AAUDIO_DIRECTION_OUTPUT,
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[allow(non_camel_case_types)]
pub enum AAudioFormat {
    PCM_Float = ffi::AAUDIO_FORMAT_PCM_FLOAT,
    PCM_I16 = ffi::AAUDIO_FORMAT_PCM_I16,
    Invalid = ffi::AAUDIO_FORMAT_INVALID,
    Unspecified = ffi::AAUDIO_FORMAT_UNSPECIFIED,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum AAudioInputPreset {
    Generic = ffi::AAUDIO_INPUT_PRESET_GENERIC,
    Camcorder = ffi::AAUDIO_INPUT_PRESET_CAMCORDER,
    VoiceRecognition = ffi::AAUDIO_INPUT_PRESET_VOICE_RECOGNITION,
    VoiceCommunication = ffi::AAUDIO_INPUT_PRESET_VOICE_COMMUNICATION,
    Unprocessed = ffi::AAUDIO_INPUT_PRESET_UNPROCESSED,
    VoicePerformance = ffi::AAUDIO_INPUT_PRESET_VOICE_PERFORMANCE,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum AAudioPerformanceMode {
    None = ffi::AAUDIO_PERFORMANCE_MODE_NONE,
    PowerSaving = ffi::AAUDIO_PERFORMANCE_MODE_POWER_SAVING,
    LowLatency = ffi::AAUDIO_PERFORMANCE_MODE_LOW_LATENCY,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum AAudioSharingMode {
    Exclusive = ffi::AAUDIO_SHARING_MODE_EXCLUSIVE,
    Shared = ffi::AAUDIO_SHARING_MODE_SHARED,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum AAudioUsage {
    Media = ffi::AAUDIO_USAGE_MEDIA,
    VoiceCommunication = ffi::AAUDIO_USAGE_VOICE_COMMUNICATION,
    VoiceCommunicationSignalling = ffi::AAUDIO_USAGE_VOICE_COMMUNICATION_SIGNALLING,
    Alarm = ffi::AAUDIO_USAGE_ALARM,
    Notification = ffi::AAUDIO_USAGE_NOTIFICATION,
    NotificationRingtone = ffi::AAUDIO_USAGE_NOTIFICATION_RINGTONE,
    NotificationEvent = ffi::AAUDIO_USAGE_NOTIFICATION_EVENT,
    AssistanceAccessibility = ffi::AAUDIO_USAGE_ASSISTANCE_ACCESSIBILITY,
    AssistanceNavigationGuidance = ffi::AAUDIO_USAGE_ASSISTANCE_NAVIGATION_GUIDANCE,
    AssistanceSonification = ffi::AAUDIO_USAGE_ASSISTANCE_SONIFICATION,
    Game = ffi::AAUDIO_USAGE_GAME,
    Assistant = ffi::AAUDIO_USAGE_ASSISTANT,
    SystemEmergency = ffi::AAUDIO_SYSTEM_USAGE_EMERGENCY,
    SystemSafety = ffi::AAUDIO_SYSTEM_USAGE_SAFETY,
    SystemVehicleStatus = ffi::AAUDIO_SYSTEM_USAGE_VEHICLE_STATUS,
    SystemAnnouncement = ffi::AAUDIO_SYSTEM_USAGE_ANNOUNCEMENT,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum AAudioStreamState {
    Uninitialized = ffi::AAUDIO_STREAM_STATE_UNINITIALIZED,
    Unknown = ffi::AAUDIO_STREAM_STATE_UNKNOWN,
    Open = ffi::AAUDIO_STREAM_STATE_OPEN,
    Starting = ffi::AAUDIO_STREAM_STATE_STARTING,
    Started = ffi::AAUDIO_STREAM_STATE_STARTED,
    Pausing = ffi::AAUDIO_STREAM_STATE_PAUSING,
    Paused = ffi::AAUDIO_STREAM_STATE_PAUSED,
    Flushing = ffi::AAUDIO_STREAM_STATE_FLUSHING,
    Flushed = ffi::AAUDIO_STREAM_STATE_FLUSHED,
    Stopping = ffi::AAUDIO_STREAM_STATE_STOPPING,
    Stopped = ffi::AAUDIO_STREAM_STATE_STOPPED,
    Closing = ffi::AAUDIO_STREAM_STATE_CLOSING,
    Closed = ffi::AAUDIO_STREAM_STATE_CLOSED,
    Disconnected = ffi::AAUDIO_STREAM_STATE_DISCONNECTED,
}

impl AAudioStreamState {
    pub fn to_text(self) -> Cow<'static, str> {
        let ptr = unsafe {
            CStr::from_ptr(ffi::AAudio_convertStreamStateToText(
                self as ffi::aaudio_stream_state_t,
            ))
        };
        ptr.to_string_lossy()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SessionId {
    None,
    Allocated(NonZeroI32),
}

#[derive(Copy, Clone, Debug)]
pub struct Timestamp {
    pub frame_position: i64,
    pub time_nanoseconds: i64,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Clockid {
    Monotonic = ffi::CLOCK_MONOTONIC,
    Boottime = ffi::CLOCK_BOOTTIME,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AAudioCallbackResult {
    Continue = ffi::AAUDIO_CALLBACK_RESULT_CONTINUE,
    Stop = ffi::AAUDIO_CALLBACK_RESULT_STOP,
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AAudioErrorResult {
    Base = ffi::AAUDIO_ERROR_BASE,
    Disconnected = ffi::AAUDIO_ERROR_DISCONNECTED,
    IllegalArgument = ffi::AAUDIO_ERROR_ILLEGAL_ARGUMENT,
    Internal = ffi::AAUDIO_ERROR_INTERNAL,
    InvalidState = ffi::AAUDIO_ERROR_INVALID_STATE,
    InvalidHandle = ffi::AAUDIO_ERROR_INVALID_HANDLE,
    Unimplemented = ffi::AAUDIO_ERROR_UNIMPLEMENTED,
    Unavailable = ffi::AAUDIO_ERROR_UNAVAILABLE,
    NoFreeHandles = ffi::AAUDIO_ERROR_NO_FREE_HANDLES,
    NoMemory = ffi::AAUDIO_ERROR_NO_MEMORY,
    Null = ffi::AAUDIO_ERROR_NULL,
    Timeout = ffi::AAUDIO_ERROR_TIMEOUT,
    WouldBlock = ffi::AAUDIO_ERROR_WOULD_BLOCK,
    InvalidFormat = ffi::AAUDIO_ERROR_INVALID_FORMAT,
    OutOfRange = ffi::AAUDIO_ERROR_OUT_OF_RANGE,
    NoService = ffi::AAUDIO_ERROR_NO_SERVICE,
    InvalidRate = ffi::AAUDIO_ERROR_INVALID_RATE,
}

impl AAudioErrorResult {
    pub fn to_text(self) -> Cow<'static, str> {
        let ptr = unsafe {
            CStr::from_ptr(ffi::AAudio_convertStreamStateToText(
                self as ffi::aaudio_result_t,
            ))
        };
        ptr.to_string_lossy()
    }
}

#[derive(Debug, Error)]
pub enum AAudioError {
    #[error("error AAudio result ({0:?})")]
    ErrorResult(AAudioErrorResult),
    #[error("unknown AAudio error result ({0})")]
    UnknownResult(i32),
    #[error("unsupported AAudio result value received ({0})")]
    UnsupportedValue(i32),
}

impl AAudioError {
    pub(crate) fn from_result<T>(
        result: ffi::aaudio_result_t,
        on_success: impl FnOnce() -> T,
    ) -> Result<T> {
        use AAudioErrorResult::*;
        let result = match result {
            value if value >= 0 => return Ok(on_success()),
            ffi::AAUDIO_ERROR_BASE => Base,
            ffi::AAUDIO_ERROR_DISCONNECTED => Disconnected,
            ffi::AAUDIO_ERROR_ILLEGAL_ARGUMENT => IllegalArgument,
            ffi::AAUDIO_ERROR_INTERNAL => Internal,
            ffi::AAUDIO_ERROR_INVALID_STATE => InvalidState,
            ffi::AAUDIO_ERROR_INVALID_HANDLE => InvalidHandle,
            ffi::AAUDIO_ERROR_UNIMPLEMENTED => Unimplemented,
            ffi::AAUDIO_ERROR_UNAVAILABLE => Unavailable,
            ffi::AAUDIO_ERROR_NO_FREE_HANDLES => NoFreeHandles,
            ffi::AAUDIO_ERROR_NO_MEMORY => NoMemory,
            ffi::AAUDIO_ERROR_NULL => Null,
            ffi::AAUDIO_ERROR_TIMEOUT => Timeout,
            ffi::AAUDIO_ERROR_WOULD_BLOCK => WouldBlock,
            ffi::AAUDIO_ERROR_INVALID_FORMAT => InvalidFormat,
            ffi::AAUDIO_ERROR_OUT_OF_RANGE => OutOfRange,
            ffi::AAUDIO_ERROR_NO_SERVICE => NoService,
            ffi::AAUDIO_ERROR_INVALID_RATE => InvalidRate,
            _ => return Err(AAudioError::UnknownResult(result)),
        };
        Err(AAudioError::ErrorResult(result))
    }
}

pub type Result<T, E = AAudioError> = std::result::Result<T, E>;

fn construct<T>(with_ptr: impl FnOnce(*mut T) -> ffi::aaudio_result_t) -> Result<T> {
    let mut result = MaybeUninit::uninit();
    let status = with_ptr(result.as_mut_ptr());
    AAudioError::from_result(status, || unsafe { result.assume_init() })
}

fn enum_return_value<T: TryFrom<u32>>(return_value: i32) -> Result<T> {
    u32::try_from(return_value)
        .ok()
        .and_then(|value| T::try_from(value).ok())
        .ok_or_else(|| AAudioError::UnsupportedValue(return_value))
}

pub struct AAudioStreamBuilder {
    inner: NonNull<ffi::AAudioStreamBuilder>,
    data_callback: Option<AudioStreamDataCallback>,
    error_callback: Option<AudioStreamErrorCallback>,
}

impl fmt::Debug for AAudioStreamBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AAudioStreamBuilder")
            .field("inner", &self.inner)
            .field(
                "data_callback",
                match &self.data_callback {
                    Some(_) => &"Some(_)",
                    None => &"None",
                },
            )
            .field(
                "error_callback",
                match &self.error_callback {
                    Some(_) => &"Some(_)",
                    None => &"None",
                },
            )
            .finish()
    }
}

pub type AudioStreamDataCallback =
    Box<dyn FnMut(&AAudioStream, *mut c_void, i32) -> AAudioCallbackResult>;
pub type AudioStreamErrorCallback = Box<dyn FnMut(&AAudioStream, AAudioError)>;

impl AAudioStreamBuilder {
    fn from_ptr(inner: NonNull<ffi::AAudioStreamBuilder>) -> Self {
        Self {
            inner,
            data_callback: None,
            error_callback: None,
        }
    }

    fn as_ptr(&self) -> *mut ffi::AAudioStreamBuilder {
        self.inner.as_ptr()
    }

    pub fn new() -> Result<Self> {
        unsafe {
            let ptr = construct(|res| ffi::AAudio_createStreamBuilder(res))?;
            Ok(Self::from_ptr(NonNull::new_unchecked(ptr)))
        }
    }

    #[cfg(feature = "api-level-29")]
    pub fn allowed_capture_policy(self, capture_policy: AAudioAllowedCapturePolicy) -> Self {
        unsafe {
            ffi::AAudioStreamBuilder_setAllowedCapturePolicy(
                self.as_ptr(),
                capture_policy as ffi::aaudio_allowed_capture_policy_t,
            )
        };
        self
    }

    pub fn buffer_capacity_in_frames(self, num_frames: i32) -> Self {
        unsafe { ffi::AAudioStreamBuilder_setBufferCapacityInFrames(self.as_ptr(), num_frames) };
        self
    }

    pub fn channel_count(self, channel_count: i32) -> Self {
        unsafe { ffi::AAudioStreamBuilder_setChannelCount(self.as_ptr(), channel_count) };
        self
    }

    #[cfg(feature = "api-level-28")]
    pub fn content_type(self, content_type: AAudioContentType) -> Self {
        unsafe {
            ffi::AAudioStreamBuilder_setContentType(
                self.as_ptr(),
                content_type as ffi::aaudio_content_type_t,
            )
        };
        self
    }

    pub fn data_callback(mut self, callback: AudioStreamDataCallback) -> Self {
        let mut boxed = Box::new(callback);
        let ptr: *mut AudioStreamDataCallback = &mut *boxed;
        self.data_callback = Some(boxed);

        unsafe extern "C" fn ffi_callback(
            stream: *mut ffi::AAudioStreamStruct,
            user_data: *mut c_void,
            audio_data: *mut c_void,
            num_frames: i32,
        ) -> ffi::aaudio_data_callback_result_t {
            let callback = user_data as *mut AudioStreamDataCallback;
            let stream = AAudioStream {
                inner: NonNull::new_unchecked(stream),
                data_callback: None,
                error_callback: None,
            };
            let result = (*callback)(&stream, audio_data, num_frames);
            std::mem::forget(stream);
            result as ffi::aaudio_data_callback_result_t
        }

        unsafe {
            ffi::AAudioStreamBuilder_setDataCallback(
                self.as_ptr(),
                Some(ffi_callback),
                ptr as *mut c_void,
            )
        };

        self
    }

    pub fn device_id(self, device_id: i32) -> Self {
        unsafe { ffi::AAudioStreamBuilder_setDeviceId(self.as_ptr(), device_id) };
        self
    }

    pub fn direction(self, direction: AAudioDirection) -> Self {
        unsafe {
            ffi::AAudioStreamBuilder_setDirection(
                self.as_ptr(),
                direction as ffi::aaudio_direction_t,
            )
        };
        self
    }

    pub fn error_callback(mut self, callback: AudioStreamErrorCallback) -> Self {
        let mut boxed = Box::new(callback);
        let ptr: *mut AudioStreamErrorCallback = &mut *boxed;
        self.error_callback = Some(boxed);

        unsafe extern "C" fn ffi_callback(
            stream: *mut ffi::AAudioStreamStruct,
            user_data: *mut c_void,
            error: ffi::aaudio_result_t,
        ) {
            let callback = user_data as *mut AudioStreamErrorCallback;
            let stream = AAudioStream {
                inner: NonNull::new_unchecked(stream),
                data_callback: None,
                error_callback: None,
            };
            let err = AAudioError::from_result(error, || ()).unwrap_err();
            (*callback)(&stream, err);
            std::mem::forget(stream);
        }

        unsafe {
            ffi::AAudioStreamBuilder_setErrorCallback(
                self.as_ptr(),
                Some(ffi_callback),
                ptr as *mut c_void,
            )
        };

        self
    }

    pub fn format(self, format: AAudioFormat) -> Self {
        unsafe {
            ffi::AAudioStreamBuilder_setFormat(self.as_ptr(), format as ffi::aaudio_format_t)
        };
        self
    }

    pub fn frames_per_data_callback(self, num_frames: i32) -> Self {
        unsafe { ffi::AAudioStreamBuilder_setFramesPerDataCallback(self.as_ptr(), num_frames) };
        self
    }

    #[cfg(feature = "api-level-28")]
    pub fn input_preset(self, input_preset: AAudioInputPreset) -> Self {
        unsafe {
            ffi::AAudioStreamBuilder_setInputPreset(
                self.as_ptr(),
                input_preset as ffi::aaudio_input_preset_t,
            )
        };
        self
    }

    pub fn performance_mode(self, mode: AAudioPerformanceMode) -> Self {
        unsafe {
            ffi::AAudioStreamBuilder_setPerformanceMode(
                self.as_ptr(),
                mode as ffi::aaudio_performance_mode_t,
            )
        };
        self
    }

    pub fn sample_rate(self, sample_rate: i32) -> Self {
        unsafe { ffi::AAudioStreamBuilder_setSampleRate(self.as_ptr(), sample_rate) };
        self
    }

    pub fn samples_per_frame(self, samples_per_frame: i32) -> Self {
        unsafe { ffi::AAudioStreamBuilder_setSamplesPerFrame(self.as_ptr(), samples_per_frame) };
        self
    }

    /// If set to `Option::None` then a session ID will be allocated when the stream is opened.
    #[cfg(feature = "api-level-28")]
    pub fn session_id(self, session_id_or_allocate: Option<SessionId>) -> Self {
        let session_id = match session_id_or_allocate {
            None => ffi::AAUDIO_SESSION_ID_ALLOCATE,
            Some(SessionId::None) => ffi::AAUDIO_SESSION_ID_NONE,
            Some(SessionId::Allocated(value)) => value.get(),
        };

        unsafe { ffi::AAudioStreamBuilder_setSessionId(self.as_ptr(), session_id) };
        self
    }

    pub fn sharing_mode(self, sharing_mode: AAudioSharingMode) -> Self {
        unsafe {
            ffi::AAudioStreamBuilder_setSharingMode(
                self.as_ptr(),
                sharing_mode as ffi::aaudio_sharing_mode_t,
            )
        };
        self
    }

    #[cfg(feature = "api-level-28")]
    pub fn usage(self, usage: AAudioUsage) -> Self {
        unsafe { ffi::AAudioStreamBuilder_setUsage(self.as_ptr(), usage as ffi::aaudio_usage_t) };
        self
    }

    pub fn open_stream(mut self) -> Result<AAudioStream> {
        unsafe {
            let ptr = construct(|res| ffi::AAudioStreamBuilder_openStream(self.as_ptr(), res))?;

            Ok(AAudioStream {
                inner: NonNull::new_unchecked(ptr),
                data_callback: self.data_callback.take(),
                error_callback: self.error_callback.take(),
            })
        }
    }
}

impl Drop for AAudioStreamBuilder {
    fn drop(&mut self) {
        let status = unsafe { ffi::AAudioStreamBuilder_delete(self.as_ptr()) };
        AAudioError::from_result(status, || ()).unwrap();
    }
}

pub struct AAudioStream {
    inner: NonNull<ffi::AAudioStream>,
    data_callback: Option<AudioStreamDataCallback>,
    error_callback: Option<AudioStreamErrorCallback>,
}

impl fmt::Debug for AAudioStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AAudioStream")
            .field("inner", &self.inner)
            .field(
                "data_callback",
                match &self.data_callback {
                    Some(_) => &"Some(_)",
                    None => &"None",
                },
            )
            .field(
                "error_callback",
                match &self.error_callback {
                    Some(_) => &"Some(_)",
                    None => &"None",
                },
            )
            .finish()
    }
}

impl AAudioStream {
    fn as_ptr(&self) -> *mut ffi::AAudioStream {
        self.inner.as_ptr()
    }

    pub fn get_buffer_capacity_in_frames(&self) -> i32 {
        unsafe { ffi::AAudioStream_getBufferCapacityInFrames(self.as_ptr()) }
    }

    pub fn get_buffer_size_in_frames(&self) -> i32 {
        unsafe { ffi::AAudioStream_getBufferSizeInFrames(self.as_ptr()) }
    }

    pub fn get_channel_count(&self) -> i32 {
        unsafe { ffi::AAudioStream_getChannelCount(self.as_ptr()) }
    }

    #[cfg(feature = "api-level-28")]
    pub fn get_content_type(&self) -> Result<AAudioContentType> {
        let value = unsafe { ffi::AAudioStream_getContentType(self.as_ptr()) };
        enum_return_value(value)
    }

    pub fn get_device_id(&self) -> i32 {
        unsafe { ffi::AAudioStream_getDeviceId(self.as_ptr()) }
    }

    pub fn get_direction(&self) -> Result<AAudioDirection> {
        let value = unsafe { ffi::AAudioStream_getDirection(self.as_ptr()) };
        enum_return_value(value)
    }

    pub fn get_format(&self) -> Result<AAudioFormat> {
        let value = unsafe { ffi::AAudioStream_getFormat(self.as_ptr()) };
        AAudioFormat::try_from(value).map_err(|_| AAudioError::UnsupportedValue(value))
    }

    pub fn get_frames_per_burst(&self) -> i32 {
        unsafe { ffi::AAudioStream_getFramesPerBurst(self.as_ptr()) }
    }

    /// Query the size of the buffer that will be passed to the data callback in the `numFrames` parameter.
    /// `None` indicates that the callback buffer size for this stream may vary from one dataProc callback to the next.
    pub fn get_frames_per_data_callback(&self) -> Option<i32> {
        let value = unsafe { ffi::AAudioStream_getFramesPerDataCallback(self.as_ptr()) };
        const AAUDIO_UNSPECIFIED: i32 = ffi::AAUDIO_UNSPECIFIED as i32;
        match value {
            AAUDIO_UNSPECIFIED => None,
            val => Some(val),
        }
    }

    pub fn get_frames_read(&self) -> i64 {
        unsafe { ffi::AAudioStream_getFramesRead(self.as_ptr()) }
    }

    pub fn get_frames_written(&self) -> i64 {
        unsafe { ffi::AAudioStream_getFramesWritten(self.as_ptr()) }
    }

    #[cfg(feature = "api-level-28")]
    pub fn get_input_preset(&self) -> Result<AAudioInputPreset> {
        let value = unsafe { ffi::AAudioStream_getInputPreset(self.as_ptr()) };
        enum_return_value(value)
    }

    pub fn get_performance_mode(&self) -> Result<AAudioPerformanceMode> {
        let value = unsafe { ffi::AAudioStream_getPerformanceMode(self.as_ptr()) };
        enum_return_value(value)
    }

    pub fn get_sample_rate(&self) -> i32 {
        unsafe { ffi::AAudioStream_getSampleRate(self.as_ptr()) }
    }

    pub fn get_samples_per_frame(&self) -> i32 {
        unsafe { ffi::AAudioStream_getSamplesPerFrame(self.as_ptr()) }
    }

    #[cfg(feature = "api-level-28")]
    pub fn get_session_id(&self) -> SessionId {
        let value = unsafe { ffi::AAudioStream_getSessionId(self.as_ptr()) };
        match value {
            ffi::AAUDIO_SESSION_ID_NONE => SessionId::None,
            allocated => SessionId::Allocated(NonZeroI32::new(value).unwrap()),
        }
    }

    pub fn get_sharing_mode(&self) -> Result<AAudioSharingMode> {
        let value = unsafe { ffi::AAudioStream_getSharingMode(self.as_ptr()) };
        enum_return_value(value)
    }

    pub fn get_state(&self) -> Result<AAudioStreamState> {
        let value = unsafe { ffi::AAudioStream_getState(self.as_ptr()) };
        enum_return_value(value)
    }

    pub fn get_timestamp(&self, clockid: Clockid) -> Result<Timestamp> {
        let frame_position;
        let time_nanoseconds = unsafe {
            let mut nanoseconds = MaybeUninit::uninit();
            frame_position = construct(|ptr| {
                ffi::AAudioStream_getTimestamp(
                    self.as_ptr(),
                    clockid as ffi::clockid_t,
                    ptr,
                    nanoseconds.as_mut_ptr(),
                )
            })?;
            nanoseconds.assume_init()
        };

        Ok(Timestamp {
            frame_position,
            time_nanoseconds,
        })
    }

    #[cfg(feature = "api-level-28")]
    pub fn get_usage(&self) -> Result<AAudioUsage> {
        let value = unsafe { ffi::AAudioStream_getUsage(self.as_ptr()) };
        enum_return_value(value)
    }

    pub fn get_x_run_count(&self) -> i32 {
        unsafe { ffi::AAudioStream_getXRunCount(self.as_ptr()) }
    }

    pub unsafe fn read(
        &self,
        buffer: *mut c_void,
        num_frames: i32,
        timeout_nanoseconds: i64,
    ) -> Result<u32> {
        let result = ffi::AAudioStream_read(self.as_ptr(), buffer, num_frames, timeout_nanoseconds);

        AAudioError::from_result(result, || result as u32)
    }

    pub fn request_flush(&self) -> Result<()> {
        let result = unsafe { ffi::AAudioStream_requestFlush(self.as_ptr()) };
        AAudioError::from_result(result, || ())
    }

    pub fn request_pause(&self) -> Result<()> {
        let result = unsafe { ffi::AAudioStream_requestPause(self.as_ptr()) };
        AAudioError::from_result(result, || ())
    }

    pub fn request_start(&self) -> Result<()> {
        let result = unsafe { ffi::AAudioStream_requestStart(self.as_ptr()) };
        AAudioError::from_result(result, || ())
    }

    pub fn request_stop(&self) -> Result<()> {
        let result = unsafe { ffi::AAudioStream_requestStop(self.as_ptr()) };
        AAudioError::from_result(result, || ())
    }

    pub fn set_buffer_size_in_frames(&self, num_frames: i32) -> Result<i32> {
        let result = unsafe { ffi::AAudioStream_setBufferSizeInFrames(self.as_ptr(), num_frames) };
        AAudioError::from_result(result, || result)
    }

    pub fn wait_for_state_change(
        &self,
        input_state: AAudioStreamState,
        timeout_nanoseconds: i64,
    ) -> Result<AAudioStreamState> {
        let value = construct(|ptr| unsafe {
            ffi::AAudioStream_waitForStateChange(
                self.as_ptr(),
                input_state as ffi::aaudio_stream_state_t,
                ptr,
                timeout_nanoseconds,
            )
        })?;
        enum_return_value(value)
    }

    pub unsafe fn write(
        &self,
        buffer: *const c_void,
        num_frames: i32,
        timeout_nanoseconds: i64,
    ) -> Result<u32> {
        let result =
            ffi::AAudioStream_write(self.as_ptr(), buffer, num_frames, timeout_nanoseconds);

        AAudioError::from_result(result, || result as u32)
    }
}

impl Drop for AAudioStream {
    fn drop(&mut self) {
        let status = unsafe { ffi::AAudioStream_close(self.as_ptr()) };
        AAudioError::from_result(status, || ()).unwrap();
    }
}
