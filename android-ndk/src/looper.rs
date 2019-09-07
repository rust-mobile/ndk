//! Bindings for `ALooper`
//!
//! In Android, `ALooper`s are inherently thread-local.  Due to this, there are two different
//! `ALooper` interfaces exposed in this module:
//!
//!  * `ThreadLooper`, which has methods for the operations performable with a looper in one's own
//!    thread; and
//!  * `ForeignLooper`, which has methods for the operations performable with any thread's looper.

use std::convert::TryInto;
use std::fmt;
use std::os::raw::c_void;
use std::os::unix::io::RawFd;
use std::ptr;
use std::ptr::NonNull;
use std::time::Duration;

/// A thread-local `ALooper`.  This contains no real data; just the promise that there is a looper
/// associated with the current thread.
pub struct ThreadLooper {
    _marker: std::marker::PhantomData<*mut ()>, // Not send or sync
}

/// The poll result from a `ThreadLooper`.
pub enum Poll {
    /// This looper was woken using `ForeignLooper::wake`
    Wake,
    /// For `ThreadLooper::poll_once*`, an event was received and processed using a callback.
    Callback,
    /// For `ThreadLooper::poll_*_timeout`, the requested timeout was reached before any events.
    Timeout,
    /// An event was received
    Event {
        ident: i32,
        fd: RawFd,
        events: i32,
        data: *mut c_void,
    },
}

#[derive(Debug, Copy, Clone)]
pub struct PollError;

impl fmt::Display for PollError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Android Looper poll error")
    }
}

impl std::error::Error for PollError {}

impl ThreadLooper {
    /// Returns the looper associated with the current thread, if any.
    pub fn for_thread() -> Option<Self> {
        unsafe {
            if ffi::ALooper_forThread().is_null() {
                None
            } else {
                Some(Self::new_unchecked())
            }
        }
    }

    /// Create a `ThreadLooper` without checking that there is one associated with the current
    /// thread.
    pub unsafe fn new_unchecked() -> Self {
        ThreadLooper {
            _marker: std::marker::PhantomData,
        }
    }

    fn poll_once_ms(&self, ms: i32) -> Result<Poll, PollError> {
        unsafe {
            let mut fd: RawFd = -1;
            let mut events: i32 = -1;
            let mut data: *mut c_void = ptr::null_mut();
            match ffi::ALooper_pollOnce(ms, &mut fd, &mut events, &mut data) {
                ffi::ALOOPER_POLL_WAKE => Ok(Poll::Wake),
                ffi::ALOOPER_POLL_CALLBACK => Ok(Poll::Callback),
                ffi::ALOOPER_POLL_TIMEOUT => Ok(Poll::Timeout),
                ffi::ALOOPER_POLL_ERROR => Err(PollError),
                ident if ident >= 0 => Ok(Poll::Event {
                    ident,
                    fd,
                    events,
                    data,
                }),
                _ => unreachable!(),
            }
        }
    }

    /// Polls the looper, blocking on processing an event.
    #[inline]
    pub fn poll_once(&self) -> Result<Poll, PollError> {
        self.poll_once_ms(-1)
    }

    /// Polls the looper, blocking on processing an event, but with a timeout.  Give a timeout of 0
    /// to make this non-blocking.
    ///
    /// It panics if the timeout is larger than expressible as an `i32` of milliseconds (roughly 25
    /// days).
    #[inline]
    pub fn poll_once_timeout(&self, timeout: Duration) -> Result<Poll, PollError> {
        self.poll_once_ms(
            timeout
                .as_millis()
                .try_into()
                .expect("Supplied timeout is too large"),
        )
    }

    fn poll_all_ms(&self, ms: i32) -> Result<Poll, PollError> {
        unsafe {
            let mut fd: RawFd = -1;
            let mut events: i32 = -1;
            let mut data: *mut c_void = ptr::null_mut();
            match ffi::ALooper_pollAll(ms, &mut fd, &mut events, &mut data) {
                ffi::ALOOPER_POLL_WAKE => Ok(Poll::Wake),
                ffi::ALOOPER_POLL_TIMEOUT => Ok(Poll::Timeout),
                ffi::ALOOPER_POLL_ERROR => Err(PollError),
                ident if ident >= 0 => Ok(Poll::Event {
                    ident,
                    fd,
                    events,
                    data,
                }),
                _ => unreachable!(),
            }
        }
    }

    /// Repeatedly polls the looper, blocking on processing an event.
    ///
    /// This function will never return `Poll::Callback`.
    #[inline]
    pub fn poll_all(&self) -> Result<Poll, PollError> {
        self.poll_all_ms(-1)
    }

    /// Repeatedly polls the looper, blocking on processing an event, but with a timeout. Give a
    /// timeout of 0 to make this non-blocking.
    ///
    /// This function will never return `Poll::Callback`.
    ///
    /// It panics if the timeout is larger than expressible as an `i32` of milliseconds (roughly 25
    /// days).
    #[inline]
    pub fn poll_all_timeout(&self, timeout: Duration) -> Result<Poll, PollError> {
        self.poll_all_ms(
            timeout
                .as_millis()
                .try_into()
                .expect("Supplied timeout is too large"),
        )
    }
}

/// An `ALooper`, not necessarily allociated with the current thread.
pub struct ForeignLooper {
    ptr: NonNull<ffi::ALooper>,
}

unsafe impl Send for ForeignLooper {}
unsafe impl Sync for ForeignLooper {}

impl ForeignLooper {
    /// Returns the looper associated with the current thread, if any.
    #[inline]
    pub fn for_thread() -> Option<Self> {
        NonNull::new(unsafe { ffi::ALooper_forThread() }).map(|ptr| Self { ptr })
    }

    /// Construct a `ForeignLooper` object from the given pointer.
    ///
    /// By calling this function, you guarantee that the pointer is a valid, non-null pointer to an
    /// NDK `ALooper`.
    #[inline]
    pub unsafe fn from_ptr(ptr: NonNull<ffi::ALooper>) -> Self {
        Self { ptr }
    }

    /// Returns a pointer to the NDK `ALooper` object
    #[inline]
    pub fn ptr(&self) -> NonNull<ffi::ALooper> {
        self.ptr
    }

    /// Wakes the looper.  An event of `Poll::Wake` will be sent.
    pub fn wake(&self) {
        unsafe { ffi::ALooper_wake(self.ptr.as_ptr()) }
    }

    // TODO addFd, removeFd, maybe acquire/release
}
