#![cfg(feature = "hardware_buffer")]

pub use super::hardware_buffer_format::HardwareBufferFormat;
use jni_sys::{jobject, JNIEnv};
use std::{
    convert::TryInto, mem::MaybeUninit, ops::Deref, os::raw::c_void, os::unix::io::RawFd,
    ptr::NonNull,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HardwareBufferUsage(pub ffi::AHardwareBuffer_UsageFlags);

impl HardwareBufferUsage {
    pub const CPU_READ_NEVER: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_CPU_READ_NEVER);
    pub const CPU_READ_RARELY: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_CPU_READ_RARELY);
    pub const CPU_READ_OFTEN: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_CPU_READ_OFTEN);
    pub const CPU_READ_MASK: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_CPU_READ_MASK);

    pub const CPU_WRITE_NEVER: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_CPU_WRITE_NEVER);
    pub const CPU_WRITE_RARELY: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_CPU_WRITE_RARELY);
    pub const CPU_WRITE_OFTEN: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_CPU_WRITE_OFTEN);
    pub const CPU_WRITE_MASK: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_CPU_WRITE_MASK);

    pub const GPU_SAMPLED_IMAGE: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_GPU_SAMPLED_IMAGE);
    pub const GPU_FRAMEBUFFER: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_GPU_FRAMEBUFFER);
    pub const COMPOSER_OVERLAY: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_COMPOSER_OVERLAY);
    pub const PROTECTED_CONTENT: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_PROTECTED_CONTENT);
    pub const VIDEO_ENCODE: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VIDEO_ENCODE);
    pub const SENSOR_DIRECT_DATA: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_SENSOR_DIRECT_DATA);
    pub const GPU_DATA_BUFFER: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_GPU_DATA_BUFFER);
    pub const GPU_CUBE_MAP: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_GPU_CUBE_MAP);
    pub const GPU_MIPMAP_COMPLETE: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_GPU_MIPMAP_COMPLETE);

    pub const VENDOR_0: Self = Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_0);
    pub const VENDOR_1: Self = Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_1);
    pub const VENDOR_2: Self = Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_2);
    pub const VENDOR_3: Self = Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_3);
    pub const VENDOR_4: Self = Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_4);
    pub const VENDOR_5: Self = Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_5);
    pub const VENDOR_6: Self = Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_6);
    pub const VENDOR_7: Self = Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_7);
    pub const VENDOR_8: Self = Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_8);
    pub const VENDOR_9: Self = Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_9);
    pub const VENDOR_10: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_10);
    pub const VENDOR_11: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_11);
    pub const VENDOR_12: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_12);
    pub const VENDOR_13: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_13);
    pub const VENDOR_14: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_14);
    pub const VENDOR_15: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_15);
    pub const VENDOR_16: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_16);
    pub const VENDOR_17: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_17);
    pub const VENDOR_18: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_18);
    pub const VENDOR_19: Self =
        Self(ffi::AHardwareBuffer_UsageFlags_AHARDWAREBUFFER_USAGE_VENDOR_19);
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct HardwareBufferError(pub i32);

pub type Result<T, E = HardwareBufferError> = std::result::Result<T, E>;

pub type Rect = ffi::ARect;

fn construct<T>(with_ptr: impl FnOnce(*mut T) -> i32) -> Result<T, HardwareBufferError> {
    let mut result = MaybeUninit::uninit();
    let status = with_ptr(result.as_mut_ptr());
    if status == 0 {
        Ok(unsafe { result.assume_init() })
    } else {
        Err(HardwareBufferError(status))
    }
}

#[derive(Debug)]
pub struct HardwareBuffer {
    inner: NonNull<ffi::AHardwareBuffer>,
}

