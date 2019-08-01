//! Bindings to the NDK's `android_native_app_glue.c`

use super::{
    AConfiguration, AInputEvent, AInputQueue, ALooper, ANativeActivity, ANativeWindow, ARect,
};
use std::os::raw::{c_int, c_void};

#[repr(C)]
pub struct android_app {
    pub userData: *mut c_void,
    pub onAppCmd: extern "C" fn(*mut android_app, i32),
    pub onInputEvent: extern "C" fn(*mut android_app, *const AInputEvent) -> i32,
    pub activity: *mut ANativeActivity,
    pub config: *mut AConfiguration,
    pub savedState: *mut c_void,
    pub savedStateSize: usize,
    pub looper: *mut ALooper,
    pub inputQueue: *mut AInputQueue,
    pub window: *mut ANativeWindow,
    pub contentRect: ARect,
    pub activityState: c_int,
    pub destroyRequested: c_int,
}

#[repr(C)]
pub struct android_poll_source {
    pub id: i32, // can be LOOPER_ID_MAIN, LOOPER_ID_INPUT or LOOPER_ID_USER
    pub app: *mut android_app,
    pub process: extern "C" fn(*mut android_app, *mut android_poll_source),
}

pub const LOOPER_ID_MAIN: i32 = 1;
pub const LOOPER_ID_INPUT: i32 = 2;
pub const LOOPER_ID_USER: i32 = 3;

pub const APP_CMD_INPUT_CHANGED: i8 = 0;
pub const APP_CMD_INIT_WINDOW: i8 = 1;
pub const APP_CMD_TERM_WINDOW: i8 = 2;
pub const APP_CMD_WINDOW_RESIZED: i8 = 3;
pub const APP_CMD_WINDOW_REDRAW_NEEDED: i8 = 4;
pub const APP_CMD_CONTENT_RECT_CHANGED: i8 = 5;
pub const APP_CMD_GAINED_FOCUS: i8 = 6;
pub const APP_CMD_LOST_FOCUS: i8 = 7;
pub const APP_CMD_CONFIG_CHANGED: i8 = 8;
pub const APP_CMD_LOW_MEMORY: i8 = 9;
pub const APP_CMD_START: i8 = 10;
pub const APP_CMD_RESUME: i8 = 11;
pub const APP_CMD_SAVE_STATE: i8 = 12;
pub const APP_CMD_PAUSE: i8 = 13;
pub const APP_CMD_STOP: i8 = 14;
pub const APP_CMD_DESTROY: i8 = 15;

extern "C" {
    pub fn android_app_read_cmd(app: *mut android_app) -> i8;
    pub fn android_app_pre_exec_cmd(app: *mut android_app, cmd: i8);
    pub fn android_app_post_exec_cmd(app: *mut android_app, cmd: i8);
}
