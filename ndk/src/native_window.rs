//! Bindings for [`ANativeWindow`]
//!
//! [`ANativeWindow`]: https://developer.android.com/ndk/reference/group/a-native-window#anativewindow

use crate::utils::status_to_io_result;

pub use super::hardware_buffer_format::HardwareBufferFormat;
use jni_sys::{jobject, JNIEnv};
use raw_window_handle::{AndroidNdkWindowHandle, HasRawWindowHandle, RawWindowHandle};
use std::{ffi::c_void, io::Result, mem::MaybeUninit, ptr::NonNull};

pub type Rect = ffi::ARect;

// [`NativeWindow`] represents the producer end of an image queue
///
/// It is the C counterpart of the [`android.view.Surface`] object in Java, and can be converted
/// both ways. Depending on the consumer, images submitted to [`NativeWindow`] can be shown on the
/// display or sent to other consumers, such as video encoders.
///
/// [`android.view.Surface`]: https://developer.android.com/reference/android/view/Surface
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeWindow {
    ptr: NonNull<ffi::ANativeWindow>,
}

unsafe impl Send for NativeWindow {}
unsafe impl Sync for NativeWindow {}

impl Drop for NativeWindow {
    fn drop(&mut self) {
        unsafe { ffi::ANativeWindow_release(self.ptr.as_ptr()) }
    }
}

impl Clone for NativeWindow {
    fn clone(&self) -> Self {
        unsafe { ffi::ANativeWindow_acquire(self.ptr.as_ptr()) }
        Self { ptr: self.ptr }
    }
}

unsafe impl HasRawWindowHandle for NativeWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = AndroidNdkWindowHandle::empty();
        handle.a_native_window = self.ptr.as_ptr() as *mut c_void;
        RawWindowHandle::AndroidNdk(handle)
    }
}

impl NativeWindow {
    /// Assumes ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::ANativeWindow`].
    pub unsafe fn from_ptr(ptr: NonNull<ffi::ANativeWindow>) -> Self {
        Self { ptr }
    }

    /// Acquires ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::ANativeWindow`].
    pub unsafe fn clone_from_ptr(ptr: NonNull<ffi::ANativeWindow>) -> Self {
        ffi::ANativeWindow_acquire(ptr.as_ptr());
        Self::from_ptr(ptr)
    }

    pub fn ptr(&self) -> NonNull<ffi::ANativeWindow> {
        self.ptr
    }

    pub fn height(&self) -> i32 {
        unsafe { ffi::ANativeWindow_getHeight(self.ptr.as_ptr()) }
    }

    pub fn width(&self) -> i32 {
        unsafe { ffi::ANativeWindow_getWidth(self.ptr.as_ptr()) }
    }

    /// Return the current pixel format ([`HardwareBufferFormat`]) of the window surface.
    pub fn format(&self) -> HardwareBufferFormat {
        let value = unsafe { ffi::ANativeWindow_getFormat(self.ptr.as_ptr()) };
        let value = u32::try_from(value).unwrap();
        HardwareBufferFormat::from(ffi::AHardwareBuffer_Format(value))
    }

    /// Change the format and size of the window buffers.
    ///
    /// The width and height control the number of pixels in the buffers, not the dimensions of the
    /// window on screen. If these are different than the window's physical size, then its buffer
    /// will be scaled to match that size when compositing it to the screen. The width and height
    /// must be either both zero or both non-zero.
    ///
    /// For all of these parameters, if `0` or [`None`] is supplied then the window's base value
    /// will come back in force.
    pub fn set_buffers_geometry(
        &self,
        width: i32,
        height: i32,
        format: Option<HardwareBufferFormat>,
    ) -> Result<()> {
        let format = format.map_or(0, |f| ffi::AHardwareBuffer_Format::from(f).0);
        let status = unsafe {
            ffi::ANativeWindow_setBuffersGeometry(self.ptr.as_ptr(), width, height, format as i32)
        };
        status_to_io_result(status, ())
    }

    /// Return the [`NativeWindow`] associated with a JNI [`android.view.Surface`] pointer.
    ///
    /// # Safety
    /// By calling this function, you assert that `env` is a valid pointer to a [`JNIEnv`] and
    /// `surface` is a valid pointer to an [`android.view.Surface`].
    ///
    /// [`android.view.Surface`]: https://developer.android.com/reference/android/view/Surface
    pub unsafe fn from_surface(env: *mut JNIEnv, surface: jobject) -> Option<Self> {
        let ptr = ffi::ANativeWindow_fromSurface(env, surface);
        Some(Self::from_ptr(NonNull::new(ptr)?))
    }

