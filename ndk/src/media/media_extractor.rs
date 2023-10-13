//! Bindings for [`AMediaExtractor`]
//!
//! [`AMediaExtractor`]: https://developer.android.com/ndk/reference/group/media#amediaextractor

use std::{
    ffi::CStr,
    fmt,
    marker::PhantomData,
    ops::Range,
    os::unix::io::{AsRawFd, BorrowedFd},
    pin::Pin,
    ptr::{self, NonNull},
    time::Duration,
};

use num_enum::{IntoPrimitive, TryFromPrimitive};

use super::{media_codec::MediaFormat, media_codec_crypto::MediaCodecCryptoInfo};
use crate::{
    media_error::{MediaError, Result},
    utils::abort_on_panic,
};

#[derive(Debug)]
#[doc(alias = "AMediaExtractor")]
pub struct MediaExtractor {
    inner: NonNull<ffi::AMediaExtractor>,
}

impl Default for MediaExtractor {
    #[doc(alias = "AMediaExtractor_new")]
    fn default() -> Self {
        Self::new()
    }
}

impl MediaExtractor {
    /// Create new media extractor.
    #[doc(alias = "AMediaExtractor_new")]
    pub fn new() -> Self {
        Self {
            inner: NonNull::new(unsafe { ffi::AMediaExtractor_new() }).unwrap(),
        }
    }

    /// Set the URI from which the extractor will read.
    #[doc(alias = "AMediaExtractor_setDataSource")]
    pub fn set_data_source(&self, uri: &CStr) -> Result<()> {
        let status =
            unsafe { ffi::AMediaExtractor_setDataSource(self.inner.as_ptr(), uri.as_ptr()) };
        MediaError::from_status(status)
    }

    /// Set the file descriptor from which the extractor will read.
    // TODO: Lifetime?
    #[doc(alias = "AMediaExtractor_setDataSourceFd")]
    pub fn set_data_source_fd(&self, fd: BorrowedFd<'_>, range: Range<usize>) -> Result<()> {
        let status = unsafe {
            ffi::AMediaExtractor_setDataSourceFd(
                self.inner.as_ptr(),
                // TODO:
                fd.as_raw_fd(),
                range.start.try_into().expect("usize -> i64 overflow"),
                (range.end - range.start)
                    .try_into()
                    .expect("usize -> i64 overflow"),
            )
        };
        MediaError::from_status(status)
    }

    /// Set the custom data source implementation from which the extractor will read.
    #[cfg(feature = "api-level-28")]
    #[doc(alias = "AMediaExtractor_setDataSourceCustom")]
    pub fn set_data_source_custom(&self, data_source: &MediaDataSource) -> Result<()> {
        let status = unsafe {
            ffi::AMediaExtractor_setDataSourceCustom(
                self.inner.as_ptr(),
                data_source.inner.as_ptr(),
            )
        };
        MediaError::from_status(status)
    }

    /// Return the number of tracks in the previously specified media file.
    #[doc(alias = "AMediaExtractor_getTrackCount")]
    pub fn track_count(&self) -> usize {
        unsafe { ffi::AMediaExtractor_getTrackCount(self.inner.as_ptr()) }
    }

    /// Return the format of the specified track.
    #[doc(alias = "AMediaExtractor_getTrackFormat")]
    pub fn track_format(&self, track_index: usize) -> MediaFormat {
        let media_format =
            unsafe { ffi::AMediaExtractor_getTrackFormat(self.inner.as_ptr(), track_index) };
        unsafe { MediaFormat::from_ptr(NonNull::new(media_format).unwrap()) }
    }

    /// Select the specified track.
    ///
    /// Subsequent calls to [`Self::read_sample_data()`], [`Self::sample_track_index()`] and
    /// [`Self::sample_time()`] only retrieve information for the subset of tracks selected.
    /// Selecting the same track multiple times has no effect, the track is only selected once.
    #[doc(alias = "AMediaExtractor_selectTrack")]
    pub fn select_track(&self, track_index: usize) -> Result<()> {
        let status = unsafe { ffi::AMediaExtractor_selectTrack(self.inner.as_ptr(), track_index) };
        MediaError::from_status(status)
    }