impl HardwareBuffer {
    /// Create a `HardwareBuffer` from a native pointer
    ///
    /// # Safety
    /// By calling this function, you assert that it is a valid pointer to
    /// an NDK `AHardwareBuffer`.
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AHardwareBuffer>) -> Self {
        Self { inner: ptr }
    }

    /// Returns the underlying [`ffi::AHardwareBuffer`] pointer
    ///
    /// The pointer can be used to import this hardware buffer into a Vulkan memory object using [`VK_ANDROID_external_memory_android_hardware_buffer`].
    ///
    /// [`VK_ANDROID_external_memory_android_hardware_buffer`]: https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VK_ANDROID_external_memory_android_hardware_buffer.html
    pub fn as_ptr(&self) -> *mut ffi::AHardwareBuffer {
        self.inner.as_ptr()
    }

    pub fn allocate(desc: HardwareBufferDesc) -> Result<HardwareBufferRef> {
        unsafe {
            let ptr = construct(|res| ffi::AHardwareBuffer_allocate(&desc.into_native(), res))?;

            Ok(HardwareBufferRef {
                inner: Self::from_ptr(NonNull::new_unchecked(ptr)),
            })
        }
    }

    /// Create a `HardwareBuffer` from JNI pointers
    ///
    /// # Safety
    /// By calling this function, you assert that it these are valid pointers to JNI objects.
    pub unsafe fn from_jni(env: *mut JNIEnv, hardware_buffer: jobject) -> Self {
        let ptr = ffi::AHardwareBuffer_fromHardwareBuffer(env, hardware_buffer);

        Self::from_ptr(NonNull::new_unchecked(ptr))
    }

    /// # Safety
    /// By calling this function, you assert that `env` is a valid pointer to a [`JNIEnv`].
    pub unsafe fn to_jni(&self, env: *mut JNIEnv) -> jobject {
        ffi::AHardwareBuffer_toHardwareBuffer(env, self.as_ptr())
    }

    pub fn describe(&self) -> HardwareBufferDesc {
        let desc = unsafe {
            let mut result = MaybeUninit::uninit();
            ffi::AHardwareBuffer_describe(self.as_ptr(), result.as_mut_ptr());
            result.assume_init()
        };

        HardwareBufferDesc {
            width: desc.width,
            height: desc.height,
            layers: desc.layers,
            format: desc.format.try_into().unwrap(),
            usage: HardwareBufferUsage(desc.usage),
            stride: desc.stride,
        }
    }

    #[cfg(feature = "api-level-29")]
    pub fn is_supported(desc: HardwareBufferDesc) -> bool {
        let res = unsafe { ffi::AHardwareBuffer_isSupported(&desc.into_native()) };
        res == 1
    }

    pub fn lock(
        &self,
        usage: HardwareBufferUsage,
        fence: Option<RawFd>,
        rect: Option<Rect>,
    ) -> Result<*mut c_void> {
        let fence = fence.unwrap_or(-1);
        let rect = match rect {
            Some(rect) => &rect,
            None => std::ptr::null(),
        };
        construct(|res| unsafe {
            ffi::AHardwareBuffer_lock(self.as_ptr(), usage.0, fence, rect, res)
        })
    }

    #[cfg(feature = "api-level-29")]
    pub fn lock_and_get_info(
        &self,
        usage: HardwareBufferUsage,
        fence: Option<RawFd>,
        rect: Option<Rect>,
    ) -> Result<LockedPlaneInfo> {
        let fence = fence.unwrap_or(-1);
        let rect = match rect {
            Some(rect) => &rect,
            None => std::ptr::null(),
        };
        let mut virtual_address = MaybeUninit::uninit();
        let mut bytes_per_pixel = MaybeUninit::uninit();
        let mut bytes_per_stride = MaybeUninit::uninit();
        let status = unsafe {
            ffi::AHardwareBuffer_lockAndGetInfo(
                self.as_ptr(),
                usage.0,
                fence,
                rect,
                virtual_address.as_mut_ptr(),
                bytes_per_pixel.as_mut_ptr(),
                bytes_per_stride.as_mut_ptr(),
            )
        };
        if status == 0 {
            Ok(unsafe {
                LockedPlaneInfo {
                    virtual_address: virtual_address.assume_init(),
                    bytes_per_pixel: bytes_per_pixel.assume_init() as u32,
                    bytes_per_stride: bytes_per_stride.assume_init() as u32,
                }
            })
        } else {
            Err(HardwareBufferError(status))
        }
    }

    #[cfg(feature = "api-level-29")]
    pub fn lock_planes(
        &self,
        usage: HardwareBufferUsage,
        fence: Option<RawFd>,
        rect: Option<Rect>,
    ) -> Result<HardwareBufferPlanes> {
        let fence = fence.unwrap_or(-1);
        let rect = match rect {
            Some(rect) => &rect,
            None => std::ptr::null(),
        };
        let planes = construct(|res| unsafe {
            ffi::AHardwareBuffer_lockPlanes(self.as_ptr(), usage.0, fence, rect, res)
        })?;

        Ok(HardwareBufferPlanes {
            inner: planes,
            index: 0,
        })
    }

    pub fn unlock(&self) -> Result<()> {
        let status = unsafe { ffi::AHardwareBuffer_unlock(self.as_ptr(), std::ptr::null_mut()) };
        if status == 0 {
            Ok(())
        } else {
            Err(HardwareBufferError(status))
        }
    }

    /// Returns a fence file descriptor that will become signalled when unlocking is completed,
    /// or `None` if unlocking is already finished.
    pub fn unlock_async(&self) -> Result<Option<RawFd>> {
        let fence = construct(|res| unsafe { ffi::AHardwareBuffer_unlock(self.as_ptr(), res) })?;
        Ok(match fence {
            -1 => None,
            fence => Some(fence),
        })
    }

    pub fn recv_handle_from_unix_socket(socket_fd: RawFd) -> Result<Self> {
        unsafe {
            let ptr =
                construct(|res| ffi::AHardwareBuffer_recvHandleFromUnixSocket(socket_fd, res))?;

            Ok(Self::from_ptr(NonNull::new_unchecked(ptr)))
        }
    }

    pub fn send_handle_to_unix_socket(&self, socket_fd: RawFd) -> Result<()> {
        unsafe {
            let status = ffi::AHardwareBuffer_sendHandleToUnixSocket(self.as_ptr(), socket_fd);
            if status == 0 {
                Ok(())
            } else {
                Err(HardwareBufferError(status))
            }
        }
    }

    pub fn acquire(&self) -> HardwareBufferRef {
        unsafe {
            ffi::AHardwareBuffer_acquire(self.as_ptr());
        }
        HardwareBufferRef {
            inner: HardwareBuffer { inner: self.inner },
        }
    }
}

