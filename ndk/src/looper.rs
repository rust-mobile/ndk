//! Bindings for `ALooper`
//!
//! In Android, `ALooper`s are inherently thread-local.  Due to this, there are two different
//! `ALooper` interfaces exposed in this module:
//!
//!  * `ThreadLooper`, which has methods for the operations performable with a looper in one's own
//!    thread; and
//!  * `ForeignLooper`, which has methods for the operations performable with any thread's looper.

use std::convert::TryInto;
use std::os::raw::c_void;
use std::os::unix::io::RawFd;
use std::ptr;
use std::ptr::NonNull;
use std::time::Duration;
use thiserror::Error;

/// A thread-local `ALooper`.  This contains a native `ALooper *` and promises that there is a
/// looper associated with the current thread.
#[derive(Debug)]
pub struct ThreadLooper {
    _marker: std::marker::PhantomData<*mut ()>, // Not send or sync
    foreign: ForeignLooper,
}

/// The poll result from a `ThreadLooper`.
#[derive(Debug)]
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

#[derive(Debug, Copy, Clone, Error)]
#[error("Android Looper error")]
pub struct LooperError;

impl ThreadLooper {
    /// Prepares a looper for the current thread and returns it
    pub fn prepare() -> Self {
        unsafe {
            let ptr = ffi::ALooper_prepare(ffi::ALOOPER_PREPARE_ALLOW_NON_CALLBACKS as _);
            let foreign = ForeignLooper::from_ptr(NonNull::new(ptr).expect("looper non null"));
            Self {
                _marker: std::marker::PhantomData,
                foreign,
            }
        }
    }

    /// Returns the looper associated with the current thread, if any.
    pub fn for_thread() -> Option<Self> {
        Some(Self {
            _marker: std::marker::PhantomData,
            foreign: ForeignLooper::for_thread()?,
        })
    }