    /// Unselect the specified track.
    ///
    /// Subsequent calls to [`Self::read_sample_data()`], [`Self::sample_track_index()`] and
    /// [`Self::sample_time()`] only retrieve information for the subset of tracks selected.
    #[doc(alias = "AMediaExtractor_unselectTrack")]
    pub fn unselect_track(&self, track_index: usize) -> Result<()> {
        let status =
            unsafe { ffi::AMediaExtractor_unselectTrack(self.inner.as_ptr(), track_index) };
        MediaError::from_status(status)
    }

    /// Read the current sample.
    ///
    /// Returns [`None`] if the sample could not be copied, e.g. when `buffer` is too small or no
    /// sample is selected or available (end of stream).
    ///
    /// After retrieving sample data, advance to the next sample with [`MediaExtractor::advance()`].
    #[doc(alias = "AMediaExtractor_readSampleData")]
    // TODO: MaybeUninit
    pub fn read_sample_data(&self, buffer: &mut [u8]) -> Option<usize> {
        let status = unsafe {
            ffi::AMediaExtractor_readSampleData(
                self.inner.as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len(),
            )
        };
        match status {
            -1 => None,
            x if x < 0 => unreachable!("readSampleData should not return negative value {x}"),
            x => Some(x as usize),
        }
    }

    /// Read the current sample's flags.
    #[doc(alias = "AMediaExtractor_getSampleFlags")]
    pub fn sample_flags(&self) -> Option<MediaExtractorSampleFlags> {
        let flags = unsafe { ffi::AMediaExtractor_getSampleFlags(self.inner.as_ptr()) };
        match flags {
            u32::MAX => None,
            flags => Some(MediaExtractorSampleFlags::from_bits_retain(flags)),
        }
    }

    /// Returns the track index the current sample originates from (or [`None`] if no more samples
    /// are available).
    #[doc(alias = "AMediaExtractor_getSampleTrackIndex")]
    pub fn sample_track_index(&self) -> Option<u32> {
        let status = unsafe { ffi::AMediaExtractor_getSampleTrackIndex(self.inner.as_ptr()) };
        match status {
            -1 => None,
            x if x < 0 => unreachable!("getSampleTrackIndex should not return negative value {x}"),
            x => Some(x as u32),
        }
    }

    /// Returns the current sample's presentation time in microseconds.
    ///
    /// Or [`None`] if no more samples are available.
    #[doc(alias = "AMediaExtractor_getSampleTime")]
    pub fn sample_time(&self) -> Option<Duration> {
        let samples = unsafe { ffi::AMediaExtractor_getSampleTime(self.inner.as_ptr()) };
        match samples {
            -1 => None,
            x if x < 0 => unreachable!("getSampleTime should not return negative value {x}"),
            x => Some(Duration::from_micros(x as u64)),
        }
    }

    /// Advance to the next sample.
    ///
    /// Returns [`false`] if no more sample data is available (end of stream).
    #[doc(alias = "AMediaExtractor_advance")]
    #[must_use]
    pub fn advance(&self) -> bool {
        unsafe { ffi::AMediaExtractor_advance(self.inner.as_ptr()) }
    }

    /// Seek to an absolute position, aligning to the given sync point in [`SeekMode`].
    #[doc(alias = "AMediaExtractor_seekTo")]
    pub fn seek_to(&self, seek_pos: Duration, seek_mode: SeekMode) -> Result<()> {
        let status = unsafe {
            ffi::AMediaExtractor_seekTo(
                self.inner.as_ptr(),
                seek_pos
                    .as_micros()
                    .try_into()
                    .expect("Nanoseconds overflow when converting to i64"),
                seek_mode.into(),
            )
        };
        MediaError::from_status(status)
    }

    /// Get the PSSH info if present.
    // TODO: lifetime of the returned pointers is unclear
    #[doc(alias = "AMediaExtractor_getPsshInfo")]
    pub fn pssh_info<'a>(&self) -> Option<PsshInfo<'a>> {
        let info = unsafe { ffi::AMediaExtractor_getPsshInfo(self.inner.as_ptr()) };
        // NonNull::new(info).map(|ptr| unsafe { *ptr.cast().as_ref() })
        NonNull::new(info).map(|ptr| unsafe { ptr::read(ptr.cast().as_ptr()) })
    }

    #[doc(alias = "AMediaExtractor_getSampleCryptoInfo")]
    pub fn sample_crypto_info(&self) -> Option<MediaCodecCryptoInfo> {
        let sample_crypto_info =
            NonNull::new(unsafe { ffi::AMediaExtractor_getSampleCryptoInfo(self.inner.as_ptr()) })?;
        Some(unsafe { MediaCodecCryptoInfo::from_ptr(sample_crypto_info) })
    }

