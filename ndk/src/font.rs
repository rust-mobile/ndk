//! Bindings for [`AFont`], [`AFontMatcher`], and [`ASystemFontIterator`]
//!
//! [`AFont`]: https://developer.android.com/ndk/reference/group/font#afont_close
//! [`AFontMatcher`]: https://developer.android.com/ndk/reference/group/font#afontmatcher_create
//! [`ASystemFontIterator`]: https://developer.android.com/ndk/reference/group/font#asystemfontiterator_open

use std::convert::TryFrom;
use std::ffi::{CStr, OsStr};
use std::fmt;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use std::ptr::NonNull;

/// Encapsulates font weights.
///
/// See the followings for more details:
/// * [`AFONT_WEIGHT_*`]
/// * [`Font::weight`]
///
/// [`AFONT_WEIGHT_*`]: https://developer.android.com/ndk/reference/group/font#anonymous-enum-33
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct FontWeight(u16);

impl FontWeight {
    pub const fn new(value: u16) -> Option<Self> {
        if 0 < value && value <= 1000 {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn value(&self) -> u16 {
        self.0
    }

    pub const THIN: FontWeight = FontWeight(ffi::AFONT_WEIGHT_THIN as u16);
    pub const EXTRA_LIGHT: FontWeight = FontWeight(ffi::AFONT_WEIGHT_EXTRA_LIGHT as u16);
    pub const LIGHT: FontWeight = FontWeight(ffi::AFONT_WEIGHT_LIGHT as u16);
    pub const NORMAL: FontWeight = FontWeight(ffi::AFONT_WEIGHT_NORMAL as u16);
    pub const MEDIUM: FontWeight = FontWeight(ffi::AFONT_WEIGHT_MEDIUM as u16);
    pub const SEMI_BOLD: FontWeight = FontWeight(ffi::AFONT_WEIGHT_SEMI_BOLD as u16);
    pub const BOLD: FontWeight = FontWeight(ffi::AFONT_WEIGHT_BOLD as u16);
    pub const EXTRA_BOLD: FontWeight = FontWeight(ffi::AFONT_WEIGHT_EXTRA_BOLD as u16);
    pub const BLACK: FontWeight = FontWeight(ffi::AFONT_WEIGHT_BLACK as u16);
    pub const MAX: FontWeight = FontWeight(ffi::AFONT_WEIGHT_MAX as u16);
}

/// The error type returned when an invalie font weight value is passed.
#[derive(Debug)]
pub struct TryFromU16Error(());

impl fmt::Display for TryFromU16Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "Font weight must be positive and less than or equal to 1000"
        )
    }
}

impl std::error::Error for TryFromU16Error {}

impl TryFrom<u16> for FontWeight {
    type Error = TryFromU16Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        FontWeight::new(value).ok_or(TryFromU16Error(()))
    }
}

/// A native [`AFont *`]
///
/// [`AFont *`]: https://developer.android.com/ndk/reference/group/font#afont_close
#[cfg(feature = "api-level-29")]
#[derive(Debug)]
pub struct Font {
    ptr: NonNull<ffi::AFont>,
}

