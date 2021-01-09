#![cfg(feature = "api-level-24")]

use super::NdkMediaError;
use super::{construct, construct_never_null, error::MediaErrorResult, Result};
use crate::native_window::NativeWindow;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{
    convert::TryInto,
    ffi::c_void,
    fmt::{self, Debug, Formatter},
    mem::MaybeUninit,
    os::unix::io::RawFd,
    ptr::NonNull,
};

#[cfg(feature = "hardware_buffer")]
use crate::hardware_buffer::{HardwareBuffer, HardwareBufferUsage};

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[allow(non_camel_case_types)]
pub enum ImageFormat {
    RGBA_8888 = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_RGBA_8888,
    RGBX_8888 = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_RGBX_8888,
    RGB_888 = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_RGB_888,
    RGB_565 = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_RGB_565,
    RGBA_FP16 = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_RGBA_FP16,
    YUV_420_888 = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_YUV_420_888,
    JPEG = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_JPEG,
    RAW16 = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_RAW16,
    RAW_PRIVATE = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_RAW_PRIVATE,
    RAW10 = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_RAW10,
    RAW12 = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_RAW12,
    DEPTH16 = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_DEPTH16,
    DEPTH_POINT_CLOUD = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_DEPTH_POINT_CLOUD,
    PRIVATE = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_PRIVATE,
    Y8 = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_Y8,
    HEIC = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_HEIC,
    DEPTH_JPEG = ffi::AIMAGE_FORMATS_AIMAGE_FORMAT_DEPTH_JPEG,
}

pub type ImageListener = Box<dyn FnMut(&ImageReader)>;

#[cfg(feature = "hardware_buffer")]
pub type BufferRemovedListener = Box<dyn FnMut(&ImageReader, &HardwareBuffer)>;

pub struct ImageReader {
    inner: NonNull<ffi::AImageReader>,
    image_cb: Option<Box<ImageListener>>,
    #[cfg(feature = "hardware_buffer")]
    buffer_removed_cb: Option<Box<BufferRemovedListener>>,
}

impl Debug for ImageReader {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("ImageReader")
            .field("inner", &self.inner)
            .field(
                "image_cb",
                match &self.image_cb {
                    Some(_) => &"Some(_)",
                    None => &"None",
                },
            )
            .finish()
    }
}

impl ImageReader {
    fn from_ptr(inner: NonNull<ffi::AImageReader>) -> Self {
        Self {
            inner,
            image_cb: None,
            #[cfg(feature = "hardware_buffer")]
            buffer_removed_cb: None,
        }
    }

    fn as_ptr(&self) -> *mut ffi::AImageReader {
        self.inner.as_ptr()
    }

    pub fn new(width: i32, height: i32, format: ImageFormat, max_images: i32) -> Result<Self> {
        let inner = construct_never_null(|res| unsafe {
            ffi::AImageReader_new(width, height, format as i32, max_images, res)
        })?;

        Ok(Self::from_ptr(inner))
    }

    #[cfg(feature = "hardware_buffer")]
    pub fn new_with_usage(
        width: i32,
        height: i32,
        format: ImageFormat,
        usage: HardwareBufferUsage,
        max_images: i32,
    ) -> Result<Self> {
        let inner = construct_never_null(|res| unsafe {
            ffi::AImageReader_newWithUsage(width, height, format as i32, usage.0, max_images, res)
        })?;

        Ok(Self::from_ptr(inner))
    }

    pub fn set_image_listener(&mut self, listener: ImageListener) -> Result<()> {
        let mut boxed = Box::new(listener);
        let ptr: *mut ImageListener = &mut *boxed;
        // keep listener alive until Drop or new listener is assigned
        self.image_cb = Some(boxed);

        unsafe extern "C" fn on_image_available(
            context: *mut c_void,
            reader: *mut ffi::AImageReader,
        ) {
            let reader = ImageReader::from_ptr(NonNull::new_unchecked(reader));
            let listener: *mut ImageListener = context as *mut _;
            (*listener)(&reader);
            std::mem::forget(reader);
        }

        let mut listener = ffi::AImageReader_ImageListener {
            context: ptr as _,
            onImageAvailable: Some(on_image_available),
        };
        let status = unsafe { ffi::AImageReader_setImageListener(self.as_ptr(), &mut listener) };
        NdkMediaError::from_status(status, || ())
    }