    /// Returns the format of the extractor.
    ///
    /// this function will always return a format; however, the format could be empty (no key-value
    /// pairs) if the media container does not provide format information.
    #[cfg(feature = "api-level-28")]
    #[doc(alias = "AMediaExtractor_getFileFormat")]
    pub fn file_format(&self) -> MediaFormat {
        let format = unsafe { ffi::AMediaExtractor_getFileFormat(self.inner.as_ptr()) };
        let format = NonNull::new(format).expect("getFileFormat should never return NULL");
        unsafe { MediaFormat::from_ptr(format) }
    }

    /// Returns the size of the current sample in bytes, or [`None`] when no samples are available
    /// (end of stream).
    ///
    /// This API can be used in in conjunction with [`MediaExtractor::read_sample_data()`]:
    ///
    /// ```no_run
    /// # let ex = ndk::media::media_extractor::MediaExtractor::new();
    /// let sample_size = ex.sample_size().expect("No sample available");
    /// let mut buf = vec![0u8; sample_size];
    /// let sample_data = ex.read_sample_data(&mut buf);
    /// ```
    #[cfg(feature = "api-level-28")]
    #[doc(alias = "AMediaExtractor_getSampleSize")]
    pub fn sample_size(&self) -> Option<usize> {
        let sample_size = unsafe { ffi::AMediaExtractor_getSampleSize(self.inner.as_ptr()) };

        match sample_size {
            -1 => None,
            x if x < 0 => unreachable!("getSampleSize should not return negative value {x}"),
            x => Some(x as usize),
        }
    }

    /// Returns the duration of cached media samples downloaded from a network data source
    /// ([`MediaExtractor::set_data_source()`] with a ``"http(s)"` URI) in microseconds.
    ///
    /// This information is calculated using total bitrate; if total bitrate is not in the media
    /// container it is calculated using total duration and file size.
    ///
    /// Returns [`None`] when the extractor is not reading from a network data source, or when the cached
    /// duration cannot be calculated (bitrate, duration, and file size information not available).
    #[cfg(feature = "api-level-28")]
    #[doc(alias = "AMediaExtractor_getCachedDuration")]
    pub fn cached_duration(&self) -> Option<Duration> {
        let duration = unsafe { ffi::AMediaExtractor_getCachedDuration(self.inner.as_ptr()) };
        match duration {
            -1 => None,
            x if x < 0 => unreachable!("getCachedDuration should not return negative value {x}"),
            x => Some(Duration::from_micros(x as u64)),
        }
    }

    /// Read the current sample's metadata format into `fmt`.
    ///
    /// Examples of sample metadata are SEI (supplemental enhancement information) and MPEG user
    /// data, both of which can embed closed-caption data.
    ///
    /// Existing key-value pairs in `fmt` would be removed if this API returns [`Ok`]. The contents
    /// of `fmt` is undefined if this API returns [`Err`].
    #[cfg(feature = "api-level-28")]
    #[doc(alias = "AMediaExtractor_getSampleFormat")]
    pub fn sample_format(&self, media_format: &mut MediaFormat) -> Result<()> {
        let status = unsafe {
            ffi::AMediaExtractor_getSampleFormat(self.inner.as_ptr(), media_format.as_ptr())
        };
        MediaError::from_status(status)
    }
}

impl Drop for MediaExtractor {
    /// Delete a previously created media extractor.
    #[doc(alias = "AMediaExtractor_delete")]
    fn drop(&mut self) {
        let status = unsafe { ffi::AMediaExtractor_delete(self.inner.as_ptr()) };
        MediaError::from_status(status).unwrap()
    }
}

/// List of crypto schemes and their data.
#[derive(Debug)]
#[repr(transparent)]
pub struct PsshInfo<'a>(ffi::PsshInfo, PhantomData<&'a PsshEntry<'a>>);

impl<'a> PsshInfo<'a> {
    pub fn entries(&self) -> &[ffi::PsshEntry] {
        unsafe { self.0.entries.as_slice(self.0.numentries) }
    }
}

/// Mapping of crypto scheme uuid to the scheme specific data for that scheme.
#[derive(Debug)]
#[repr(transparent)]
pub struct PsshEntry<'a>(ffi::PsshEntry, PhantomData<&'a [u8]>);

