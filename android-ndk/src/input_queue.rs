// TODO: mod docs

use std::fmt;
use std::os::raw::c_int;
use std::ptr;

use crate::event::InputEvent;

// TODO docs
pub struct InputQueue {
    ptr: ptr::NonNull<ffi::AInputQueue>,
}

impl fmt::Debug for InputQueue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InputQueue {{ .. }}")
    }
}

pub struct InputQueueError;

impl InputQueue {
    /// Construct an `InputQueue` from the native pointer.
    ///
    /// By calling this function, you assert that the pointer is a valid pointer to an NDK `AInputQueue`.
    pub unsafe fn from_ptr(ptr: ptr::NonNull<ffi::AInputQueue>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> *mut ffi::AInputQueue {
        self.ptr.as_ptr()
    }

    pub fn get_event(&mut self) -> Option<InputEvent> {
        unsafe {
            let mut out_event = ptr::null_mut();
            if ffi::AInputQueue_getEvent(self.ptr(), &mut out_event) < 0 {
                None
            } else {
                debug_assert!(out_event != ptr::null_mut());
                Some(InputEvent::from_ptr(out_event))
            }
        }
    }

    pub fn has_events(&self) -> Result<bool, InputQueueError> {
        unsafe {
            match ffi::AInputQueue_hasEvents(self.ptr()) {
                0 => Ok(false),
                1 => Ok(true),
                x if x < 0 => Err(InputQueueError),
                _ => unreachable!(),
            }
        }
    }

    pub fn pre_dispatch(&mut self, event: InputEvent) -> Option<InputEvent> {
        unsafe {
            if ffi::AInputQueue_preDispatchEvent(self.ptr(), event.ptr()) == 0 {
                Some(event)
            } else {
                None
            }
        }
    }

    pub fn finish_event(&mut self, event: InputEvent, handled: bool) {
        unsafe {
            ffi::AInputQueue_finishEvent(self.ptr(), event.ptr(), handled as c_int);
        }
    }
}