/// A `HardwareBuffer` with an owned reference, the reference is released when dropped.
/// It behaves much like a strong `Rc` reference.
#[derive(Debug)]
pub struct HardwareBufferRef {
    inner: HardwareBuffer,
}

impl Deref for HardwareBufferRef {
    type Target = HardwareBuffer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Drop for HardwareBufferRef {
    fn drop(&mut self) {
        unsafe {
            ffi::AHardwareBuffer_release(self.inner.as_ptr());
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HardwareBufferDesc {
    width: u32,
    height: u32,
    layers: u32,
    format: HardwareBufferFormat,
    usage: HardwareBufferUsage,
    stride: u32,
}

impl HardwareBufferDesc {
    fn into_native(self) -> ffi::AHardwareBuffer_Desc {
        ffi::AHardwareBuffer_Desc {
            width: self.width,
            height: self.height,
            layers: self.layers,
            format: self.format.try_into().unwrap(),
            usage: self.usage.0,
            stride: self.stride,
            rfu0: 0,
            rfu1: 0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LockedPlaneInfo {
    pub virtual_address: *mut c_void,
    pub bytes_per_pixel: u32,
    pub bytes_per_stride: u32,
}

#[derive(Debug)]
pub struct HardwareBufferPlanes {
    inner: ffi::AHardwareBuffer_Planes,
    index: u32,
}

impl Iterator for HardwareBufferPlanes {
    type Item = LockedPlaneInfo;

    fn next(&mut self) -> Option<LockedPlaneInfo> {
        if self.index == self.inner.planeCount {
            None
        } else {
            let plane = self.inner.planes[self.index as usize];
            self.index += 1;
            Some(LockedPlaneInfo {
                virtual_address: plane.data,
                bytes_per_pixel: plane.pixelStride,
                bytes_per_stride: plane.rowStride,
            })
        }
    }
}