impl<'a> PsshEntry<'a> {
    pub fn uuid(&self) -> ffi::AMediaUUID {
        self.0.uuid
    }

    pub fn data(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.0.data.cast(), self.0.datalen) }
    }
}

/// Called to request data from the given `offset`.
///
/// Implementations should should write up to the size of the slice number of bytes into `buffer`,
/// and return the number of bytes written.
///
/// Return `0` if the slice is empty (thus no bytes are read).
///
/// Return [`None`] to indicate that end of stream is reached.
#[cfg(feature = "api-level-28")]
#[doc(alias = "AMediaDataSourceReadAt")]
pub type MediaDataSourceReadAt = Box<dyn FnMut(usize, &mut [u8]) -> Option<usize> + Send + Sync>;

/// Called to get the size of the data source.
///
/// Return the size of data source in bytes, or [`None`] if the size is unknown.
#[cfg(feature = "api-level-28")]
#[doc(alias = "AMediaDataSourceGetSize")]
pub type MediaDataSourceGetSize = Box<dyn FnMut() -> Option<usize> + Send + Sync>;

/// Called to close the data source, unblock reads, and release associated resources.
///
/// The NDK media framework guarantees that after the first [`MediaDataSourceCallbacks::close`]
/// is called, no future callbacks will be invoked on the data source except for
/// [`MediaDataSourceCallbacks::close`] itself.
///
/// Closing a data source allows [`MediaDataSourceCallbacks::read_at`] calls that were blocked
/// waiting for I/O data to return promptly.
///
/// When using [`MediaDataSource`] as input to [`MediaExtractor`], closing has the
/// effect of unblocking slow reads inside of [`MediaExtractor::set_data_source()`] and
/// [`MediaExtractor::read_sample_data()`].
#[cfg(feature = "api-level-28")]
#[doc(alias = "AMediaDataSourceClose")]
pub type MediaDataSourceClose = Box<dyn FnMut() + Send + Sync>;

/// Called to get an estimate of the number of bytes that can be read from this data source starting
/// at `offset` without blocking for I/O.
///
/// Return [`None`] when such an estimate is not possible.
#[cfg(feature = "api-level-29")]
#[doc(alias = "AMediaDataSourceGetAvailableSize")]
pub type MediaDataSourceGetAvailableSize =
    Box<dyn FnMut(Option<usize>) -> Option<usize> + Send + Sync>;

/// Callbacks for [`MediaDataSource`].
#[derive(Default)]
#[cfg(feature = "api-level-28")]
pub struct MediaDataSourceCallbacks {
    /// Set a custom callback for supplying random access media data to the NDK media framework.
    ///
    /// Implement this if your app has special requirements for the way media data is obtained, or
    /// if you need a callback when data is read by the NDK media framework.
    ///
    /// Please refer to the definition of [`MediaDataSourceReadAt`] for additional details.
    pub read_at: Option<MediaDataSourceReadAt>,
    /// Set a custom callback for supplying the size of the data source to the NDK media framework.
    ///
    /// Please refer to the definition of [`MediaDataSourceGetSize`] for additional details.
    pub get_size: Option<MediaDataSourceGetSize>,
    /// Set a custom callback to receive signal from the NDK media framework when the data source
    /// is closed.
    ///
    /// Please refer to the definition of [`MediaDataSourceClose`] for additional details.
    pub close: Option<MediaDataSourceClose>,
    /// Set a custom callback for supplying the estimated number of bytes
    /// that can be read from this data source starting at an offset without
    /// blocking for I/O.
    ///
    /// Please refer to the definition of [`MediaDataSourceGetAvailableSize`] for additional
    /// details.
    #[cfg(feature = "api-level-29")]
    pub get_available_size: Option<MediaDataSourceGetAvailableSize>,
}

impl fmt::Debug for MediaDataSourceCallbacks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("MediaDataSourceCallbacks");
        d.field(
            "read_at",
            match &self.read_at {
                Some(_) => &"Some(_)",
                None => &"None",
            },
        )
        .field(
            "get_size",
            match &self.get_size {
                Some(_) => &"Some(_)",
                None => &"None",
            },
        )
        .field(
            "close",
            match &self.close {
                Some(_) => &"Some(_)",
                None => &"None",
            },
        );
        #[cfg(feature = "api-level-29")]
        {
            d.field(
                "get_available_size",
                match &self.get_available_size {
                    Some(_) => &"Some(_)",
                    None => &"None",
                },
            );
        }
        d.finish()
    }
}

