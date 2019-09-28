//! Bindings for the `android_app` struct found in `android_native_app_glue.c`
//!
//! If you are not using `native_app_glue`, you can disable these bindings by disabling the
//! `native_app_glue` Cargo feature.

use crate::configuration::Configuration;
use crate::input_queue::InputQueue;
use crate::native_activity::NativeActivity;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::NonNull;

/// A `struct android_app *`.
pub struct AndroidApp {
    ptr: NonNull<ffi::native_app_glue::android_app>,
}

// TODO: docs
impl AndroidApp {
    pub unsafe fn from_ptr(ptr: NonNull<ffi::native_app_glue::android_app>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> NonNull<ffi::native_app_glue::android_app> {
        self.ptr
    }

    pub fn activity(&self) -> Ref<'_, NativeActivity> {
        unsafe {
            Ref::new(NativeActivity::from_ptr(
                NonNull::new(self.ptr.as_ref().activity).unwrap(),
            ))
        }
    }

    pub fn input_queue(&self) -> Ref<'_, InputQueue> {
        unsafe {
            Ref::new(InputQueue::from_ptr(
                NonNull::new(self.ptr.as_ref().inputQueue).unwrap(),
            ))
        }
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

    // TODO: all the other things
}

// TODO docs
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