    /// Return a JNI [`android.view.Surface`] pointer derived from this [`NativeWindow`].
    ///
    /// # Safety
    /// By calling this function, you assert that `env` is a valid pointer to a [`JNIEnv`].
    ///
    /// [`android.view.Surface`]: https://developer.android.com/reference/android/view/Surface
    #[cfg(feature = "api-level-26")]
    pub unsafe fn to_surface(&self, env: *mut JNIEnv) -> jobject {
        ffi::ANativeWindow_toSurface(env, self.ptr().as_ptr())
    }

    /// Lock the window's next drawing surface for writing.
    ///
    /// Optionally pass the region you intend to draw into `dirty_bounds`.  When this function
    /// returns it is updated (commonly enlarged) with the actual area the caller needs to redraw.
    pub fn lock(&self, dirty_bounds: Option<&mut Rect>) -> Result<NativeWindowBufferLockGuard<'_>> {
        let dirty_bounds = match dirty_bounds {
            Some(dirty_bounds) => dirty_bounds,
            None => std::ptr::null_mut(),
        };
        let mut buffer = MaybeUninit::uninit();
        let ret = unsafe {
            ffi::ANativeWindow_lock(self.ptr.as_ptr(), buffer.as_mut_ptr(), dirty_bounds)
        };
        status_to_io_result(ret, ())?;

        Ok(NativeWindowBufferLockGuard {
            window: self,
            buffer: unsafe { buffer.assume_init() },
        })
    }
}

/// Lock holding the next drawing surface for writing.  It is unlocked and posted on [`drop()`].
#[derive(Debug)]
pub struct NativeWindowBufferLockGuard<'a> {
    window: &'a NativeWindow,
    buffer: ffi::ANativeWindow_Buffer,
}

impl<'a> NativeWindowBufferLockGuard<'a> {
    /// The number of pixels that are shown horizontally.
    pub fn width(&self) -> usize {
        usize::try_from(self.buffer.width).unwrap()
    }

    // The number of pixels that are shown vertically.
    pub fn height(&self) -> usize {
        usize::try_from(self.buffer.height).unwrap()
    }

    /// The number of _pixels_ that a line in the buffer takes in memory.
    ///
    /// This may be `>= width`.
    pub fn stride(&self) -> usize {
        usize::try_from(self.buffer.stride).unwrap()
    }

    /// The format of the buffer. One of [`HardwareBufferFormat`].
    pub fn format(&self) -> HardwareBufferFormat {
        let format = u32::try_from(self.buffer.format).unwrap();
        HardwareBufferFormat::from(ffi::AHardwareBuffer_Format(format))
    }

    /// The actual bits.
    ///
    /// This points to a memory segment of [`stride()`][Self::stride()] *
    /// [`height()`][Self::height()] * [`HardwareBufferFormat::bytes_per_pixel()`] bytes.
    ///
    /// Only [`width()`][Self::width()] pixels are visible for each [`stride()`][Self::stride()]
    /// line of pixels in the buffer.
    ///
    /// See [`bytes()`][Self::bytes()] for safe access to these bytes.
    pub fn bits(&mut self) -> *mut c_void {
        self.buffer.bits
    }

    /// Safe write access to likely uninitialized pixel buffer data.
    ///
    /// Returns [`None`] when there is no [`HardwareBufferFormat::bytes_per_pixel()`] size
    /// available for this [`format()`][Self::format()].
    ///
    /// The returned slice consists of [`stride()`][Self::stride()] * [`height()`][Self::height()]
    /// \* [`HardwareBufferFormat::bytes_per_pixel()`] bytes.
    ///
    /// Only [`width()`][Self::width()] pixels are visible for each [`stride()`][Self::stride()]
    /// line of pixels in the buffer.
    pub fn bytes(&mut self) -> Option<&mut [MaybeUninit<u8>]> {
        let num_pixels = self.stride() * self.height();
        let num_bytes = num_pixels * self.format().bytes_per_pixel()?;
        Some(unsafe { std::slice::from_raw_parts_mut(self.bits().cast(), num_bytes) })
    }

    /// Returns a slice of bytes for each line of visible pixels in the buffer, ignoring any
    /// padding pixels incurred by the stride.
    ///
    /// See [`bits()`][Self::bits()] and [`bytes()`][Self::bytes()] for contiguous access to the
    /// underlying buffer.
    pub fn lines(&mut self) -> Option<impl Iterator<Item = &mut [MaybeUninit<u8>]>> {
        let bpp = self.format().bytes_per_pixel()?;
        let scanline_bytes = bpp * self.stride();
        let width_bytes = bpp * self.width();
        let bytes = self.bytes()?;

        Some(
            bytes
                .chunks_exact_mut(scanline_bytes)
                .map(move |scanline| &mut scanline[..width_bytes]),
        )
    }
}

impl<'a> Drop for NativeWindowBufferLockGuard<'a> {
    fn drop(&mut self) {
        let ret = unsafe { ffi::ANativeWindow_unlockAndPost(self.window.ptr.as_ptr()) };
        assert_eq!(ret, 0);
    }
}