/// [`MediaDataSource`]'s callbacks will be invoked on an implementation-defined thread or
/// thread pool. No guarantees are provided about which thread(s) will be used for callbacks.
/// For example, [`MediaDataSourceCallbacks::close`] can be invoked from a different thread than
/// the thread invoking [`MediaDataSourceCallbacks::read_at`]. As such, the Implementations of
/// [`MediaDataSource`] callbacks must be threadsafe.
///
/// Note that [`ffi::AMediaDataSource_setUserdata()`] is not made available as it is used internally
/// to store a pointer to this [`MediaDataSource`] wrapper in which boxed callbacks are stored.  Any
/// relevant userdata can be stored inside the closure instead.
#[cfg(feature = "api-level-28")]
#[derive(Debug)]
#[doc(alias = "AMediaDataSource")]
pub struct MediaDataSource {
    inner: NonNull<ffi::AMediaDataSource>,
    // TODO: Option
    callbacks: Pin<Box<MediaDataSourceCallbacks>>,
}

#[cfg(feature = "api-level-28")]
impl MediaDataSource {
    /// Create new media data source. Returns [`None`] if memory allocation for the new data source
    /// object fails.
    #[doc(alias = "AMediaDataSource_new")]
    pub fn new() -> Option<Self> {
        NonNull::new(unsafe { ffi::AMediaDataSource_new() }).map(|inner| Self {
            inner,
            callbacks: Box::pin(Default::default()),
        })
    }

    /// Create new media data source. Returns [`None`] if memory allocation for the new data source
    /// object fails.
    ///
    /// Set the `uri` from which the data source will read, plus additional http headers when
    /// initiating the request.
    #[cfg(feature = "api-level-29")]
    #[doc(alias = "AMediaDataSource_newUri")]
    pub fn new_uri<'h>(
        uri: &CStr,
        key_values: impl IntoIterator<Item = (&'h CStr, &'h CStr)>,
    ) -> Option<Self> {
        let ptrs = key_values
            .into_iter()
            .flat_map(|(k, v)| [k.as_ptr(), v.as_ptr()])
            .collect::<Vec<_>>();
        NonNull::new(unsafe {
            ffi::AMediaDataSource_newUri(
                uri.as_ptr(),
                // Headers are passed as a flat list of:
                // ptrs[0]: ptrs[1]
                // ptrs[2]: ptrs[3]
                (ptrs.len() / 2).try_into().expect("usize -> i32 overflow"),
                ptrs.as_ptr(),
            )
        })
        .map(|inner| Self {
            inner,
            callbacks: Box::pin(Default::default()),
        })
    }
    /// Close the data source, unblock reads, and release associated resources.
    ///
    /// Please refer to the definition of [`MediaDataSourceClose`] for additional details.
    #[cfg(feature = "api-level-29")]
    #[doc(alias = "AMediaDataSource_close")]
    pub fn close(&self) {
        unsafe { ffi::AMediaDataSource_close(self.inner.as_ptr()) }
    }

    #[doc(alias = "AMediaDataSource_setUserdata")]
    #[doc(alias = "AMediaDataSource_setReadAt")]
    #[doc(alias = "AMediaDataSource_setGetSize")]
    #[doc(alias = "AMediaDataSource_setClose")]
    #[doc(alias = "AMediaDataSource_setGetAvailableSize")]
    pub fn set_callbacks(&mut self, callbacks: MediaDataSourceCallbacks) {
        let mut boxed = Box::pin(callbacks);
        unsafe {
            ffi::AMediaDataSource_setUserdata(self.inner.as_ptr(), <*mut _>::cast(&mut *boxed))
        };

        unsafe extern "C" fn ffi_read_at(
            userdata: *mut ::std::os::raw::c_void,
            offset: i64,
            buffer: *mut ::std::os::raw::c_void,
            size: usize,
        ) -> isize {
            abort_on_panic(|| {
                let callback = &mut *(userdata as *mut MediaDataSourceCallbacks);
                let f = callback.read_at.as_mut().expect("Async?");
                let buffer = unsafe { std::slice::from_raw_parts_mut(buffer.cast(), size) };
                let offset = offset.try_into().expect("Offset cannot be negative");
                f(offset, buffer).map_or(-1, |s| s as isize)
            })
        }
        unsafe {
            ffi::AMediaDataSource_setReadAt(
                self.inner.as_ptr(),
                if boxed.read_at.is_some() {
                    Some(ffi_read_at)
                } else {
                    None
                },
            )
        };

        unsafe extern "C" fn ffi_get_size(userdata: *mut ::std::os::raw::c_void) -> isize {
            abort_on_panic(|| {
                let callback = &mut *(userdata as *mut MediaDataSourceCallbacks);
                let f = callback.get_size.as_mut().expect("Async?");
                f().map_or(-1, |s| s as isize)
            })
        }
        // let ffi_get_size = boxed.get_size.map(|_| ffi_get_size);
        unsafe {
            ffi::AMediaDataSource_setGetSize(
                self.inner.as_ptr(),
                if boxed.get_size.is_some() {
                    Some(ffi_get_size)
                } else {
                    None
                },
            )
        };

        unsafe extern "C" fn ffi_close(userdata: *mut ::std::os::raw::c_void) {
            abort_on_panic(|| {
                let callback = &mut *(userdata as *mut MediaDataSourceCallbacks);
                let f = callback.close.as_mut().expect("Async?");
                f()
            })
        }
        // let ffi_close = boxed.close.map(|_| ffi_close);
        unsafe {
            ffi::AMediaDataSource_setClose(
                self.inner.as_ptr(),
                if boxed.close.is_some() {
                    Some(ffi_close)
                } else {
                    None
                },
            )
        };

        #[cfg(feature = "api-level-29")]
        {
            unsafe extern "C" fn ffi_get_available_size(
                userdata: *mut ::std::os::raw::c_void,
                offset: i64,
            ) -> isize {
                abort_on_panic(|| {
                    let callback = &mut *(userdata as *mut MediaDataSourceCallbacks);
                    let f = callback.get_available_size.as_mut().expect("Async?");
                    let offset = match offset {
                        -1 => None,
                        x if x < 0 => {
                            unreachable!("AMediaDataSourceGetAvailableSize callback should not receive negative value {x}")
                        }
                        x => Some(x as usize),
                    };
                    f(offset).map_or(-1, |s| s as isize)
                })
            }
            // let ffi_get_available_size = boxed.get_available_size.map(|_| ffi_get_available_size);
            unsafe {
                ffi::AMediaDataSource_setGetAvailableSize(
                    self.inner.as_ptr(),
                    if boxed.get_available_size.is_some() {
                        Some(ffi_get_available_size)
                    } else {
                        None
                    },
                )
            };
        }

        self.callbacks = boxed;
    }
}