    #[cfg(feature = "hardware_buffer")]
    pub fn set_buffer_removed_listener(&mut self, listener: BufferRemovedListener) -> Result<()> {
        let mut boxed = Box::new(listener);
        let ptr: *mut BufferRemovedListener = &mut *boxed;
        // keep listener alive until Drop or new listener is assigned
        self.buffer_removed_cb = Some(boxed);

        unsafe extern "C" fn on_buffer_removed(
            context: *mut c_void,
            reader: *mut ffi::AImageReader,
            buffer: *mut ffi::AHardwareBuffer,
        ) {
            let reader = ImageReader::from_ptr(NonNull::new_unchecked(reader));
            let buffer = HardwareBuffer::from_ptr(NonNull::new_unchecked(buffer));
            let listener: *mut BufferRemovedListener = context as *mut _;
            (*listener)(&reader, &buffer);
            std::mem::forget(reader);
        }

        let mut listener = ffi::AImageReader_BufferRemovedListener {
            context: ptr as _,
            onBufferRemoved: Some(on_buffer_removed),
        };
        let status =
            unsafe { ffi::AImageReader_setBufferRemovedListener(self.as_ptr(), &mut listener) };
        NdkMediaError::from_status(status, || ())
    }

    pub fn get_window(&self) -> Result<NativeWindow> {
        unsafe {
            let ptr = construct_never_null(|res| ffi::AImageReader_getWindow(self.as_ptr(), res))?;
            Ok(NativeWindow::from_ptr(ptr))
        }
    }

    pub fn get_width(&self) -> Result<i32> {
        construct(|res| unsafe { ffi::AImageReader_getWidth(self.as_ptr(), res) })
    }

    pub fn get_height(&self) -> Result<i32> {
        construct(|res| unsafe { ffi::AImageReader_getHeight(self.as_ptr(), res) })
    }

    pub fn get_format(&self) -> Result<ImageFormat> {
        let format = construct(|res| unsafe { ffi::AImageReader_getFormat(self.as_ptr(), res) })?;
        Ok((format as ffi::AIMAGE_FORMATS).try_into().unwrap())
    }

    pub fn get_max_images(&self) -> Result<i32> {
        construct(|res| unsafe { ffi::AImageReader_getMaxImages(self.as_ptr(), res) })
    }

