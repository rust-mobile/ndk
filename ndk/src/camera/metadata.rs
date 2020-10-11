use static_assertions::assert_eq_size;

use super::{
    tags::ScalerAvailableRecommendedStreamConfigurations, CameraError, CameraErrorStatus, Result,
};
use std::{
    convert::TryFrom,
    ffi::CStr,
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    ptr::NonNull,
    slice,
};

pub type Rational = ffi::ACameraMetadata_rational;
pub type Rect = ffi::ARect;

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum MetadataEntryType {
    Byte = ffi::ACAMERA_TYPE_BYTE,
    Int32 = ffi::ACAMERA_TYPE_INT32,
    Float = ffi::ACAMERA_TYPE_FLOAT,
    Int64 = ffi::ACAMERA_TYPE_INT64,
    Double = ffi::ACAMERA_TYPE_DOUBLE,
    Rational = ffi::ACAMERA_TYPE_RATIONAL,
    Unknown,
}

#[derive(Copy, Clone)]
pub struct ConstEntry<'a> {
    pub(super) entry: ffi::ACameraMetadata_const_entry,
    pub(super) _pd: PhantomData<&'a ()>,
}

impl<'a> Debug for ConstEntry<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("ConstEntry")
            .field("tag", &self.entry.tag)
            .field("count", &self.entry.count)
            .field("type", &self.entry_type())
            .field("data", unsafe { &self.entry.data.u8 })
            .finish()
    }
}

impl<'a> ConstEntry<'a> {
    fn entry_type(&self) -> MetadataEntryType {
        match u32::from(self.entry.type_) {
            ffi::ACAMERA_TYPE_BYTE => MetadataEntryType::Byte,
            ffi::ACAMERA_TYPE_INT32 => MetadataEntryType::Int32,
            ffi::ACAMERA_TYPE_FLOAT => MetadataEntryType::Float,
            ffi::ACAMERA_TYPE_INT64 => MetadataEntryType::Int64,
            ffi::ACAMERA_TYPE_DOUBLE => MetadataEntryType::Double,
            ffi::ACAMERA_TYPE_RATIONAL => MetadataEntryType::Rational,
            _ => MetadataEntryType::Unknown,
        }
    }
}

impl<'a> ConstEntry<'a> {
    /// # Safety
    /// Safe when the additional lifetime matches the corresponding CaptureMetadata/Request lifetime.
    pub unsafe fn new(entry: ffi::ACameraMetadata_const_entry) -> Self {
        Self {
            entry,
            _pd: PhantomData,
        }
    }

    pub fn inner(self) -> ffi::ACameraMetadata_const_entry {
        self.entry
    }
}

pub trait EntryType: Sized + Copy {
    const EXPECTED: MetadataEntryType;