#[cfg(feature = "api-level-29")]
impl Font {
    /// Create an `Font` from a pointer
    ///
    /// # Safety
    /// By calling this function, you assert that the pointer is a valid pointer to a native
    /// `AFont`.
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AFont>) -> Self {
        Self { ptr }
    }

    /// Returns the pointer to the native `AFont`.
    pub fn ptr(&self) -> NonNull<ffi::AFont> {
        self.ptr
    }

    /// Return a count of font variation settings associated with the current font.
    ///
    /// The font variation settings are provided as multiple tag-values pairs.
    ///
    /// For example, bold italic font may have following font variation settings: 'wght' 700,
    /// 'slnt' -12. In this case, [`Font::axis_count`] returns 2 and [`Font::axis_tag_at`] and
    /// [`Font::axis_value_at`] will return following values.
    pub fn axis_count(&self) -> usize {
        unsafe { ffi::AFont_getAxisCount(self.ptr.as_ptr()) as usize }
    }

    /// Return an OpenType axis tag associated with the current font.
    ///
    /// See [`Font::axis_count`] for more details.
    pub fn axis_tag_at(&self, idx: usize) -> u32 {
        unsafe { ffi::AFont_getAxisTag(self.ptr.as_ptr(), idx as u32) }
    }

    /// Return an OpenType axis value associated with the current font.
    ///
    /// See [`Font::axis_count`] for more details.
    pub fn axis_value_at(&self, idx: usize) -> f32 {
        unsafe { ffi::AFont_getAxisValue(self.ptr.as_ptr(), idx as u32) }
    }

    /// Return a font collection index value associated with the current font.
    ///
    /// In case the target font file is a font collection (e.g. .ttc or .otc), this returns a non
    /// negative value as an font offset in the collection. This always returns 0 if the target font
    /// file is a regular font.
    pub fn collection_index(&self) -> usize {
        unsafe { ffi::AFont_getCollectionIndex(self.ptr.as_ptr()) as usize }
    }

    /// Return an absolute path to the current font file.
    ///
    /// Here is a list of font formats returned by this method:
    ///
    /// * OpenType
    /// * OpenType Font Collection
    /// * TrueType
    /// * TrueType Collection
    ///
    /// The file extension could be one of *.otf, *.ttf, *.otc or *.ttc.
    /// The font file returned is guaranteed to be opened with `O_RDONLY`.
    pub fn path(&self) -> &Path {
        let path = unsafe { CStr::from_ptr(ffi::AFont_getFontFilePath(self.ptr.as_ptr())) };
        OsStr::from_bytes(path.to_bytes()).as_ref()
    }

    /// Return a IETF BCP47 compliant language tag associated with the current font.
    ///
    /// For information about IETF BCP47, read [`Locale.forLanguageTag(java.lang.String)`].
    ///
    /// [`Locale.forLanguageTag(java.lang.String)`]: https://developer.android.com/reference/java/util/Locale.html#forLanguageTag(java.lang.String)
    pub fn locale(&self) -> &CStr {
        unsafe { CStr::from_ptr(ffi::AFont_getLocale(self.ptr.as_ptr())) }
    }

    /// Return a weight value associated with the current font.
    ///
    /// The weight values are positive and less than or equal to 1000. Here are pairs of the common
    /// names and their values.
    ///
    /// | Value | Name                      | NDK Definition              |
    /// | ----- | ------------------------- | --------------------------- |
    /// | 100   | Thin                      | [`FontWeight::THIN`]        |
    /// | 200   | Extra Light (Ultra Light) | [`FontWeight::EXTRA_LIGHT`] |
    /// | 300   | Light                     | [`FontWeight::LIGHT`]       |
    /// | 400   | Normal (Regular)          | [`FontWeight::NORMAL`]      |
    /// | 500   | Medium                    | [`FontWeight::MEDIUM`]      |
    /// | 600   | Semi Bold (Demi Bold)     | [`FontWeight::SEMI_BOLD`]   |
    /// | 700   | Bold                      | [`FontWeight::BOLD`]        |
    /// | 800   | Extra Bold (Ultra Bold)   | [`FontWeight::EXTRA_BOLD`]  |
    /// | 900   | Black (Heavy)             | [`FontWeight::BLACK`]       |
    pub fn weight(&self) -> FontWeight {
        FontWeight(unsafe { ffi::AFont_getWeight(self.ptr.as_ptr()) })
    }

    /// Return true if the current font is italic, otherwise returns false.
    pub fn is_italic(&self) -> bool {
        unsafe { ffi::AFont_isItalic(self.ptr.as_ptr()) }
    }
}

#[cfg(feature = "api-level-29")]
impl Drop for Font {
    fn drop(&mut self) {
        unsafe { ffi::AFont_close(self.ptr.as_ptr()) }
    }
}

/// Encapsulates [`AFAMILY_VARIANT_*`].
///
/// [`AFAMILY_VARIANT_*`]: https://developer.android.com/ndk/reference/group/font#group___font_1gga96a58e29e8dbf2b5bdeb775cba46556ea662aafc7016e35d6758da93416fc0833
#[repr(u32)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::unnecessary_cast)]
pub enum FamilyVariant {
    Compact = ffi::AFAMILY_VARIANT_COMPACT as u32,
    Default = ffi::AFAMILY_VARIANT_DEFAULT as u32,
    Elegant = ffi::AFAMILY_VARIANT_ELEGANT as u32,
}