    pub fn acquire_next_image(&self) -> Result<Option<Image>> {
        let res = construct_never_null(|res| unsafe {
            ffi::AImageReader_acquireNextImage(self.as_ptr(), res)
        });

        match res {
            Ok(inner) => Ok(Some(Image { inner })),
            Err(NdkMediaError::ErrorResult(MediaErrorResult::ImgreaderNoBufferAvailable)) => {
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// # Safety
    /// If the returned file descriptor is not `None`, it must be awaited before attempting to access the Image returned.
    /// <https://developer.android.com/ndk/reference/group/media#aimagereader_acquirenextimageasync>
    #[cfg(feature = "api-level-26")]
    pub unsafe fn acquire_next_image_async(&self) -> Result<(Image, Option<RawFd>)> {
        let mut fence = MaybeUninit::uninit();
        let inner = construct_never_null(|res| {
            ffi::AImageReader_acquireNextImageAsync(self.as_ptr(), res, fence.as_mut_ptr())
        })?;

        let image = Image { inner };

        Ok(match fence.assume_init() {
            -1 => (image, None),
            fence => (image, Some(fence)),
        })
    }

    pub fn acquire_latest_image(&self) -> Result<Option<Image>> {
        let res = construct_never_null(|res| unsafe {
            ffi::AImageReader_acquireLatestImage(self.as_ptr(), res)
        });

        if let Err(NdkMediaError::ErrorResult(MediaErrorResult::ImgreaderNoBufferAvailable)) = res {
            return Ok(None);
        }

        Ok(Some(Image { inner: res? }))
    }

    /// # Safety
    /// If the returned file descriptor is not `None`, it must be awaited before attempting to access the Image returned.
    /// <https://developer.android.com/ndk/reference/group/media#aimagereader_acquirelatestimageasync>
    #[cfg(feature = "api-level-26")]
    pub fn acquire_latest_image_async(&self) -> Result<(Image, Option<RawFd>)> {
        let mut fence = MaybeUninit::uninit();
        let inner = construct_never_null(|res| unsafe {
            ffi::AImageReader_acquireLatestImageAsync(self.as_ptr(), res, fence.as_mut_ptr())
        })?;

        let image = Image { inner };

        Ok(match unsafe { fence.assume_init() } {
            -1 => (image, None),
            fence => (image, Some(fence)),
        })
    }
}

impl Drop for ImageReader {
    fn drop(&mut self) {
        unsafe { ffi::AImageReader_delete(self.as_ptr()) };
    }
}

#[derive(Debug)]
pub struct Image {
    inner: NonNull<ffi::AImage>,
}

pub type CropRect = ffi::AImageCropRect;

impl Image {
    fn as_ptr(&self) -> *mut ffi::AImage {
        self.inner.as_ptr()
    }

    pub fn get_plane_data(&self, plane_idx: i32) -> Result<&[u8]> {
        let mut result_ptr = MaybeUninit::uninit();
        let mut result_len = MaybeUninit::uninit();
        let status = unsafe {
            ffi::AImage_getPlaneData(
                self.as_ptr(),
                plane_idx,
                result_ptr.as_mut_ptr(),
                result_len.as_mut_ptr(),
            )
        };

        NdkMediaError::from_status(status, || unsafe {
            std::slice::from_raw_parts(result_ptr.assume_init(), result_len.assume_init() as _)
        })
    }

    pub fn get_plane_pixel_stride(&self, plane_idx: i32) -> Result<i32> {
        construct(|res| unsafe { ffi::AImage_getPlanePixelStride(self.as_ptr(), plane_idx, res) })
    }

    pub fn get_plane_row_stride(&self, plane_idx: i32) -> Result<i32> {
        construct(|res| unsafe { ffi::AImage_getPlaneRowStride(self.as_ptr(), plane_idx, res) })
    }

    pub fn get_crop_rect(&self) -> Result<CropRect> {
        construct(|res| unsafe { ffi::AImage_getCropRect(self.as_ptr(), res) })
    }

    pub fn get_width(&self) -> Result<i32> {
        construct(|res| unsafe { ffi::AImage_getWidth(self.as_ptr(), res) })
    }

    pub fn get_height(&self) -> Result<i32> {
        construct(|res| unsafe { ffi::AImage_getHeight(self.as_ptr(), res) })
    }

    pub fn get_format(&self) -> Result<ImageFormat> {
        let format = construct(|res| unsafe { ffi::AImage_getFormat(self.as_ptr(), res) })?;
        Ok((format as ffi::AIMAGE_FORMATS).try_into().unwrap())
    }

    pub fn get_timestamp(&self) -> Result<i64> {
        construct(|res| unsafe { ffi::AImage_getTimestamp(self.as_ptr(), res) })
    }

    pub fn get_number_of_planes(&self) -> Result<i32> {
        construct(|res| unsafe { ffi::AImage_getNumberOfPlanes(self.as_ptr(), res) })
    }

    #[cfg(feature = "hardware_buffer")]
    pub fn get_hardware_buffer(&self) -> Result<HardwareBuffer> {
        unsafe {
            let ptr =
                construct_never_null(|res| ffi::AImage_getHardwareBuffer(self.as_ptr(), res))?;
            Ok(HardwareBuffer::from_ptr(ptr))
        }
    }

    #[cfg(feature = "api-level-26")]
    pub fn delete_async(self, release_fence_fd: RawFd) {
        unsafe { ffi::AImage_deleteAsync(self.as_ptr(), release_fence_fd) };
        std::mem::forget(self);
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe { ffi::AImage_delete(self.as_ptr()) };
    }
}