    fn from_const_entry(entry: ConstEntry<'_>) -> &[Self];

    fn set_entry(
        req: NonNull<ffi::ACaptureRequest>,
        tag: u32,
        data: &[Self],
    ) -> ffi::camera_status_t;

    fn set_entry_physical(
        req: NonNull<ffi::ACaptureRequest>,
        physical_id: &CStr,
        tag: u32,
        data: &[Self],
    ) -> ffi::camera_status_t;
}

macro_rules! entry {
    (($type:ty, $name:literal, $expected:expr), $set_fn:path, $set_phys_fn:path, |$entry_ident:ident| $entry_field:expr,) => {
        impl EntryType for $type {
            const EXPECTED: MetadataEntryType = $expected;

            fn from_const_entry(entry: ConstEntry<'_>) -> &[Self] {
                let $entry_ident = entry.inner();

                if std::cfg!(debug_assertions) {
                    let actual = entry.entry_type();
                    assert!(
                        actual == Self::EXPECTED,
                        concat!($name, " data requested, but got {:?}. Read the Android NDK documentation for the correct type to use!"),
                        actual,
                    );
                }

                unsafe {
                    std::slice::from_raw_parts($entry_field, $entry_ident.count as usize)
                }
            }

            fn set_entry(
                req: NonNull<ffi::ACaptureRequest>,
                tag: u32,
                data: &[Self],
            ) -> ffi::camera_status_t {
                unsafe { $set_fn(req.as_ptr(), tag, data.len() as u32, data.as_ptr()) }
            }

            fn set_entry_physical(
                req: NonNull<ffi::ACaptureRequest>,
                physical_id: &CStr,
                tag: u32,
                data: &[Self],
            ) -> ffi::camera_status_t {
                unsafe {
                    $set_phys_fn(
                        req.as_ptr(),
                        physical_id.as_ptr(),
                        tag,
                        data.len() as u32,
                        data.as_ptr(),
                    )
                }
            }
        }

        // Making these generic (over T: EntryType) results in trait impl conflicts for other generic impls
        // The solution is manual monomorphization in a macro
        impl<'a> FromEntryData<'a> for $type {
            fn from_entry_data(entry: ConstEntry<'a>) -> Result<Self> {
                let slice = <$type>::from_const_entry(entry);

                if slice.len() != 1 {
                    return Err(CameraError::InvalidMetadataEntryCount(
                        slice.len(),
                        "1",
                    ));
                }

                Ok(slice[0])
            }
        }

        impl ToEntryData for $type {
            type EntryType = $type;

            fn as_entry_data(&self, set: impl FnOnce(&[$type])) {
                set(::std::slice::from_ref(self))
            }
        }
    };
}

entry!(
    (u8, "byte", MetadataEntryType::Byte),
    ffi::ACaptureRequest_setEntry_u8,
    ffi::ACaptureRequest_setEntry_physicalCamera_u8,
    |entry| entry.data.u8,
);
entry!(
    (i32, "int32", MetadataEntryType::Int32),
    ffi::ACaptureRequest_setEntry_i32,
    ffi::ACaptureRequest_setEntry_physicalCamera_i32,
    |entry| entry.data.i32,
);
entry!(
    (f32, "float", MetadataEntryType::Float),
    ffi::ACaptureRequest_setEntry_float,
    ffi::ACaptureRequest_setEntry_physicalCamera_float,
    |entry| entry.data.f,
);
entry!(
    (i64, "int64", MetadataEntryType::Int64),
    ffi::ACaptureRequest_setEntry_i64,
    ffi::ACaptureRequest_setEntry_physicalCamera_i64,
    |entry| entry.data.i64,
);
entry!(
    (f64, "double", MetadataEntryType::Double),
    ffi::ACaptureRequest_setEntry_double,
    ffi::ACaptureRequest_setEntry_physicalCamera_double,
    |entry| entry.data.d,
);
entry!(
    (Rational, "rational", MetadataEntryType::Rational),
    ffi::ACaptureRequest_setEntry_rational,
    ffi::ACaptureRequest_setEntry_physicalCamera_rational,
    |entry| entry.data.r,
);

/// Defines the conversion from a metadata entry to a struct
pub trait FromEntryData<'a>: Sized {
    fn from_entry_data(entry: ConstEntry<'a>) -> Result<Self>;
}

/// Defines the conversion from a struct to a metadata entry
pub trait ToEntryData {
    type EntryType: EntryType;
    fn as_entry_data(&self, set: impl FnOnce(&[Self::EntryType]));
}

/// Adds `Option<T>` support for optional values which are reported through
/// `CameraErrorStatus::MetadataNotFound`
pub trait FromEntryResult<'a>: Sized {
    fn from_entry_result(result: Result<ConstEntry<'a>>) -> Result<Self>;
}

impl<'a, T: FromEntryData<'a>> FromEntryResult<'a> for T {
    fn from_entry_result(result: Result<ConstEntry<'a>>) -> Result<Self> {
        result.and_then(T::from_entry_data)
    }
}

