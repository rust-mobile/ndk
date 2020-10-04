#![cfg(feature = "camera")]

pub mod metadata;
mod tags;

use crate::native_window::NativeWindow;
use jni_sys::{jobject, JNIEnv};
use metadata::{ConstEntry, EntryType, FromEntryResult, ToEntryData};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{
    convert::TryInto,
    ffi::{CStr, CString},
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    os::raw::{c_char, c_int, c_void},
    ptr::NonNull,
};
pub use tags::*;
use thiserror::Error;

#[derive(Debug)]
pub struct CameraId(CString);

impl CameraId {
    pub fn new(id: CString) -> Self {
        Self(id)
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraErrorStatus {
    Unknown = ffi::camera_status_t_ACAMERA_ERROR_UNKNOWN,
    InvalidParameter = ffi::camera_status_t_ACAMERA_ERROR_INVALID_PARAMETER,
    CameraDisconnected = ffi::camera_status_t_ACAMERA_ERROR_CAMERA_DISCONNECTED,
    NotEnoughMemory = ffi::camera_status_t_ACAMERA_ERROR_NOT_ENOUGH_MEMORY,
    MetadataNotFound = ffi::camera_status_t_ACAMERA_ERROR_METADATA_NOT_FOUND,
    CameraDevice = ffi::camera_status_t_ACAMERA_ERROR_CAMERA_DEVICE,
    CameraService = ffi::camera_status_t_ACAMERA_ERROR_CAMERA_SERVICE,
    SessionClosed = ffi::camera_status_t_ACAMERA_ERROR_SESSION_CLOSED,
    InvalidOperation = ffi::camera_status_t_ACAMERA_ERROR_INVALID_OPERATION,
    StreamConfigureFail = ffi::camera_status_t_ACAMERA_ERROR_STREAM_CONFIGURE_FAIL,
    CameraInUse = ffi::camera_status_t_ACAMERA_ERROR_CAMERA_IN_USE,
    MaxCameraInUse = ffi::camera_status_t_ACAMERA_ERROR_MAX_CAMERA_IN_USE,
    CameraDisabled = ffi::camera_status_t_ACAMERA_ERROR_CAMERA_DISABLED,
    PermissionDenied = ffi::camera_status_t_ACAMERA_ERROR_PERMISSION_DENIED,
    UnsupportedOperation = ffi::camera_status_t_ACAMERA_ERROR_UNSUPPORTED_OPERATION,
}

#[derive(Debug, Error)]
pub enum CameraError {
    #[error("error camera result ({0:?})")]
    ErrorResult(CameraErrorStatus),
    #[error("unknown camera error result ({0})")]
    UnknownError(i32),
    #[error("unsupported camera enum value received ({0})")]
    UnsupportedEnumValue(i64),
    #[error("invalid metadata entry count ({0}), expected it to be {1}")]
    InvalidMetadataEntryCount(usize, &'static str),
}

impl CameraError {
    fn from_status<T>(status: ffi::camera_status_t, on_success: impl FnOnce() -> T) -> Result<T> {
        use CameraErrorStatus::*;
        let result = match status {
            value if value >= 0 => return Ok(on_success()),
            ffi::camera_status_t_ACAMERA_ERROR_UNKNOWN => Unknown,
            ffi::camera_status_t_ACAMERA_ERROR_INVALID_PARAMETER => InvalidParameter,
            ffi::camera_status_t_ACAMERA_ERROR_CAMERA_DISCONNECTED => CameraDisconnected,
            ffi::camera_status_t_ACAMERA_ERROR_NOT_ENOUGH_MEMORY => NotEnoughMemory,
            ffi::camera_status_t_ACAMERA_ERROR_METADATA_NOT_FOUND => MetadataNotFound,
            ffi::camera_status_t_ACAMERA_ERROR_CAMERA_DEVICE => CameraDevice,
            ffi::camera_status_t_ACAMERA_ERROR_CAMERA_SERVICE => CameraService,
            ffi::camera_status_t_ACAMERA_ERROR_SESSION_CLOSED => SessionClosed,
            ffi::camera_status_t_ACAMERA_ERROR_INVALID_OPERATION => InvalidOperation,
            ffi::camera_status_t_ACAMERA_ERROR_STREAM_CONFIGURE_FAIL => StreamConfigureFail,
            ffi::camera_status_t_ACAMERA_ERROR_CAMERA_IN_USE => CameraInUse,
            ffi::camera_status_t_ACAMERA_ERROR_MAX_CAMERA_IN_USE => MaxCameraInUse,
            ffi::camera_status_t_ACAMERA_ERROR_CAMERA_DISABLED => CameraDisabled,
            ffi::camera_status_t_ACAMERA_ERROR_PERMISSION_DENIED => PermissionDenied,
            ffi::camera_status_t_ACAMERA_ERROR_UNSUPPORTED_OPERATION => UnsupportedOperation,
            _ => return Err(CameraError::UnknownError(status)),
        };
        Err(CameraError::ErrorResult(result))
    }
}

pub type Result<T, E = CameraError> = std::result::Result<T, E>;

fn construct<T>(with_ptr: impl FnOnce(*mut T) -> ffi::camera_status_t) -> Result<T> {
    let mut result = MaybeUninit::uninit();
    let status = with_ptr(result.as_mut_ptr());
    CameraError::from_status(status, || unsafe { result.assume_init() })
}

fn construct_never_null<T>(
    with_ptr: impl FnOnce(*mut *mut T) -> ffi::camera_status_t,
) -> Result<NonNull<T>> {
    let result = construct(with_ptr)?;
    let non_null = if cfg!(debug_assertions) {
        NonNull::new(result).expect("result should never be null")
    } else {
        unsafe { NonNull::new_unchecked(result) }
    };
    Ok(non_null)
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum CameraDeviceError {
    CameraInUse = ffi::ERROR_CAMERA_IN_USE,
    MaxCamerasInUse = ffi::ERROR_MAX_CAMERAS_IN_USE,
    CameraDisabled = ffi::ERROR_CAMERA_DISABLED,
    CameraDevice = ffi::ERROR_CAMERA_DEVICE,
    CameraService = ffi::ERROR_CAMERA_SERVICE,
    /// unknown device error value returned from NDK
    UnrecognizedErrorValue,
}

pub trait CameraAvailabilityCallbacks: Send {
    fn on_camera_available(&self, camera_id: CameraId);
    fn on_camera_unavailable(&self, camera_id: CameraId);
}

pub trait CameraExtendedAvailabilityCallbacks: CameraAvailabilityCallbacks {
    fn on_camera_access_priorities_changed(&self);
}

pub trait CameraDeviceStateCallbacks: Send {
    fn on_disconnected(&self, device: &CameraDevice);
    fn on_error(&self, device: &CameraDevice, error: CameraDeviceError);
}

pub struct CameraManager {
    inner: NonNull<ffi::ACameraManager>,
    availability_callback: Option<Box<Box<dyn CameraAvailabilityCallbacks>>>,
    extended_availability_callback: Option<Box<Box<dyn CameraExtendedAvailabilityCallbacks>>>,
}

impl Debug for CameraManager {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("CameraManager")
            .field("inner", &self.inner)
            .field(
                "availability_callback",
                match &self.availability_callback {
                    Some(_) => &"Some(_)",
                    None => &"None",
                },
            )
            .field(
                "extended_availability_callback",
                match &self.extended_availability_callback {
                    Some(_) => &"Some(_)",
                    None => &"None",
                },
            )
            .finish()
    }
}

unsafe impl Send for CameraManager {}
unsafe impl Sync for CameraManager {}

impl Default for CameraManager {
    fn default() -> Self {
        Self {
            inner: unsafe {
                NonNull::new(ffi::ACameraManager_create()).expect("could not create camera manager")
            },
            availability_callback: None,
            extended_availability_callback: None,
        }
    }
}

/// Convert a callback pointer into the Rust wrapper type, which won't call drop()
macro_rules! cb_tmp {
    ($Type:ty, $var:expr) => {
        ::std::mem::ManuallyDrop::new(<$Type>::from_ptr(::std::ptr::NonNull::new_unchecked($var)));
    };
}

impl CameraManager {
    pub fn new() -> Self {
        <_>::default()
    }

    fn as_ptr(&self) -> *mut ffi::ACameraManager {
        self.inner.as_ptr()
    }

    pub fn iter_cameras(&self) -> Result<CameraIdList> {
        let camera_id_list = construct_never_null(|ptr| unsafe {
            ffi::ACameraManager_getCameraIdList(self.as_ptr(), ptr)
        })?;

        Ok(CameraIdList {
            camera_id_list,
            index: 0,
        })
    }

    pub fn get_metadata(&self, id: &CameraId) -> Result<CameraMetadata> {
        let metadata = construct_never_null(|ptr| unsafe {
            ffi::ACameraManager_getCameraCharacteristics(self.as_ptr(), id.0.as_ptr(), ptr)
        })?;

        Ok(CameraMetadata { inner: metadata })
    }

    fn availability_callback(
        callback: *mut Box<dyn CameraAvailabilityCallbacks>,
    ) -> ffi::ACameraManager_AvailabilityCallbacks {
        type Context = *mut Box<dyn CameraAvailabilityCallbacks>;

        unsafe extern "C" fn available(context: *mut c_void, camera_id: *const c_char) {
            let context = context as Context;
            let camera_id = CStr::from_ptr(camera_id as *mut _).to_owned();
            (*context).on_camera_available(CameraId::new(camera_id))
        }

        unsafe extern "C" fn unavailable(context: *mut c_void, camera_id: *const c_char) {
            let context = context as Context;
            let camera_id = CStr::from_ptr(camera_id as *mut _).to_owned();
            (*context).on_camera_unavailable(CameraId::new(camera_id))
        }

        ffi::ACameraManager_AvailabilityCallbacks {
            context: callback as *mut c_void,
            onCameraAvailable: Some(available),
            onCameraUnavailable: Some(unavailable),
        }
    }

    pub fn register_availability_callback(
        &mut self,
        callback: Box<dyn CameraAvailabilityCallbacks>,
    ) -> Result<()> {
        let mut availability_callback = Box::new(callback);

        let callback = Self::availability_callback(&mut *availability_callback);

        self.availability_callback = Some(availability_callback);
        let status =
            unsafe { ffi::ACameraManager_registerAvailabilityCallback(self.as_ptr(), &callback) };

        CameraError::from_status(status, || ())
    }

    pub fn unregister_availability_callback(&mut self) -> Result<()> {
        if let Some(callback) = self.availability_callback.as_mut() {
            let callback = Self::availability_callback(&mut **callback);
            let status = unsafe {
                ffi::ACameraManager_unregisterAvailabilityCallback(self.as_ptr(), &callback)
            };

            return CameraError::from_status(status, || ());
        }
        Ok(())
    }

    #[cfg(feature = "api-level-29")]
    fn extended_availability_callback(
        callback: *mut Box<dyn CameraExtendedAvailabilityCallbacks>,
    ) -> ffi::ACameraManager_ExtendedAvailabilityListener {
        type Context = *mut Box<dyn CameraExtendedAvailabilityCallbacks>;

        unsafe extern "C" fn available(context: *mut c_void, camera_id: *const c_char) {
            let context = context as Context;
            let camera_id = CStr::from_ptr(camera_id as *mut _).to_owned();
            (*context).on_camera_available(CameraId::new(camera_id))
        }

        unsafe extern "C" fn unavailable(context: *mut c_void, camera_id: *const c_char) {
            let context = context as Context;
            let camera_id = CStr::from_ptr(camera_id as *mut _).to_owned();
            (*context).on_camera_unavailable(CameraId::new(camera_id))
        }

        unsafe extern "C" fn access_priorities_changed(context: *mut c_void) {
            let context = context as Context;
            (*context).on_camera_access_priorities_changed()
        }

        ffi::ACameraManager_ExtendedAvailabilityListener {
            availabilityCallbacks: ffi::ACameraManager_AvailabilityCallbacks {
                context: callback as *mut c_void,
                onCameraAvailable: Some(available),
                onCameraUnavailable: Some(unavailable),
            },
            onCameraAccessPrioritiesChanged: Some(access_priorities_changed),
            onPhysicalCameraAvailable: None,   // Not yet stable
            onPhysicalCameraUnavailable: None, // Not yet stable
            reserved: [std::ptr::null_mut(); 4],
        }
    }

    #[cfg(feature = "api-level-29")]
    pub fn register_extended_availability_callback(
        &mut self,
        callback: Box<dyn CameraExtendedAvailabilityCallbacks>,
    ) -> Result<()> {
        let mut extended_availability_callback = Box::new(callback);

        let callback = Self::extended_availability_callback(&mut *extended_availability_callback);

        self.extended_availability_callback = Some(extended_availability_callback);
        let status = unsafe {
            ffi::ACameraManager_registerExtendedAvailabilityCallback(self.as_ptr(), &callback)
        };

        CameraError::from_status(status, || ())
    }

    #[cfg(feature = "api-level-29")]
    pub fn unregister_extended_availability_callback(&mut self) -> Result<()> {
        if let Some(callback) = self.extended_availability_callback.as_mut() {
            let callback = Self::extended_availability_callback(&mut **callback);
            let status = unsafe {
                ffi::ACameraManager_unregisterExtendedAvailabilityCallback(self.as_ptr(), &callback)
            };

            return CameraError::from_status(status, || ());
        }
        Ok(())
    }

    pub fn open_camera(
        &self,
        id: &CameraId,
        callbacks: Box<dyn CameraDeviceStateCallbacks>,
    ) -> Result<CameraDevice> {
        type Context = *mut Box<dyn CameraDeviceStateCallbacks>;

        unsafe extern "C" fn disconnected(context: *mut c_void, device: *mut ffi::ACameraDevice) {
            let context = context as Context;
            let device = ManuallyDrop::new(CameraDevice {
                inner: NonNull::new_unchecked(device),
                callbacks: None,
            });

            (*context).on_disconnected(&device);
        }

        unsafe extern "C" fn error(
            context: *mut c_void,
            device: *mut ffi::ACameraDevice,
            error: c_int,
        ) {
            let context = context as Context;
            let device = ManuallyDrop::new(CameraDevice {
                inner: NonNull::new_unchecked(device),
                callbacks: None,
            });
            let error: CameraDeviceError = (error as u32)
                .try_into()
                .unwrap_or(CameraDeviceError::UnrecognizedErrorValue);

            (*context).on_error(&device, error);
        }

        let mut boxed = Box::new(callbacks);
        let context: Context = &mut *boxed;
        let mut callbacks = ffi::ACameraDevice_StateCallbacks {
            context: context as *mut c_void,
            onDisconnected: Some(disconnected),
            onError: Some(error),
        };

        let inner = construct_never_null(|ptr| unsafe {
            ffi::ACameraManager_openCamera(self.as_ptr(), id.0.as_ptr(), &mut callbacks, ptr)
        })?;

        Ok(CameraDevice {
            inner,
            callbacks: Some(boxed),
        })
    }
}

impl std::ops::Drop for CameraManager {
    fn drop(&mut self) {
        let _ = self.unregister_availability_callback();
        #[cfg(feature = "api-level-29")]
        let _ = self.unregister_extended_availability_callback();
        unsafe { ffi::ACameraManager_delete(self.as_ptr()) }
    }
}

#[derive(Debug)]
pub struct CameraIdList {
    camera_id_list: NonNull<ffi::ACameraIdList>,
    index: i32,
}

impl Iterator for CameraIdList {
    type Item = CameraId;

    fn next(&mut self) -> Option<Self::Item> {
        let list = unsafe { self.camera_id_list.as_ref() };
        if self.index < list.numCameras {
            let id = unsafe {
                let id = *list.cameraIds.offset(self.index as isize);
                CStr::from_ptr(id as *mut _).to_owned()
            };
            self.index += 1;
            Some(CameraId(id))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, std::option::Option<usize>) {
        let total = unsafe { self.camera_id_list.as_ref() }.numCameras as usize;
        (total, Some(total))
    }
}

impl ExactSizeIterator for CameraIdList {
    fn len(&self) -> usize {
        let total = unsafe { self.camera_id_list.as_ref() }.numCameras;
        (total - self.index) as usize
    }
}

impl std::ops::Drop for CameraIdList {
    fn drop(&mut self) {
        unsafe { ffi::ACameraManager_deleteCameraIdList(self.camera_id_list.as_ptr()) }
    }
}

impl Display for CameraId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

#[derive(Debug)]
pub struct CameraMetadata {
    inner: NonNull<ffi::ACameraMetadata>,
}

impl Clone for CameraMetadata {
    fn clone(&self) -> Self {
        let copy = unsafe { ffi::ACameraMetadata_copy(self.as_ptr()) };
        Self {
            inner: NonNull::new(copy).expect("ACameraMetadata_copy returned null"),
        }
    }
}

impl CameraMetadata {
    pub fn from_ptr(ptr: NonNull<ffi::ACameraMetadata>) -> Self {
        Self { inner: ptr }
    }

    fn as_ptr(&self) -> *mut ffi::ACameraMetadata {
        self.inner.as_ptr()
    }

    /// Create a `CameraMetadata` from a JNI based `CameraCharacteristics` or `CaptureResult`
    pub fn from_jni(env: *mut JNIEnv, camera_metadata: jobject) -> Self {
        let ptr = unsafe {
            ffi::ACameraMetadata_fromCameraMetadata(
                env as *mut ffi::JNIEnv,
                camera_metadata as ffi::jobject,
            )
        };
        Self::from_ptr(
            NonNull::new(ptr).expect("camera_metadata is not a correct metadata class instance"),
        )
    }

    fn get_const_entry(&self, tag: MetadataTag) -> Result<ConstEntry<'_>> {
        let entry = construct(|ptr| unsafe {
            ffi::ACameraMetadata_getConstEntry(self.as_ptr(), tag.0, ptr)
        })?;
        Ok(unsafe { ConstEntry::new(entry) })
    }

    pub fn get<'a, T: FromEntryResult<'a>>(&'a self, tag: MetadataTag) -> Result<T> {
        let result = self.get_const_entry(tag);
        T::from_entry_result(result)
    }

    pub fn get_all_tags(&self) -> Result<&[MetadataTag]> {
        let mut num_entries = MaybeUninit::uninit();
        let tags = construct(|ptr| unsafe {
            ffi::ACameraMetadata_getAllTags(self.as_ptr(), num_entries.as_mut_ptr(), ptr)
        })?;
        Ok(unsafe {
            std::slice::from_raw_parts(
                tags as *const MetadataTag,
                num_entries.assume_init() as usize,
            )
        })
    }

    #[cfg(feature = "api-level-29")]
    pub fn is_logical_multi_camera(&self) -> Option<impl Iterator<Item = CameraId>> {
        let mut count = MaybeUninit::uninit();
        let mut ids = MaybeUninit::uninit();
        let is_multi = unsafe {
            ffi::ACameraMetadata_isLogicalMultiCamera(
                self.as_ptr(),
                count.as_mut_ptr(),
                ids.as_mut_ptr(),
            )
        };

        if !is_multi {
            return None;
        }
        unsafe {
            let count = count.assume_init();
            let ids = ids.assume_init();
            let mut i = 0;
            Some(std::iter::from_fn(move || {
                if i < count {
                    let res = CStr::from_ptr(*ids.offset(i as isize));
                    i += 1;
                    Some(CameraId(res.to_owned()))
                } else {
                    None
                }
            }))
        }
    }
}

impl std::ops::Drop for CameraMetadata {
    fn drop(&mut self) {
        unsafe { ffi::ACameraMetadata_free(self.as_ptr()) }
    }
}

pub struct CameraDevice {
    inner: NonNull<ffi::ACameraDevice>,
    callbacks: Option<Box<Box<dyn CameraDeviceStateCallbacks>>>,
}
unsafe impl Send for CameraDevice {}
unsafe impl Sync for CameraDevice {}

impl Debug for CameraDevice {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("CameraDevice")
            .field("inner", &self.inner)
            .field(
                "callbacks",
                match &self.callbacks {
                    Some(_) => &"Some(_)",
                    None => &"None",
                },
            )
            .finish()
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum RequestTemplate {
    Preview = ffi::ACameraDevice_request_template_TEMPLATE_PREVIEW,
    StilCapture = ffi::ACameraDevice_request_template_TEMPLATE_STILL_CAPTURE,
    Record = ffi::ACameraDevice_request_template_TEMPLATE_RECORD,
    VideoSnapshot = ffi::ACameraDevice_request_template_TEMPLATE_VIDEO_SNAPSHOT,
    ZeroShutterLag = ffi::ACameraDevice_request_template_TEMPLATE_ZERO_SHUTTER_LAG,
    Manual = ffi::ACameraDevice_request_template_TEMPLATE_MANUAL,
}

pub trait CaptureSessionStateCallbacks: Send {
    fn on_ready(&self, session: &CameraCaptureSession);
    fn on_closed(&self, session: &CameraCaptureSession);
    fn on_active(&self, session: &CameraCaptureSession);
}

impl CameraDevice {
    fn as_ptr(&self) -> *mut ffi::ACameraDevice {
        self.inner.as_ptr()
    }

    pub fn create_capture_request<UC>(
        &self,
        template_id: RequestTemplate,
    ) -> Result<CaptureRequest<UC>> {
        let inner = construct_never_null(|ptr| unsafe {
            ffi::ACameraDevice_createCaptureRequest(self.as_ptr(), template_id.into(), ptr)
        })?;
        Ok(CaptureRequest {
            inner,
            _pd: PhantomData,
        })
    }

    fn create_session_callbacks(
        callbacks: Box<dyn CaptureSessionStateCallbacks>,
    ) -> ffi::ACameraCaptureSession_stateCallbacks {
        type Context = *mut Box<dyn CaptureSessionStateCallbacks>;
        // freed when session is closed
        let context: Context = Box::into_raw(Box::new(callbacks));

        unsafe extern "C" fn closed(
            context: *mut c_void,
            session: *mut ffi::ACameraCaptureSession,
        ) {
            let context = Box::from_raw(context as Context);
            let session = cb_tmp!(CameraCaptureSession, session);

            context.on_closed(&session);
        }

        unsafe extern "C" fn ready(context: *mut c_void, session: *mut ffi::ACameraCaptureSession) {
            let context = context as Context;
            let session = cb_tmp!(CameraCaptureSession, session);

            (*context).on_ready(&session);
        }

        unsafe extern "C" fn active(
            context: *mut c_void,
            session: *mut ffi::ACameraCaptureSession,
        ) {
            let context = context as Context;
            let session = cb_tmp!(CameraCaptureSession, session);

            (*context).on_ready(&session);
        }

        ffi::ACameraCaptureSession_stateCallbacks {
            context: context as *mut c_void,
            onClosed: Some(closed),
            onReady: Some(ready),
            onActive: Some(active),
        }
    }

    pub fn create_capture_session(
        &self,
        outputs: &CaptureSessionOutputContainer,
        callbacks: Box<dyn CaptureSessionStateCallbacks>,
    ) -> Result<CameraCaptureSession> {
        let callbacks = Self::create_session_callbacks(callbacks);
        let inner = construct_never_null(|ptr| unsafe {
            ffi::ACameraDevice_createCaptureSession(
                self.as_ptr(),
                outputs.as_ptr(),
                &callbacks,
                ptr,
            )
        })?;

        Ok(CameraCaptureSession { inner })
    }
}

impl Drop for CameraDevice {
    fn drop(&mut self) {
        unsafe { ffi::ACameraDevice_close(self.as_ptr()) };
    }
}

#[derive(Debug)]
pub struct CaptureSessionOutputContainer {
    inner: NonNull<ffi::ACaptureSessionOutputContainer>,
}
unsafe impl Send for CaptureSessionOutputContainer {}

impl CaptureSessionOutputContainer {
    fn as_ptr(&self) -> *mut ffi::ACaptureSessionOutputContainer {
        self.inner.as_ptr()
    }

    pub fn new() -> Result<Self> {
        let inner =
            construct_never_null(|ptr| unsafe { ffi::ACaptureSessionOutputContainer_create(ptr) })?;
        Ok(Self { inner })
    }

    pub fn add(&self, output: &CaptureSessionOutput) -> Result<()> {
        let status =
            unsafe { ffi::ACaptureSessionOutputContainer_add(self.as_ptr(), output.as_ptr()) };
        CameraError::from_status(status, || ())
    }

    pub fn remove(&self, output: &CaptureSessionOutput) -> Result<()> {
        let status =
            unsafe { ffi::ACaptureSessionOutputContainer_remove(self.as_ptr(), output.as_ptr()) };
        CameraError::from_status(status, || ())
    }
}

impl Drop for CaptureSessionOutputContainer {
    fn drop(&mut self) {
        unsafe { ffi::ACaptureSessionOutputContainer_free(self.as_ptr()) };
    }
}

#[derive(Debug)]
pub struct CaptureSessionOutput {
    inner: NonNull<ffi::ACaptureSessionOutput>,
}

impl CaptureSessionOutput {
    fn as_ptr(&self) -> *mut ffi::ACaptureSessionOutput {
        self.inner.as_ptr()
    }

    pub fn new(anw: NativeWindow) -> Result<Self> {
        Ok(Self {
            inner: construct_never_null(|ptr| unsafe {
                ffi::ACaptureSessionOutput_create(anw.ptr().as_ptr(), ptr)
            })?,
        })
    }

    #[cfg(feature = "api-level-29")]
    pub fn create_physical(anw: NativeWindow, physical_id: &CameraId) -> Result<Self> {
        Ok(Self {
            inner: construct_never_null(|ptr| unsafe {
                ffi::ACaptureSessionPhysicalOutput_create(
                    anw.ptr().as_ptr(),
                    physical_id.0.as_ptr(),
                    ptr,
                )
            })?,
        })
    }

    #[cfg(feature = "api-level-28")]
    pub fn create_shared(anw: NativeWindow) -> Result<Self> {
        Ok(Self {
            inner: construct_never_null(|ptr| unsafe {
                ffi::ACaptureSessionSharedOutput_create(anw.ptr().as_ptr(), ptr)
            })?,
        })
    }

    #[cfg(feature = "api-level-28")]
    pub fn add_shared(&self, anw: NativeWindow) -> Result<()> {
        unsafe { ffi::ACaptureSessionSharedOutput_add(self.as_ptr(), anw.ptr().as_ptr()) }
    }

    #[cfg(feature = "api-level-28")]
    pub fn remove_shared(&self, anw: NativeWindow) -> Result<()> {
        unsafe { ffi::ACaptureSessionSharedOutput_remove(self.as_ptr(), anw.ptr().as_ptr()) }
    }
}

impl Drop for CaptureSessionOutput {
    fn drop(&mut self) {
        unsafe { ffi::ACaptureSessionOutput_free(self.as_ptr()) };
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CameraCaptureSession {
    inner: NonNull<ffi::ACameraCaptureSession>,
}
unsafe impl Send for CameraCaptureSession {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CaptureFailureError {
    Flushed = ffi::CAPTURE_FAILURE_REASON_FLUSHED as _,
    Error = ffi::CAPTURE_FAILURE_REASON_ERROR as _,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CameraCaptureFailure {
    pub frame_number: i64,
    pub reason: i32,
    pub sequence_id: i32,
    pub was_image_captured: bool,
}

impl CameraCaptureFailure {
    pub fn reason(&self) -> Result<CaptureFailureError, i32> {
        match self.reason.try_into() {
            Ok(ffi::CAPTURE_FAILURE_REASON_FLUSHED) => Ok(CaptureFailureError::Flushed),
            Ok(ffi::CAPTURE_FAILURE_REASON_ERROR) => Ok(CaptureFailureError::Error),
            _ => Err(self.reason),
        }
    }
}

pub trait CaptureCallbacks: Send {
    fn on_capture_started(
        &self,
        session: &CameraCaptureSession,
        request: &CaptureRequest,
        timestamp: i64,
    );

    fn on_capture_progressed(
        &self,
        session: &CameraCaptureSession,
        request: &CaptureRequest,
        result: &CameraMetadata,
    );

    fn on_capture_completed(
        &self,
        session: &CameraCaptureSession,
        request: &CaptureRequest,
        result: &CameraMetadata,
    );

    fn on_capture_failed(
        &self,
        session: &CameraCaptureSession,
        request: &CaptureRequest,
        failure: CameraCaptureFailure,
    );

    fn on_capture_sequence_completed(
        &self,
        session: &CameraCaptureSession,
        sequence_id: CaptureSequenceId,
        frame_number: i64,
    );

    fn on_capture_sequence_aborted(
        &self,
        session: &CameraCaptureSession,
        sequence_id: CaptureSequenceId,
    );

    fn on_capture_buffer_lost(
        &self,
        session: &CameraCaptureSession,
        request: &CaptureRequest,
        window: &NativeWindow,
        frame_number: i64,
    );
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CaptureSequenceId(pub i32);

impl CameraCaptureSession {
    fn from_ptr(ptr: NonNull<ffi::ACameraCaptureSession>) -> Self {
        Self { inner: ptr }
    }

    fn as_ptr(&self) -> *mut ffi::ACameraCaptureSession {
        self.inner.as_ptr()
    }

    fn make_callbacks(
        callbacks: Box<dyn CaptureCallbacks>,
    ) -> ffi::ACameraCaptureSession_captureCallbacks {
        type Context = *mut Box<dyn CaptureCallbacks>;
        // freed when sequence is completed or aborted
        let context: Context = Box::into_raw(Box::new(callbacks));

        unsafe extern "C" fn started(
            context: *mut c_void,
            session: *mut ffi::ACameraCaptureSession,
            request: *const ffi::ACaptureRequest,
            timestamp: i64,
        ) {
            let context = context as Context;
            let session = cb_tmp!(CameraCaptureSession, session);
            let request = cb_tmp!(CaptureRequest, request as *mut ffi::ACaptureRequest);

            (*context).on_capture_started(&session, &request, timestamp);
        }

        unsafe extern "C" fn progressed(
            context: *mut c_void,
            session: *mut ffi::ACameraCaptureSession,
            request: *mut ffi::ACaptureRequest,
            result: *const ffi::ACameraMetadata,
        ) {
            let context = context as Context;
            let session = cb_tmp!(CameraCaptureSession, session);
            let request = cb_tmp!(CaptureRequest, request);
            let result = cb_tmp!(CameraMetadata, result as *mut ffi::ACameraMetadata);

            (*context).on_capture_progressed(&session, &request, &result);
        }

        unsafe extern "C" fn completed(
            context: *mut c_void,
            session: *mut ffi::ACameraCaptureSession,
            request: *mut ffi::ACaptureRequest,
            result: *const ffi::ACameraMetadata,
        ) {
            let context = context as Context;
            let session = cb_tmp!(CameraCaptureSession, session);
            let request = cb_tmp!(CaptureRequest, request);
            let result = cb_tmp!(CameraMetadata, result as *mut ffi::ACameraMetadata);

            (*context).on_capture_completed(&session, &request, &result);
        }

        unsafe extern "C" fn failed(
            context: *mut c_void,
            session: *mut ffi::ACameraCaptureSession,
            request: *mut ffi::ACaptureRequest,
            failure: *mut ffi::ACameraCaptureFailure,
        ) {
            let context = context as Context;
            let session = cb_tmp!(CameraCaptureSession, session);
            let request = cb_tmp!(CaptureRequest, request);
            let failure = *failure;
            let failure = CameraCaptureFailure {
                frame_number: failure.frameNumber,
                reason: failure.reason,
                sequence_id: failure.sequenceId,
                was_image_captured: failure.wasImageCaptured,
            };

            (*context).on_capture_failed(&session, &request, failure);
        }

        unsafe extern "C" fn sequence_completed(
            context: *mut c_void,
            session: *mut ffi::ACameraCaptureSession,
            sequence_id: c_int,
            frame_number: i64,
        ) {
            let context = Box::from_raw(context as Context);
            let session = cb_tmp!(CameraCaptureSession, session);

            (*context).on_capture_sequence_completed(
                &session,
                CaptureSequenceId(sequence_id),
                frame_number,
            );
        }

        unsafe extern "C" fn sequence_aborted(
            context: *mut c_void,
            session: *mut ffi::ACameraCaptureSession,
            sequence_id: c_int,
        ) {
            let context = Box::from_raw(context as Context);
            let session = cb_tmp!(CameraCaptureSession, session);

            (*context).on_capture_sequence_aborted(&session, CaptureSequenceId(sequence_id));
        }

        unsafe extern "C" fn buffer_lost(
            context: *mut c_void,
            session: *mut ffi::ACameraCaptureSession,
            request: *mut ffi::ACaptureRequest,
            window: *mut ffi::ACameraWindowType,
            frame_number: i64,
        ) {
            let context = context as Context;
            let session = cb_tmp!(CameraCaptureSession, session);
            let request = cb_tmp!(CaptureRequest, request);
            let window = cb_tmp!(NativeWindow, window);

            (*context).on_capture_buffer_lost(&session, &request, &window, frame_number);
        }

        ffi::ACameraCaptureSession_captureCallbacks {
            context: context as *mut c_void,
            onCaptureStarted: Some(started),
            onCaptureProgressed: Some(progressed),
            onCaptureCompleted: Some(completed),
            onCaptureFailed: Some(failed),
            onCaptureSequenceCompleted: Some(sequence_completed),
            onCaptureSequenceAborted: Some(sequence_aborted),
            onCaptureBufferLost: Some(buffer_lost),
        }
    }

    pub fn capture(
        &self,
        callbacks: Box<dyn CaptureCallbacks>,
        requests: &[CaptureRequest],
    ) -> Result<CaptureSequenceId> {
        let req_ptr = requests.as_ptr() as *mut *mut ffi::ACaptureRequest;
        let total = requests.len() as i32;
        let mut callbacks = Self::make_callbacks(callbacks);
        let res = construct(|res_ptr| unsafe {
            ffi::ACameraCaptureSession_capture(
                self.as_ptr(),
                &mut callbacks,
                total,
                req_ptr,
                res_ptr,
            )
        })?;
        Ok(CaptureSequenceId(res))
    }

    pub fn set_repeating_request(
        &self,
        callbacks: Box<dyn CaptureCallbacks>,
        requests: &[CaptureRequest],
    ) -> Result<CaptureSequenceId> {
        let req_ptr = requests.as_ptr() as *mut *mut ffi::ACaptureRequest;
        let total = requests.len() as i32;
        let mut callbacks = Self::make_callbacks(callbacks);
        let res = construct(|res_ptr| unsafe {
            ffi::ACameraCaptureSession_setRepeatingRequest(
                self.as_ptr(),
                &mut callbacks,
                total,
                req_ptr,
                res_ptr,
            )
        })?;
        Ok(CaptureSequenceId(res))
    }

    pub fn stop_repeating(&self) -> Result<()> {
        let status = unsafe { ffi::ACameraCaptureSession_stopRepeating(self.as_ptr()) };
        CameraError::from_status(status, || ())
    }

    pub fn abort_captures(&self) -> Result<()> {
        let status = unsafe { ffi::ACameraCaptureSession_abortCaptures(self.as_ptr()) };
        CameraError::from_status(status, || ())
    }
}

impl Drop for CameraCaptureSession {
    fn drop(&mut self) {
        unsafe { ffi::ACameraCaptureSession_close(self.as_ptr()) };
    }
}

#[derive(Debug)]
pub struct CaptureRequest<UserContext = ()> {
    // assumed to contain ONLY the native pointer!
    inner: NonNull<ffi::ACaptureRequest>,
    _pd: PhantomData<UserContext>,
}
unsafe impl<UserContext: Send> Send for CaptureRequest<UserContext> {}

impl<UC> CaptureRequest<UC> {
    pub fn from_ptr(ptr: NonNull<ffi::ACaptureRequest>) -> Self {
        Self {
            inner: ptr,
            _pd: PhantomData,
        }
    }

    fn as_ptr(&self) -> *mut ffi::ACaptureRequest {
        self.inner.as_ptr()
    }

    fn get_const_entry(&self, tag: MetadataTag) -> Result<ConstEntry<'_>> {
        let entry = construct(|ptr| unsafe {
            ffi::ACaptureRequest_getConstEntry(self.as_ptr(), tag.0, ptr)
        })?;
        Ok(unsafe { ConstEntry::new(entry) })
    }

    pub fn get<'a, T: FromEntryResult<'a>>(&'a self, tag: MetadataTag) -> Result<T> {
        let result = self.get_const_entry(tag);
        T::from_entry_result(result)
    }

    #[cfg(feature = "api-level-28")]
    fn get_const_entry_physical_camera(
        &self,
        id: &CameraId,
        tag: MetadataTag,
    ) -> Result<ConstEntry<'_>> {
        let entry = construct(|ptr| unsafe {
            ffi::ACaptureRequest_getConstEntry_physicalCamera(
                self.as_ptr(),
                id.0.as_ptr(),
                tag.0,
                ptr,
            )
        })?;
        Ok(unsafe { ConstEntry::new(entry) })
    }

    #[cfg(feature = "api-level-28")]
    pub fn get_physical_camera_key<'a, T: FromEntryResult<'a>>(
        &'a self,
        id: &CameraId,
        tag: MetadataTag,
    ) -> Result<T> {
        let result = self.get_const_entry_physical_camera(id, tag);
        T::from_entry_result(result)
    }

    pub fn get_all_tags(&self) -> Result<&[MetadataTag]> {
        let mut num_entries = MaybeUninit::uninit();
        let tags = construct(|ptr| unsafe {
            ffi::ACaptureRequest_getAllTags(self.as_ptr(), num_entries.as_mut_ptr(), ptr)
        })?;
        Ok(unsafe {
            std::slice::from_raw_parts(
                tags as *const MetadataTag,
                num_entries.assume_init() as usize,
            )
        })
    }

    pub fn set<T: ToEntryData + ?Sized>(&self, tag: MetadataTag, value: &T) -> Result<()> {
        let mut status = 0;

        value.as_entry_data(|slice| {
            status = T::EntryType::set_entry(self.inner, tag.0, slice);
        });
        CameraError::from_status(status, || ())
    }

    #[cfg(feature = "api-level-28")]
    pub fn set_physical_camera_key<T: ToEntryData>(
        &self,
        tag: MetadataTag,
        value: &T,
        camera: CameraId,
    ) -> Result<()> {
        let slice = value.as_entry_data();
        let status = T::EntryType::set_entry_physical(self.inner, &camera.0, tag.0, slice);
        CameraError::from_status(status, || ())
    }

    pub fn add_target(&self, output: &CameraOutputTarget) -> Result<()> {
        let status = unsafe { ffi::ACaptureRequest_addTarget(self.as_ptr(), output.as_ptr()) };
        CameraError::from_status(status, || ())
    }

    pub fn remove_target(&self, output: &CameraOutputTarget) -> Result<()> {
        let status = unsafe { ffi::ACaptureRequest_removeTarget(self.as_ptr(), output.as_ptr()) };
        CameraError::from_status(status, || ())
    }

    #[cfg(feature = "api-level-28")]
    pub fn get_user_context(&self) -> Option<NonNull<UC>> {
        let mut result = MaybeUninit::uninit();
        let inner = unsafe {
            ffi::ACaptureRequest_getUserContext(self.as_ptr(), result.as_mut_ptr());
            result.assume_init()
        };
        NonNull::new(inner.cast())
    }

    #[cfg(feature = "api-level-28")]
    pub fn set_user_context(&self, context: NonNull<UC>) {
        unsafe {
            ffi::ACaptureRequest_setUserContext(self.as_ptr(), context.as_ptr().cast());
        }
    }
}

impl<UC> Drop for CaptureRequest<UC> {
    fn drop(&mut self) {
        unsafe { ffi::ACaptureRequest_free(self.as_ptr()) };
    }
}

#[cfg(feature = "api-level-28")]
impl Clone for CaptureRequest {
    fn clone(&self) -> Self {
        let copy = unsafe { ffi::ACaptureRequest_copy(self.as_ptr()) };
        Self {
            inner: NonNull::new(copy).expect("ACaptureRequest_copy returned null"),
            _pd: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct CameraOutputTarget {
    inner: NonNull<ffi::ACameraOutputTarget>,
}
unsafe impl Send for CameraOutputTarget {}

impl CameraOutputTarget {
    fn as_ptr(&self) -> *mut ffi::ACameraOutputTarget {
        self.inner.as_ptr()
    }

    pub fn new(anw: NativeWindow) -> Result<Self> {
        Ok(Self {
            inner: construct_never_null(|ptr| unsafe {
                ffi::ACameraOutputTarget_create(anw.ptr().as_ptr(), ptr)
            })?,
        })
    }
}

impl Drop for CameraOutputTarget {
    fn drop(&mut self) {
        unsafe { ffi::ACameraOutputTarget_free(self.as_ptr()) };
    }
}
