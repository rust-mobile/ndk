//! Bindings for [`AHardwareBuffer_Format`]
//!
//! [`AHardwareBuffer_Format`]: https://developer.android.com/ndk/reference/group/a-hardware-buffer#ahardwarebuffer_format

/// Buffer pixel formats.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum HardwareBufferFormat {
    /// Matches deprecated [`ffi::ANativeWindow_LegacyFormat::WINDOW_FORMAT_RGBA_8888`].
    R8G8B8A8_UNORM,
    /// Matches deprecated [`ffi::ANativeWindow_LegacyFormat::WINDOW_FORMAT_RGBX_8888`].
    R8G8B8X8_UNORM,
    #[cfg(feature = "api-level-26")]
    R8G8B8_UNORM,
    /// Matches deprecated [`ffi::ANativeWindow_LegacyFormat::WINDOW_FORMAT_RGB_565`].
    R5G6B5_UNORM,
    #[cfg(feature = "api-level-26")]
    R16G16B16A16_FLOAT,
    #[cfg(feature = "api-level-26")]
    R10G10B10A2_UNORM,
    #[cfg(feature = "api-level-26")]
    BLOB,
    #[cfg(feature = "api-level-26")]
    D16_UNORM,
    #[cfg(feature = "api-level-26")]
    D24_UNORM,
    #[cfg(feature = "api-level-26")]
    D24_UNORM_S8_UINT,
    #[cfg(feature = "api-level-26")]
    D32_FLOAT,
    #[cfg(feature = "api-level-26")]
    D32_FLOAT_S8_UINT,
    #[cfg(feature = "api-level-26")]
    S8_UINT,
    #[cfg(feature = "api-level-26")]
    Y8Cb8Cr8_420,
    #[cfg(feature = "api-level-26")]
    YCbCr_P010,
    #[cfg(feature = "api-level-26")]
    R8_UNORM,
    Unknown(ffi::AHardwareBuffer_Format),
}

impl From<ffi::AHardwareBuffer_Format> for HardwareBufferFormat {
    fn from(value: ffi::AHardwareBuffer_Format) -> Self {
        use ffi::AHardwareBuffer_Format as AFormat;
        use HardwareBufferFormat::*;
        match value {
            AFormat::AHARDWAREBUFFER_FORMAT_R8G8B8A8_UNORM => R8G8B8A8_UNORM,
            AFormat::AHARDWAREBUFFER_FORMAT_R8G8B8X8_UNORM => R8G8B8X8_UNORM,
            #[cfg(feature = "api-level-26")]
            AFormat::AHARDWAREBUFFER_FORMAT_R8G8B8_UNORM => R8G8B8_UNORM,
            AFormat::AHARDWAREBUFFER_FORMAT_R5G6B5_UNORM => R5G6B5_UNORM,
            #[cfg(feature = "api-level-26")]
            AFormat::AHARDWAREBUFFER_FORMAT_R16G16B16A16_FLOAT => R16G16B16A16_FLOAT,
            #[cfg(feature = "api-level-26")]
            AFormat::AHARDWAREBUFFER_FORMAT_R10G10B10A2_UNORM => R10G10B10A2_UNORM,
            #[cfg(feature = "api-level-26")]
            AFormat::AHARDWAREBUFFER_FORMAT_BLOB => BLOB,
            #[cfg(feature = "api-level-26")]
            AFormat::AHARDWAREBUFFER_FORMAT_D16_UNORM => D16_UNORM,
            #[cfg(feature = "api-level-26")]
            AFormat::AHARDWAREBUFFER_FORMAT_D24_UNORM => D24_UNORM,
            #[cfg(feature = "api-level-26")]
            AFormat::AHARDWAREBUFFER_FORMAT_D24_UNORM_S8_UINT => D24_UNORM_S8_UINT,
            #[cfg(feature = "api-level-26")]
            AFormat::AHARDWAREBUFFER_FORMAT_D32_FLOAT => D32_FLOAT,
            #[cfg(feature = "api-level-26")]
            AFormat::AHARDWAREBUFFER_FORMAT_D32_FLOAT_S8_UINT => D32_FLOAT_S8_UINT,
            #[cfg(feature = "api-level-26")]
            AFormat::AHARDWAREBUFFER_FORMAT_S8_UINT => S8_UINT,
            #[cfg(feature = "api-level-26")]
            AFormat::AHARDWAREBUFFER_FORMAT_Y8Cb8Cr8_420 => Y8Cb8Cr8_420,
            #[cfg(feature = "api-level-26")]
            AFormat::AHARDWAREBUFFER_FORMAT_YCbCr_P010 => YCbCr_P010,
            #[cfg(feature = "api-level-26")]
            AFormat::AHARDWAREBUFFER_FORMAT_R8_UNORM => R8_UNORM,
            _ => Unknown(value),
        }
    }
}

