#![cfg(feature = "hardware_buffer")]
use std::ptr::NonNull;

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

pub struct HardwareBuffer {
    inner: NonNull<ffi::AHardwareBuffer>,
}

impl HardwareBuffer {
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AHardwareBuffer>) -> Self {
        Self { inner: ptr }
    }
}
