//! Bindings for [`AInputQueue`]
//!
//! [`AInputQueue`]: https://developer.android.com/ndk/reference/group/input#ainputqueue

use std::io::Result;
use std::os::raw::c_int;
use std::ptr::{self, NonNull};

use crate::event::InputEvent;
#[cfg(doc)]
use crate::event::KeyEvent;
use crate::looper::ForeignLooper;
use crate::utils::status_to_io_result;

/// A native [`AInputQueue *`]
///
/// An input queue is the facility through which you retrieve input events.
///
/// [`AInputQueue *`]: https://developer.android.com/ndk/reference/group/input#ainputqueue
#[derive(Debug)]
pub struct InputQueue {
    ptr: NonNull<ffi::AInputQueue>,
}

// It gets shared between threads in `ndk-glue`
unsafe impl Send for InputQueue {}
unsafe impl Sync for InputQueue {}

impl InputQueue {
    /// Construct an [`InputQueue`] from the native pointer.
    ///
    /// # Safety
    /// By calling this function, you assert that the pointer is a valid pointer to an NDK [`ffi::AInputQueue`].
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AInputQueue>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> NonNull<ffi::AInputQueue> {
        self.ptr
    }

    /// Returns the next available [`InputEvent`] from the queue.
    ///
    /// Returns [`None`] if no event is available.
    pub fn get_event(&self) -> Result<Option<InputEvent>> {
        let mut out_event = ptr::null_mut();
        let status = unsafe { ffi::AInputQueue_getEvent(self.ptr.as_ptr(), &mut out_event) };
        match status_to_io_result(status, ()) {
            Ok(()) => {
                debug_assert!(!out_event.is_null());
                Ok(Some(unsafe {
                    InputEvent::from_ptr(NonNull::new_unchecked(out_event))
                }))
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Returns [`true`] if there are one or more events available in the input queue.
    pub fn has_events(&self) -> bool {
        match unsafe { ffi::AInputQueue_hasEvents(self.ptr.as_ptr()) } {
            0 => false,
            1 => true,
            r => unreachable!("AInputQueue_hasEvents returned non-boolean {}", r),
        }
    }

    /// Sends the key for standard pre-dispatching that is, possibly deliver it to the current IME
    /// to be consumed before the app.
    ///
    /// Returns [`Some`] if it was not pre-dispatched, meaning you can process it right now. If
    /// [`None`] is returned, you must abandon the current event processing and allow the event to
    /// appear again in the event queue (if it does not get consumed during pre-dispatching).
    ///
    /// Also returns [`None`] if `event` is not a [`KeyEvent`].
    pub fn pre_dispatch(&self, event: InputEvent) -> Option<InputEvent> {
        match unsafe { ffi::AInputQueue_preDispatchEvent(self.ptr.as_ptr(), event.ptr().as_ptr()) }
        {
            0 => Some(event),
            1 => None,
            r => unreachable!("AInputQueue_preDispatchEvent returned non-boolean {}", r),
        }
    }

    /// Report that dispatching has finished with the given [`InputEvent`].
    ///
    /// This must be called after receiving an event with [`InputQueue::get_event()`].
    pub fn finish_event(&self, event: InputEvent, handled: bool) {
        unsafe {
            ffi::AInputQueue_finishEvent(self.ptr.as_ptr(), event.ptr().as_ptr(), handled as c_int)
        }
    }

    /// Add this input queue to a [`ForeignLooper`] for processing.
    ///
    /// See [`ForeignLooper::add_fd()`] for information on the `ident`, `callback`, and `data` params.
    pub fn attach_looper(&self, looper: &ForeignLooper, id: i32) {
        unsafe {
            ffi::AInputQueue_attachLooper(
                self.ptr.as_ptr(),
                looper.ptr().as_ptr(),
                id,
                None,
                std::ptr::null_mut(),
            )
        }
    }

    /// Remove this input queue from the [`ForeignLooper`] it is currently attached to.
    pub fn detach_looper(&self) {
        unsafe { ffi::AInputQueue_detachLooper(self.ptr.as_ptr()) }
    }
}