impl From<HardwareBufferFormat> for ffi::AHardwareBuffer_Format {
    fn from(value: HardwareBufferFormat) -> Self {
        use ffi::AHardwareBuffer_Format as AFormat;
        use HardwareBufferFormat::*;
        match value {
            R8G8B8A8_UNORM => AFormat::AHARDWAREBUFFER_FORMAT_R8G8B8A8_UNORM,
            R8G8B8X8_UNORM => AFormat::AHARDWAREBUFFER_FORMAT_R8G8B8X8_UNORM,
            #[cfg(feature = "api-level-26")]
            R8G8B8_UNORM => AFormat::AHARDWAREBUFFER_FORMAT_R8G8B8_UNORM,
            R5G6B5_UNORM => AFormat::AHARDWAREBUFFER_FORMAT_R5G6B5_UNORM,
            #[cfg(feature = "api-level-26")]
            R16G16B16A16_FLOAT => AFormat::AHARDWAREBUFFER_FORMAT_R16G16B16A16_FLOAT,
            #[cfg(feature = "api-level-26")]
            R10G10B10A2_UNORM => AFormat::AHARDWAREBUFFER_FORMAT_R10G10B10A2_UNORM,
            #[cfg(feature = "api-level-26")]
            BLOB => AFormat::AHARDWAREBUFFER_FORMAT_BLOB,
            #[cfg(feature = "api-level-26")]
            D16_UNORM => AFormat::AHARDWAREBUFFER_FORMAT_D16_UNORM,
            #[cfg(feature = "api-level-26")]
            D24_UNORM => AFormat::AHARDWAREBUFFER_FORMAT_D24_UNORM,
            #[cfg(feature = "api-level-26")]
            D24_UNORM_S8_UINT => AFormat::AHARDWAREBUFFER_FORMAT_D24_UNORM_S8_UINT,
            #[cfg(feature = "api-level-26")]
            D32_FLOAT => AFormat::AHARDWAREBUFFER_FORMAT_D32_FLOAT,
            #[cfg(feature = "api-level-26")]
            D32_FLOAT_S8_UINT => AFormat::AHARDWAREBUFFER_FORMAT_D32_FLOAT_S8_UINT,
            #[cfg(feature = "api-level-26")]
            S8_UINT => AFormat::AHARDWAREBUFFER_FORMAT_S8_UINT,
            #[cfg(feature = "api-level-26")]
            Y8Cb8Cr8_420 => AFormat::AHARDWAREBUFFER_FORMAT_Y8Cb8Cr8_420,
            #[cfg(feature = "api-level-26")]
            YCbCr_P010 => AFormat::AHARDWAREBUFFER_FORMAT_YCbCr_P010,
            #[cfg(feature = "api-level-26")]
            R8_UNORM => AFormat::AHARDWAREBUFFER_FORMAT_R8_UNORM,
            Unknown(x) => x,
        }
    }
}

impl HardwareBufferFormat {
    /// Returns [`None`] when there is no immediate byte size available for this format, for
    /// example on planar buffer formats.
    pub fn bytes_per_pixel(self) -> Option<usize> {
        Some(match self {
            Self::R8G8B8A8_UNORM | Self::R8G8B8X8_UNORM => 4,
            #[cfg(feature = "api-level-26")]
            Self::R8G8B8_UNORM => 3,
            Self::R5G6B5_UNORM => 2,
            #[cfg(feature = "api-level-26")]
            Self::R16G16B16A16_FLOAT => 8,
            #[cfg(feature = "api-level-26")]
            Self::R10G10B10A2_UNORM => 4,
            #[cfg(feature = "api-level-26")]
            Self::BLOB => 1,
            #[cfg(feature = "api-level-26")]
            Self::D16_UNORM => 2,
            #[cfg(feature = "api-level-26")]
            Self::D24_UNORM => 3,
            #[cfg(feature = "api-level-26")]
            Self::D24_UNORM_S8_UINT => 4,
            #[cfg(feature = "api-level-26")]
            Self::D32_FLOAT => 4,
            #[cfg(feature = "api-level-26")]
            Self::D32_FLOAT_S8_UINT => 5,
            #[cfg(feature = "api-level-26")]
            Self::S8_UINT => 1,
            #[cfg(feature = "api-level-26")]
            Self::Y8Cb8Cr8_420 => 3,
            #[cfg(feature = "api-level-26")]
            Self::YCbCr_P010 => return None,
            #[cfg(feature = "api-level-26")]
            Self::R8_UNORM => 1,
            Self::Unknown(_) => return None,
        })
    }
}