    fn poll_once_ms(&self, ms: i32) -> Result<Poll, LooperError> {
        unsafe {
            let mut fd: RawFd = -1;
            let mut events: i32 = -1;
            let mut data: *mut c_void = ptr::null_mut();
            match ffi::ALooper_pollOnce(ms, &mut fd, &mut events, &mut data) {
                ffi::ALOOPER_POLL_WAKE => Ok(Poll::Wake),
                ffi::ALOOPER_POLL_CALLBACK => Ok(Poll::Callback),
                ffi::ALOOPER_POLL_TIMEOUT => Ok(Poll::Timeout),
                ffi::ALOOPER_POLL_ERROR => Err(LooperError),
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
    pub fn poll_once(&self) -> Result<Poll, LooperError> {
        self.poll_once_ms(-1)
    }

    /// Polls the looper, blocking on processing an event, but with a timeout.  Give a timeout of 0
    /// to make this non-blocking.
    ///
    /// It panics if the timeout is larger than expressible as an `i32` of milliseconds (roughly 25
    /// days).
    #[inline]
    pub fn poll_once_timeout(&self, timeout: Duration) -> Result<Poll, LooperError> {
        self.poll_once_ms(
            timeout
                .as_millis()
                .try_into()
                .expect("Supplied timeout is too large"),
        )
    }

    fn poll_all_ms(&self, ms: i32) -> Result<Poll, LooperError> {
        unsafe {
            let mut fd: RawFd = -1;
            let mut events: i32 = -1;
            let mut data: *mut c_void = ptr::null_mut();
            match ffi::ALooper_pollAll(ms, &mut fd, &mut events, &mut data) {
                ffi::ALOOPER_POLL_WAKE => Ok(Poll::Wake),
                ffi::ALOOPER_POLL_TIMEOUT => Ok(Poll::Timeout),
                ffi::ALOOPER_POLL_ERROR => Err(LooperError),
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
    pub fn poll_all(&self) -> Result<Poll, LooperError> {
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
    pub fn poll_all_timeout(&self, timeout: Duration) -> Result<Poll, LooperError> {
        self.poll_all_ms(
            timeout
                .as_millis()
                .try_into()
                .expect("Supplied timeout is too large"),
        )
    }

    /// Returns a reference to the `ForeignLooper` that is associated with the current thread.
    pub fn as_foreign(&self) -> &ForeignLooper {
        &self.foreign
    }

    pub fn into_foreign(self) -> ForeignLooper {
        self.foreign
    }
}

/// An `ALooper`, not necessarily allocated with the current thread.
#[derive(Debug)]
pub struct ForeignLooper {
    ptr: NonNull<ffi::ALooper>,
}

unsafe impl Send for ForeignLooper {}
unsafe impl Sync for ForeignLooper {}

impl Drop for ForeignLooper {
    fn drop(&mut self) {
        unsafe { ffi::ALooper_release(self.ptr.as_ptr()) }
    }
}

impl Clone for ForeignLooper {
    fn clone(&self) -> Self {
        unsafe {
            ffi::ALooper_acquire(self.ptr.as_ptr());
            Self { ptr: self.ptr }
        }
    }
}

impl ForeignLooper {
    /// Returns the looper associated with the current thread, if any.
    #[inline]
    pub fn for_thread() -> Option<Self> {
        NonNull::new(unsafe { ffi::ALooper_forThread() }).map(|ptr| unsafe { Self::from_ptr(ptr) })
    }

    /// Construct a `ForeignLooper` object from the given pointer.
    ///
    /// By calling this function, you guarantee that the pointer is a valid, non-null pointer to an
    /// NDK `ALooper`.
    #[inline]
    pub unsafe fn from_ptr(ptr: NonNull<ffi::ALooper>) -> Self {
        ffi::ALooper_acquire(ptr.as_ptr());
        Self { ptr }
    }

    /// Returns a pointer to the NDK `ALooper` object.
    #[inline]
    pub fn ptr(&self) -> NonNull<ffi::ALooper> {
        self.ptr
    }

    /// Wakes the looper.  An event of `Poll::Wake` will be sent.
    pub fn wake(&self) {
        unsafe { ffi::ALooper_wake(self.ptr.as_ptr()) }
    }

    /// Adds a file descriptor to be polled, without a callback.
    ///
    /// See also [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/looper.html#alooper_addfd).
    // TODO why is this unsafe?
    pub unsafe fn add_fd(
        &self,
        fd: RawFd,
        ident: i32,
        event: i32,
        data: *mut c_void,
    ) -> Result<(), LooperError> {
        match ffi::ALooper_addFd(self.ptr.as_ptr(), fd, ident, event, None, data) {
            1 => Ok(()),
            -1 => Err(LooperError),
            _ => unreachable!(),
        }
    }

    /// Adds a file descriptor to be polled, with a callback.
    ///
    /// The callback takes as an argument the file descriptor, and should return true to continue
    /// receiving callbacks, or false to have the callback unregistered.
    ///
    /// See also [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/looper.html#alooper_addfd).
    // TODO why is this unsafe?
    pub unsafe fn add_fd_with_callback(
        &self,
        fd: RawFd,
        ident: i32,
        event: i32,
        callback: Box<dyn FnMut(RawFd) -> bool>,
    ) -> Result<(), LooperError> {
        extern "C" fn cb_handler(fd: RawFd, _events: i32, data: *mut c_void) -> i32 {
            unsafe {
                let cb: &mut Box<dyn FnMut(RawFd) -> bool> = &mut *(data as *mut _);
                cb(fd) as i32
            }
        }
        let data: *mut c_void = Box::into_raw(Box::new(callback)) as *mut _;
        match ffi::ALooper_addFd(self.ptr.as_ptr(), fd, ident, event, Some(cb_handler), data) {
            1 => Ok(()),
            -1 => Err(LooperError),
            _ => unreachable!(),
        }
    }
}