impl<'a, T: FromEntryData<'a>> FromEntryResult<'a> for Option<T> {
    fn from_entry_result(result: Result<ConstEntry<'a>>) -> Result<Self> {
        match result {
            Ok(entry) => Some(T::from_entry_data(entry)).transpose(),
            Err(CameraError::ErrorResult(CameraErrorStatus::MetadataNotFound)) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl<'a, T: EntryType> FromEntryData<'a> for &'a [T] {
    fn from_entry_data(entry: ConstEntry<'a>) -> Result<Self> {
        Ok(T::from_const_entry(entry))
    }
}
impl<T: EntryType> ToEntryData for [T] {
    type EntryType = T;
    fn as_entry_data(&self, set: impl FnOnce(&[Self::EntryType])) {
        set(self)
    }
}

macro_rules! entry_array {
    ($len:literal) => {
        impl<T: EntryType> ToEntryData for [T; $len] {
            type EntryType = T;
            fn as_entry_data(&self, set: impl FnOnce(&[Self::EntryType])) {
                set(&self[..])
            }
        }

        impl<'a, T: EntryType> FromEntryData<'a> for &'a [T; $len] {
            fn from_entry_data(entry: ConstEntry<'a>) -> Result<Self> {
                use ::std::convert::TryInto;
                let slice = T::from_const_entry(entry);
                slice.try_into().map_err(|_| {
                    CameraError::InvalidMetadataEntryCount(slice.len(), stringify!($len))
                })
            }
        }
    };
}

entry_array!(2);
entry_array!(3);
entry_array!(4);
entry_array!(5);
entry_array!(9);