impl Drop for MediaDataSource {
    /// Delete a previously created media data source.
    #[doc(alias = "AMediaDataSource_delete")]
    fn drop(&mut self) {
        unsafe { ffi::AMediaDataSource_delete(self.inner.as_ptr()) }
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct MediaExtractorSampleFlags : u32 {
        #[doc(alias = "AMEDIAEXTRACTOR_SAMPLE_FLAG_SYNC")]
        const SYNC = ffi::AMEDIAEXTRACTOR_SAMPLE_FLAG_SYNC;
        #[doc(alias = "AMEDIAEXTRACTOR_SAMPLE_FLAG_ENCRYPTED")]
        const ENCRYPTED = ffi::AMEDIAEXTRACTOR_SAMPLE_FLAG_ENCRYPTED;
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[non_exhaustive]
pub enum SeekMode {
    #[doc(alias = "AMEDIAEXTRACTOR_SEEK_PREVIOUS_SYNC")]
    PreviousSync = ffi::SeekMode::AMEDIAEXTRACTOR_SEEK_PREVIOUS_SYNC.0,
    #[doc(alias = "AMEDIAEXTRACTOR_SEEK_NEXT_SYNC")]
    NextSync = ffi::SeekMode::AMEDIAEXTRACTOR_SEEK_NEXT_SYNC.0,
    #[doc(alias = "AMEDIAEXTRACTOR_SEEK_CLOSEST_SYNC")]
    ClosestSync = ffi::SeekMode::AMEDIAEXTRACTOR_SEEK_CLOSEST_SYNC.0,
}

impl From<SeekMode> for ffi::SeekMode {
    fn from(value: SeekMode) -> Self {
        ffi::SeekMode(value.into())
    }
}
