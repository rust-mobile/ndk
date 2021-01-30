// TODO: mod docs

use std::os::raw::c_int;
use std::ptr;
use std::ptr::NonNull;

use crate::event::InputEvent;
use crate::looper::ForeignLooper;

// TODO docs
#[derive(Debug)]
pub struct InputQueue {
    ptr: NonNull<ffi::AInputQueue>,
}

// It gets shared between threads in android_native_app_glue
unsafe impl Send for InputQueue {}
unsafe impl Sync for InputQueue {}

#[derive(Debug)]
pub struct InputQueueError;

impl InputQueue {
    /// Construct an `InputQueue` from the native pointer.
    ///
    /// By calling this function, you assert that the pointer is a valid pointer to an NDK `AInputQueue`.
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AInputQueue>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> NonNull<ffi::AInputQueue> {
        self.ptr
    }

    pub fn get_event(&self) -> Option<InputEvent> {
        unsafe {
            let mut out_event = ptr::null_mut();
            if ffi::AInputQueue_getEvent(self.ptr.as_ptr(), &mut out_event) < 0 {
                None
            } else {
                debug_assert!(!out_event.is_null());
                Some(InputEvent::from_ptr(NonNull::new_unchecked(out_event)))
            }
        }
    }

    pub fn has_events(&self) -> Result<bool, InputQueueError> {
        unsafe {
            match ffi::AInputQueue_hasEvents(self.ptr.as_ptr()) {
                0 => Ok(false),
                1 => Ok(true),
                x if x < 0 => Err(InputQueueError),
                x => unreachable!("AInputQueue_hasEvents returned {}", x),
            }
        }
    }

    pub fn pre_dispatch(&self, event: InputEvent) -> Option<InputEvent> {
        unsafe {
            if ffi::AInputQueue_preDispatchEvent(self.ptr.as_ptr(), event.ptr().as_ptr()) == 0 {
                Some(event)
            } else {
                None
            }
        }
    }

    pub fn finish_event(&self, event: InputEvent, handled: bool) {
        unsafe {
            ffi::AInputQueue_finishEvent(self.ptr.as_ptr(), event.ptr().as_ptr(), handled as c_int);
        }
    }

    pub fn attach_looper(&self, looper: &ForeignLooper, id: i32) {
        unsafe {
            ffi::AInputQueue_attachLooper(
                self.ptr.as_ptr(),
                looper.ptr().as_ptr(),
                id,
                None,
                std::ptr::null_mut(),
            );
        }
    }

    pub fn detach_looper(&self) {
        unsafe {
            ffi::AInputQueue_detachLooper(self.ptr.as_ptr());
        }
    }
}