/// A native [`AFontMatcher *`]
///
/// [`AFontMatcher *`]: https://developer.android.com/ndk/reference/group/font#afontmatcher_create
#[cfg(feature = "api-level-29")]
#[derive(Debug)]
pub struct FontMatcher {
    ptr: NonNull<ffi::AFontMatcher>,
}

#[cfg(feature = "api-level-29")]
impl FontMatcher {
    /// Create an `FontMatcher` from a pointer
    ///
    /// # Safety
    /// By calling this function, you assert that the pointer is a valid pointer to a native
    /// `AFontMatcher`.
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AFontMatcher>) -> Self {
        Self { ptr }
    }

    /// Returns the pointer to the native `AFontMatcher`.
    pub fn ptr(&self) -> NonNull<ffi::AFontMatcher> {
        self.ptr
    }

    /// Select the best font from given parameters.
    ///
    /// Creates a new [`FontMatcher`] object.
    pub fn new() -> Self {
        NonNull::new(unsafe { ffi::AFontMatcher_create() })
            .map(|p| unsafe { FontMatcher::from_ptr(p) })
            .unwrap()
    }

    /// Set family variant to matcher.
    ///
    /// If this function is not called, the matcher performs with [`FamilyVariant::Default`].
    pub fn set_family_variant(&mut self, family_variant: FamilyVariant) {
        unsafe { ffi::AFontMatcher_setFamilyVariant(self.ptr.as_ptr(), family_variant as u32) }
    }

    /// Set font locales to matcher.
    ///
    /// If this function is not called, the matcher performs with empty locale list.
    ///
    /// # Arguments
    /// * `language_tags`: a null character terminated comma separated IETF BCP47 compliant language tags.
    pub fn set_locales(&mut self, language_tags: &CStr) {
        unsafe { ffi::AFontMatcher_setLocales(self.ptr.as_ptr(), language_tags.as_ptr()) }
    }

    /// Set font style to matcher.
    ///
    /// If this function is not called, the matcher performs with [`FontWeight::NORMAL`] with non-italic style.
    pub fn set_style(&mut self, weight: FontWeight, italic: bool) {
        unsafe { ffi::AFontMatcher_setStyle(self.ptr.as_ptr(), weight.value(), italic) }
    }
}

#[cfg(feature = "api-level-29")]
impl Drop for FontMatcher {
    fn drop(&mut self) {
        unsafe { ffi::AFontMatcher_destroy(self.ptr.as_ptr()) }
    }
}

/// A native [`ASystemFontIterator *`]
///
/// [`ASystemFontIterator *`]: https://developer.android.com/ndk/reference/group/font#asystemfontiterator_open
#[cfg(feature = "api-level-29")]
#[derive(Debug)]
pub struct SystemFontIterator {
    ptr: NonNull<ffi::ASystemFontIterator>,
}

#[cfg(feature = "api-level-29")]
impl SystemFontIterator {
    /// Create an `SystemFontIterator` from a pointer
    ///
    /// # Safety
    /// By calling this function, you assert that the pointer is a valid pointer to a native
    /// `ASystemFontIterator`.
    pub unsafe fn from_ptr(ptr: NonNull<ffi::ASystemFontIterator>) -> Self {
        Self { ptr }
    }

    /// Returns the pointer to the native `ASystemFontIterator`.
    pub fn ptr(&self) -> NonNull<ffi::ASystemFontIterator> {
        self.ptr
    }

    /// Create a system font iterator.
    pub fn new() -> Self {
        NonNull::new(unsafe { ffi::ASystemFontIterator_open() })
            .map(|p| unsafe { SystemFontIterator::from_ptr(p) })
            .unwrap()
    }
}

#[cfg(feature = "api-level-29")]
impl Iterator for SystemFontIterator {
    type Item = Font;

    fn next(&mut self) -> Option<Self::Item> {
        NonNull::new(unsafe { ffi::ASystemFontIterator_next(self.ptr.as_ptr()) })
            .map(|p| unsafe { Font::from_ptr(p) })
    }
}

#[cfg(feature = "api-level-29")]
impl Drop for SystemFontIterator {
    fn drop(&mut self) {
        unsafe { ffi::ASystemFontIterator_close(self.ptr.as_ptr()) }
    }
}
