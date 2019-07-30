// TODO: mod docs

use std::convert::TryInto;
use std::os::raw::c_void;
use std::os::unix::io::RawFd;
use std::ptr;
use std::time::Duration;

// TODO docs
pub struct ThreadLooper {
    _marker: std::marker::PhantomData<*mut ()>, // Not send or sync
}

// TODO docs
pub enum Poll {
    Wake,
    Callback,
    Timeout,
    Event(i32, RawFd, i32, *mut c_void),
}

// TODO impl Error
#[derive(Debug, Copy, Clone)]
pub struct PollError;

impl ThreadLooper {
    /// Returns the looper associated with the current thread, if any.
    pub fn for_thread() -> Option<Self> {
        if unsafe { ffi::ALooper_forThread() } == ptr::null_mut() {
            None
        } else {
            Some(ThreadLooper {
                _marker: std::marker::PhantomData,
            })
        }
    }

    fn poll_once_ms(&self, ms: i32) -> Result<Poll, PollError> {
        unsafe {
            let mut fd: RawFd = -1;
            let mut out_events: i32 = -1;
            let mut out_data: *mut c_void = ptr::null_mut();
            match ffi::ALooper_pollOnce(ms, &mut fd, &mut out_events, &mut out_data) {
                ffi::ALOOPER_POLL_WAKE => Ok(Poll::Wake),
                ffi::ALOOPER_POLL_CALLBACK => Ok(Poll::Callback),
                ffi::ALOOPER_POLL_TIMEOUT => Ok(Poll::Timeout),
                ffi::ALOOPER_POLL_ERROR => Err(PollError),
                x if x >= 0 => Ok(Poll::Event(x, fd, out_events, out_data)),
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
            let mut out_events: i32 = -1;
            let mut out_data: *mut c_void = ptr::null_mut();
            match ffi::ALooper_pollAll(ms, &mut fd, &mut out_events, &mut out_data) {
                ffi::ALOOPER_POLL_WAKE => Ok(Poll::Wake),
                ffi::ALOOPER_POLL_TIMEOUT => Ok(Poll::Timeout),
                ffi::ALOOPER_POLL_ERROR => Err(PollError),
                x if x >= 0 => Ok(Poll::Event(x, fd, out_events, out_data)),
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

// TODO docs
pub struct ForeignLooper {
    ptr: *mut ffi::ALooper,
}

impl ForeignLooper {
    /// Returns the looper associated with the current thread, if any.
    #[inline]
    pub fn for_thread() -> Option<Self> {
        let ptr = unsafe { ffi::ALooper_forThread() };
        if ptr == ptr::null_mut() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    /// Construct a `ForeignLooper` object from the given pointer.
    ///
    /// By calling this function, you guarantee that the pointer is a valid, non-null pointer to an
    /// NDK `ALooper`.
    #[inline]
    pub unsafe fn from_ptr(ptr: *mut ffi::ALooper) -> Self {
        Self { ptr }
    }

    /// Returns a pointer to the NDK `ALooper` object
    pub fn ptr(&self) -> *mut ffi::ALooper {
        self.ptr
    }

    // TODO wrap addFd, removeFd, maybe acquire/release
}