#[derive(Debug)]
pub enum AnyEntry<'a> {
    Byte(&'a [u8]),
    Int32(&'a [i32]),
    Float(&'a [f32]),
    Int64(&'a [i64]),
    Double(&'a [f64]),
    Rational(&'a [Rational]),
}

impl<'a> FromEntryData<'a> for AnyEntry<'a> {
    fn from_entry_data(entry: ConstEntry<'a>) -> Result<Self> {
        Ok(match entry.entry_type() {
            MetadataEntryType::Byte => AnyEntry::Byte(u8::from_const_entry(entry)),
            MetadataEntryType::Int32 => AnyEntry::Int32(i32::from_const_entry(entry)),
            MetadataEntryType::Float => AnyEntry::Float(f32::from_const_entry(entry)),
            MetadataEntryType::Int64 => AnyEntry::Int64(i64::from_const_entry(entry)),
            MetadataEntryType::Double => AnyEntry::Double(f64::from_const_entry(entry)),
            MetadataEntryType::Rational => AnyEntry::Rational(Rational::from_const_entry(entry)),
            MetadataEntryType::Unknown => {
                panic!("Unknown metadata type found, could not create AnyEntry")
            }
        })
    }
}

pub trait CameraEnum {
    type Entry: EntryType;
}

impl<'a, T> FromEntryData<'a> for T
where
    T: CameraEnum + TryFrom<<T as CameraEnum>::Entry> + Debug,
    <T as CameraEnum>::Entry: Into<i64>,
{
    fn from_entry_data(entry: ConstEntry<'a>) -> Result<Self> {
        let slice = <Self as CameraEnum>::Entry::from_const_entry(entry);

        if slice.len() != 1 {
            return Err(CameraError::InvalidMetadataEntryCount(slice.len(), "1"));
        }

        Self::try_from(slice[0]).map_err(|_| CameraError::UnsupportedEnumValue(slice[0].into()))
    }
}

impl<T> ToEntryData for T
where
    T: CameraEnum + Copy + Into<<T as CameraEnum>::Entry>,
{
    type EntryType = <T as CameraEnum>::Entry;

    fn as_entry_data(&self, set: impl FnOnce(&[Self::EntryType])) {
        set(&[(*self).into()])
    }
}

macro_rules! slice_ref_from_entry {
    ($name:ty, [$type:ty; $count:expr]) => {
        assert_eq_size!($name, [$type; $count]);

        impl<'a> FromEntryData<'a> for &'a [$name] {
            fn from_entry_data(entry: ConstEntry<'a>) -> Result<Self> {
                let slice = <$type>::from_const_entry(entry);

                if slice.len() % $count != 0 {
                    return Err(CameraError::InvalidMetadataEntryCount(
                        slice.len(),
                        concat!("an exact multiple of ", stringify!($count)),
                    ));
                }

                Ok(unsafe {
                    let ptr = slice.as_ptr() as *const $name;
                    slice::from_raw_parts(ptr, slice.len() / $count)
                })
            }
        }
    };
}

impl<'a> FromEntryData<'a> for Rect {
    fn from_entry_data(entry: ConstEntry<'a>) -> Result<Self> {
        let &[left, top, width, height] = <&[i32; 4]>::from_entry_data(entry)?;
        Ok(Rect {
            left,
            top,
            bottom: top + height,
            right: left + width,
        })
    }
}

impl ToEntryData for Rect {
    type EntryType = i32;

    fn as_entry_data(&self, set: impl FnOnce(&[Self::EntryType])) {
        let arr = [
            self.left,
            self.top,
            self.right - self.left,
            self.bottom - self.top,
        ];
        set(&arr);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamConfiguration {
    pub format: i32,
    pub width: i32,
    pub height: i32,
    pub is_input: i32,
}

slice_ref_from_entry!(StreamConfiguration, [i32; 4]);

impl StreamConfiguration {
    /// Get the `ImageFormat` for this configuration.
    /// If the format is not a known `ImageFormat`, the value is returned as `Err(value)`.
    #[cfg(feature = "media")]
    pub fn format(&self) -> Result<crate::media::image_reader::ImageFormat, i32> {
        let format = u32::try_from(self.format).map_err(|_| self.format)?;
        TryFrom::try_from(format).map_err(|_| self.format)
    }

    pub fn size(&self) -> Size {
        let &Self { width, height, .. } = self;
        Size { width, height }
    }

    pub fn is_input(&self) -> bool {
        self.is_input == (ffi::acamera_metadata_enum_acamera_scaler_available_stream_configurations_ACAMERA_SCALER_AVAILABLE_STREAM_CONFIGURATIONS_INPUT as i32)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamConfigurationDuration {
    pub format: i64,
    pub width: i64,
    pub height: i64,
    pub duration: i64,
}

slice_ref_from_entry!(StreamConfigurationDuration, [i64; 4]);

impl StreamConfigurationDuration {
    /// Get the `ImageFormat` for this configuration.
    /// If the format is not a known `ImageFormat`, the value is returned as `Err(value)`.
    #[cfg(feature = "media")]
    pub fn format(&self) -> Result<crate::media::image_reader::ImageFormat, i64> {
        let format = u32::try_from(self.format).map_err(|_| self.format)?;
        TryFrom::try_from(format).map_err(|_| self.format)
    }

    pub fn size(&self) -> Size {
        let &Self { width, height, .. } = self;
        Size {
            width: width as _,
            height: height as _,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecommendedStreamConfiguration {
    pub format: i32,
    pub width: i32,
    pub height: i32,
    pub is_input: i32,
    pub usecase: i32,
}

slice_ref_from_entry!(RecommendedStreamConfiguration, [i32; 5]);

impl RecommendedStreamConfiguration {
    /// Get the `ImageFormat` for this configuration.
    /// If the format is not a known `ImageFormat`, the value is returned as `Err(value)`.
    #[cfg(feature = "media")]
    pub fn format(&self) -> Result<crate::media::image_reader::ImageFormat, i32> {
        let format = u32::try_from(self.format).map_err(|_| self.format)?;
        TryFrom::try_from(format).map_err(|_| self.format)
    }

    pub fn usecase(&self) -> Result<ScalerAvailableRecommendedStreamConfigurations, i32> {
        TryFrom::try_from(self.usecase).map_err(|_| self.usecase)
    }

    pub fn size(&self) -> Size {
        let &Self { width, height, .. } = self;
        Size { width, height }
    }

    pub fn is_input(&self) -> bool {
        self.is_input == (ffi::acamera_metadata_enum_acamera_scaler_available_stream_configurations_ACAMERA_SCALER_AVAILABLE_STREAM_CONFIGURATIONS_INPUT as i32)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RggbChannelVector {
    pub red: f32,
    pub green_even: f32,
    pub green_odd: f32,
    pub blue: f32,
}

impl<'a> FromEntryData<'a> for RggbChannelVector {
    fn from_entry_data(entry: ConstEntry<'a>) -> Result<Self> {
        let data = <&[f32; 4]>::from_entry_data(entry)?;

        Ok(RggbChannelVector {
            red: data[0],
            green_even: data[1],
            green_odd: data[2],
            blue: data[3],
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

slice_ref_from_entry!(Point, [i32; 2]);

/// A 4-element vector of integers corresponding to a 2x2 pattern of color channel offsets used
/// for the black level offsets of each color channel.
/// For a camera device with MONOCHROME capability, all 4 elements of the pattern will have the same value.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BlackLevelPattern {
    pub offsets: [i32; 4],
}

impl BlackLevelPattern {
    /// Return the color channel offset for a given index into the array of raw pixel values.
    pub fn get_offset_for_index(&self, column: usize, row: usize) -> i32 {
        self.offsets[((row & 1) << 1) | (column & 1)]
    }
}

impl<'a> FromEntryData<'a> for BlackLevelPattern {
    fn from_entry_data(entry: ConstEntry<'a>) -> Result<Self> {
        let data = <&[i32; 4]>::from_entry_data(entry)?;

        Ok(BlackLevelPattern { offsets: *data })
    }
}

/// Describes a 3x3 matrix of `Rational` values in row-major order.
///
/// This matrix maps a transform from one color space to another.
/// For the particular color space source and target, see the appropriate camera metadata documentation for the key that provides this value.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct ColorSpaceTransform {
    pub elements: [Rational; Self::COUNT],
}

impl ColorSpaceTransform {
    const COLUMNS: usize = 3;
    const ROWS: usize = 3;
    const COUNT: usize = Self::ROWS * Self::COLUMNS;

    /// Get an element of this matrix by its row and column.
    ///
    /// The rows must be within the range [0, 3), and the column must be within the range [0, 3).
    pub fn get_element(&self, column: usize, row: usize) -> Rational {
        self.elements[row * Self::COLUMNS + column]
    }
}

assert_eq_size!([Rational; ColorSpaceTransform::COUNT], ColorSpaceTransform);

impl<'a> FromEntryData<'a> for &'a ColorSpaceTransform {
    fn from_entry_data(entry: ConstEntry<'a>) -> Result<Self> {
        let slice = <&[Rational; 3 * 3]>::from_entry_data(entry)?;

        let result = unsafe { &*(slice as *const [Rational; 9] as *const ColorSpaceTransform) };

        Ok(result)
    }
}

impl ToEntryData for ColorSpaceTransform {
    type EntryType = Rational;

    fn as_entry_data(&self, set: impl FnOnce(&[Self::EntryType])) {
        let slice = unsafe { &*(self as *const ColorSpaceTransform as *const [Rational; 9]) };
        set(slice);
    }
}

/// Represents a rectangle with an additional weight component.
///
/// The rectangle is defined to be inclusive of the specified coordinates.
/// When used with a `CaptureRequest`, the coordinate system is based on the active pixel array.
///
/// The weight must range from `WEIGHT_MIN` to `WEIGHT_MAX` inclusively,
/// and represents a weight for every pixel in the area. This means that a large metering area with the same weight as a smaller area will have more effect in the metering result.
/// Metering areas can partially overlap and the camera device will add the weights in the overlap rectangle.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MeteringRectangle {
    pub xmin: i32,
    pub ymin: i32,
    pub xmax: i32,
    pub ymax: i32,
    pub weight: i32,
}

impl MeteringRectangle {
    /// Weights set to this value will cause the camera device to ignore this rectangle.
    /// If all metering rectangles are weighed with 0, the camera device will choose its own metering rectangles.
    pub const WEIGHT_DONT_CARE: i32 = 0;
    /// The minimum value of valid metering weight.
    pub const WEIGHT_MIN: i32 = 0;
    /// The maximum value of valid metering weight.
    pub const WEIGHT_MAX: i32 = 1000;

    pub fn width(&self) -> i32 {
        self.xmax - self.xmin
    }

    pub fn height(&self) -> i32 {
        self.ymax - self.ymin
    }
}

slice_ref_from_entry!(MeteringRectangle, [i32; 5]);

impl ToEntryData for [MeteringRectangle] {
    type EntryType = i32;

    fn as_entry_data(&self, set: impl FnOnce(&[Self::EntryType])) {
        let slice = unsafe { slice::from_raw_parts(self.as_ptr() as *const i32, self.len() * 5) };
        set(slice);
    }
}
