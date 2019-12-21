//! Bindings for the `android_app` struct found in `android_native_app_glue.c`
//!
//! If you are not using `native_app_glue`, you can disable these bindings by disabling the
//! `native_app_glue` Cargo feature.

use crate::configuration::Configuration;
use crate::input_queue::InputQueue;
use crate::looper::ForeignLooper;
use crate::native_activity::NativeActivity;
use crate::native_window::NativeWindow;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryInto;
use std::fmt;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::NonNull;

/// A `struct android_app *`.
#[derive(Debug)]
pub struct AndroidApp {
    ptr: NonNull<ffi::native_app_glue::android_app>,
}

// It is used between threads in android_native_app_glue
unsafe impl Send for AndroidApp {}
unsafe impl Sync for AndroidApp {}

// TODO: docs
impl AndroidApp {
    pub unsafe fn from_ptr(ptr: NonNull<ffi::native_app_glue::android_app>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> NonNull<ffi::native_app_glue::android_app> {
        self.ptr
    }

    // It's OK to not give a Ref<'_, _> because the ANativeActivity * will never change
    pub fn activity(&self) -> NativeActivity {
        unsafe { NativeActivity::from_ptr(NonNull::new(self.ptr.as_ref().activity).unwrap()) }
    }

    pub fn native_window(&self) -> Option<Ref<'_, NativeWindow>> {
        unsafe {
            if let Some(ptr) = NonNull::new(self.ptr.as_ref().window) {
                return Some(Ref::new(NativeWindow::from_ptr(ptr)));
            }
        }
        None
    }

    pub fn input_queue(&self) -> Option<Ref<'_, InputQueue>> {
        unsafe {
            if let Some(ptr) = NonNull::new(self.ptr.as_ref().inputQueue) {
                return Some(Ref::new(InputQueue::from_ptr(ptr)));
            }
        }
        None
    }

    pub fn config(&self) -> Ref<'_, Configuration> {
        unsafe {
            Ref::new(Configuration::from_ptr(
                NonNull::new(self.ptr.as_ref().config).unwrap(),
            ))
        }
    }

    /* pub */
    fn read_cmd(&mut self) -> Cmd {
        unsafe {
            ffi::native_app_glue::android_app_read_cmd(self.ptr.as_ptr())
                .try_into()
                .unwrap()
        }
    }

    /* pub */
    fn pre_exec(&mut self, cmd: Cmd) {
        unsafe {
            ffi::native_app_glue::android_app_pre_exec_cmd(self.ptr.as_ptr(), cmd.into());
        }
    }

    /* pub */
    fn post_exec(&mut self, cmd: Cmd) {
        unsafe {
            ffi::native_app_glue::android_app_post_exec_cmd(self.ptr.as_ptr(), cmd.into());
        }
    }

    pub fn handle_cmd<T>(&mut self, f: impl FnOnce(&mut Self, Cmd) -> T) -> T {
        let cmd = self.read_cmd();
        self.pre_exec(cmd);
        let res = f(self, cmd);
        self.post_exec(cmd);
        res
    }

    pub fn saved_state(&self) -> Option<&[u8]> {
        unsafe {
            let ptr = self.ptr.as_ref().savedState;
            if ptr.is_null() {
                None
            } else {
                Some(std::slice::from_raw_parts(
                    ptr as *mut u8,
                    self.ptr.as_ref().savedStateSize,
                ))
            }
        }
    }

    pub fn content_rect(&self) -> ffi::ARect {
        unsafe { self.ptr.as_ref().contentRect }
    }

    // The looper will also never change
    pub fn looper(&self) -> ForeignLooper {
        unsafe { ForeignLooper::from_ptr(NonNull::new(self.ptr.as_ref().looper).unwrap()) }
    }

    // TODO: all the other things
}

// TODO docs
// Best thing would be to word-for-word copy the docs from android_native_app_glue.h to here,
// because there's really no good online source for it
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(i8)]
pub enum Cmd {
    InputChanged = ffi::native_app_glue::APP_CMD_INPUT_CHANGED,
    InitWindow = ffi::native_app_glue::APP_CMD_INIT_WINDOW,
    TermWindow = ffi::native_app_glue::APP_CMD_TERM_WINDOW,
    WindowResized = ffi::native_app_glue::APP_CMD_WINDOW_RESIZED,
    WindowRedrawNeeded = ffi::native_app_glue::APP_CMD_WINDOW_REDRAW_NEEDED,
    ContentRectChanged = ffi::native_app_glue::APP_CMD_CONTENT_RECT_CHANGED,
    GainedFocus = ffi::native_app_glue::APP_CMD_GAINED_FOCUS,
    LostFocus = ffi::native_app_glue::APP_CMD_LOST_FOCUS,
    ConfigChanged = ffi::native_app_glue::APP_CMD_CONFIG_CHANGED,
    LowMemory = ffi::native_app_glue::APP_CMD_LOW_MEMORY,
    Start = ffi::native_app_glue::APP_CMD_START,
    Resume = ffi::native_app_glue::APP_CMD_RESUME,
    SaveState = ffi::native_app_glue::APP_CMD_SAVE_STATE,
    Pause = ffi::native_app_glue::APP_CMD_PAUSE,
    Stop = ffi::native_app_glue::APP_CMD_STOP,
    Destroy = ffi::native_app_glue::APP_CMD_DESTROY,
}

/// A wrapper that associates a lifetime with data.
///
/// This is used to ensure that data associated with an `AndroidApp` doesn't outlive its container.
pub struct Ref<'a, T> {
    _marker: PhantomData<&'a T>,
    data: ManuallyDrop<T>,
}

impl<'a, T: fmt::Debug> fmt::Debug for Ref<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ref {{ {:?} }}", &*self)
    }
}

impl<'a, T> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.data
    }
}

impl<'a, T> Ref<'a, T> {
    fn new(data: T) -> Self {
        Self {
            _marker: PhantomData,
            data: ManuallyDrop::new(data),
        }
    }
}
