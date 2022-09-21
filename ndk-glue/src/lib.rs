#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use log::Level;
use ndk::input_queue::InputQueue;
use ndk::looper::{FdEvent, ForeignLooper, ThreadLooper};
use ndk::native_activity::NativeActivity;
use ndk::native_window::NativeWindow;
use ndk_sys::{AInputQueue, ANativeActivity, ANativeWindow, ARect};
use once_cell::sync::Lazy;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use std::ffi::{CStr, CString};
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Deref;
use std::os::raw;
use std::os::unix::prelude::*;
use std::ptr::NonNull;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

#[cfg(feature = "logger")]
pub use android_logger;
#[cfg(feature = "logger")]
pub use log;

pub use ndk_macro::main;

/// `ndk-glue` macros register the reading end of an event pipe with the
/// main [`ThreadLooper`] under this `ident`.
/// When returned from [`ThreadLooper::poll_*`][ThreadLooper::poll_once]
/// an event can be retrieved from [`poll_events()`].
pub const NDK_GLUE_LOOPER_EVENT_PIPE_IDENT: i32 = 0;

/// The [`InputQueue`] received from Android is registered with the main
/// [`ThreadLooper`] under this `ident`.
/// When returned from [`ThreadLooper::poll_*`][ThreadLooper::poll_once]
/// an event can be retrieved from [`input_queue()`].
pub const NDK_GLUE_LOOPER_INPUT_QUEUE_IDENT: i32 = 1;

pub fn android_log(level: Level, tag: &CStr, msg: &CStr) {
    let prio = match level {
        Level::Error => ndk_sys::android_LogPriority::ANDROID_LOG_ERROR,
        Level::Warn => ndk_sys::android_LogPriority::ANDROID_LOG_WARN,
        Level::Info => ndk_sys::android_LogPriority::ANDROID_LOG_INFO,
        Level::Debug => ndk_sys::android_LogPriority::ANDROID_LOG_DEBUG,
        Level::Trace => ndk_sys::android_LogPriority::ANDROID_LOG_VERBOSE,
    };
    unsafe {
        ndk_sys::__android_log_write(prio.0 as raw::c_int, tag.as_ptr(), msg.as_ptr());
    }
}

static NATIVE_WINDOW: Lazy<RwLock<Option<NativeWindow>>> = Lazy::new(Default::default);
static INPUT_QUEUE: Lazy<RwLock<Option<InputQueue>>> = Lazy::new(Default::default);
static CONTENT_RECT: Lazy<RwLock<Rect>> = Lazy::new(Default::default);
static LOOPER: Lazy<Mutex<Option<ForeignLooper>>> = Lazy::new(Default::default);

static mut NATIVE_ACTIVITY: Option<NativeActivity> = None;

/// This function accesses a `static` variable internally and must only be used if you are sure
/// there is exactly one version of `ndk_glue` in your dependency tree.
///
/// If you need access to the `JavaVM` through [`NativeActivity::vm()`] or Activity `Context`
/// through [`NativeActivity::activity()`], please use the [`ndk_context`] crate and its
/// [`ndk_context::android_context()`] getter to acquire the `JavaVM` and `Context` instead.
pub fn native_activity() -> &'static NativeActivity {
    unsafe { NATIVE_ACTIVITY.as_ref().unwrap() }
}

pub struct LockReadGuard<T: ?Sized + 'static>(MappedRwLockReadGuard<'static, T>);

impl<T> LockReadGuard<T> {
    /// Transpose an [`Option`] wrapped inside a [`LockReadGuard`]
    ///
    /// This is a _read_ lock for which the contents can't change; hence allowing the user to only
    /// check for [`None`] once and hold a lock containing `T` directly thereafter, without
    /// subsequent infallible [`Option::unwrap()`]s.
    fn from_wrapped_option(wrapped: RwLockReadGuard<'static, Option<T>>) -> Option<Self> {
        RwLockReadGuard::try_map(wrapped, Option::as_ref)
            .ok()
            .map(Self)
    }
}

impl<T: ?Sized> Deref for LockReadGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for LockReadGuard<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for LockReadGuard<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Returns a [`NativeWindow`] held inside a lock, preventing Android from freeing it immediately
/// in [its `NativeWindow` destructor].
///
/// If the window is in use by e.g. a graphics API, make sure to hold on to this lock.
///
/// After receiving [`Event::WindowDestroyed`] `ndk-glue` will block in Android's [`NativeWindow`] destructor
/// callback until the lock is released, returning to Android and allowing it to free the window.
///
/// [its `NativeWindow` destructor]: https://developer.android.com/ndk/reference/struct/a-native-activity-callbacks#onnativewindowdestroyed
///
/// # Warning
/// This function accesses a `static` variable internally and must only be used if you are sure
/// there is exactly one version of `ndk_glue` in your dependency tree.
pub fn native_window() -> Option<LockReadGuard<NativeWindow>> {
    LockReadGuard::from_wrapped_option(NATIVE_WINDOW.read())
}

/// Returns an [`InputQueue`] held inside a lock, preventing Android from freeing it immediately
/// in [its `InputQueue` destructor].
///
/// After receiving [`Event::InputQueueDestroyed`] `ndk-glue` will block in Android's [`InputQueue`] destructor
/// callback until the lock is released, returning to Android and allowing it to free the window.
///
/// [its `InputQueue` destructor]: https://developer.android.com/ndk/reference/struct/a-native-activity-callbacks#oninputqueuedestroyed
///
/// # Warning
/// This function accesses a `static` variable internally and must only be used if you are sure
/// there is exactly one version of `ndk_glue` in your dependency tree.
pub fn input_queue() -> Option<LockReadGuard<InputQueue>> {
    LockReadGuard::from_wrapped_option(INPUT_QUEUE.read())
}

/// This function accesses a `static` variable internally and must only be used if you are sure
/// there is exactly one version of `ndk_glue` in your dependency tree.
pub fn content_rect() -> Rect {
    CONTENT_RECT.read().clone()
}

static PIPE: Lazy<[RawFd; 2]> = Lazy::new(|| {
    let mut pipe: [RawFd; 2] = Default::default();
    unsafe { libc::pipe(pipe.as_mut_ptr()) };
    pipe
});

pub fn poll_events() -> Option<Event> {
    unsafe {
        let size = std::mem::size_of::<Event>();
        let mut event = Event::Start;
        if libc::read(PIPE[0], &mut event as *mut _ as *mut _, size) == size as _ {
            Some(event)
        } else {
            None
        }
    }
}

unsafe fn wake(_activity: *mut ANativeActivity, event: Event) {
    log::trace!("{:?}", event);
    let size = std::mem::size_of::<Event>();
    let res = libc::write(PIPE[1], &event as *const _ as *const _, size);
    assert_eq!(res, size as _);
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Rect {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Event {
    Start,
    Resume,
    SaveInstanceState,
    Pause,
    Stop,
    Destroy,
    ConfigChanged,
    LowMemory,
    WindowLostFocus,
    WindowHasFocus,
    /// A [`NativeWindow`] is now available through [`native_window()`]. See that function for more
    /// details about holding on to the returned [`LockReadGuard`].
    ///
    /// Be sure to release any resources (e.g. Vulkan/OpenGL graphics surfaces) created from
    /// it followed by releasing this lock upon receiving [`Event::WindowDestroyed`].
    WindowCreated,
    WindowResized,
    WindowRedrawNeeded,
    /// If the window is in use by e.g. a graphics API, make sure the [`LockReadGuard`] from
    /// [`native_window()`] is held on to until after freeing those resources.
    ///
    /// After receiving this [`Event`] `ndk_glue` will block inside its [`NativeWindow`] destructor
    /// until that read-lock is released before returning to Android and allowing it to free the
    /// window.
    ///
    /// From this point [`native_window()`] will return [`None`] until receiving
    /// [`Event::WindowCreated`] again.
    WindowDestroyed,
    /// An [`InputQueue`] is now available through [`input_queue()`].
    ///
    /// Be sure to release the returned lock upon receiving [`Event::InputQueueDestroyed`].
    InputQueueCreated,
    /// After receiving this [`Event`] `ndk_glue` will block inside its [`InputQueue`] destructor
    /// until the read-lock from [`input_queue()`] is released before returning to Android and
    /// allowing it to free the input queue.
    ///
    /// From this point [`input_queue()`] will return [`None`] until receiving
    /// [`Event::InputQueueCreated`] again.
    InputQueueDestroyed,
    ContentRectChanged,
}

/// # Safety
/// `activity` must either be null (resulting in a safe panic)
/// or a pointer to a valid Android `ANativeActivity`.
pub unsafe fn init(
    activity: *mut ANativeActivity,
    _saved_state: *mut u8,
    _saved_state_size: usize,
    main: fn(),
) {
    let mut activity = NonNull::new(activity).unwrap();
    let mut callbacks = activity.as_mut().callbacks.as_mut().unwrap();
    callbacks.onStart = Some(on_start);
    callbacks.onResume = Some(on_resume);
    callbacks.onSaveInstanceState = Some(on_save_instance_state);
    callbacks.onPause = Some(on_pause);
    callbacks.onStop = Some(on_stop);
    callbacks.onDestroy = Some(on_destroy);
    callbacks.onWindowFocusChanged = Some(on_window_focus_changed);
    callbacks.onNativeWindowCreated = Some(on_window_created);
    callbacks.onNativeWindowResized = Some(on_window_resized);
    callbacks.onNativeWindowRedrawNeeded = Some(on_window_redraw_needed);
    callbacks.onNativeWindowDestroyed = Some(on_window_destroyed);
    callbacks.onInputQueueCreated = Some(on_input_queue_created);
    callbacks.onInputQueueDestroyed = Some(on_input_queue_destroyed);
    callbacks.onContentRectChanged = Some(on_content_rect_changed);
    callbacks.onConfigurationChanged = Some(on_configuration_changed);
    callbacks.onLowMemory = Some(on_low_memory);

    let activity = NativeActivity::from_ptr(activity);
    ndk_context::initialize_android_context(activity.vm().cast(), activity.activity().cast());
    NATIVE_ACTIVITY.replace(activity);

    let mut logpipe: [RawFd; 2] = Default::default();
    libc::pipe(logpipe.as_mut_ptr());
    libc::dup2(logpipe[1], libc::STDOUT_FILENO);
    libc::dup2(logpipe[1], libc::STDERR_FILENO);
    thread::spawn(move || {
        let tag = CStr::from_bytes_with_nul(b"RustStdoutStderr\0").unwrap();
        let file = File::from_raw_fd(logpipe[0]);
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        loop {
            buffer.clear();
            if let Ok(len) = reader.read_line(&mut buffer) {
                if len == 0 {
                    break;
                } else if let Ok(msg) = CString::new(buffer.clone()) {
                    android_log(Level::Info, tag, &msg);
                }
            }
        }
    });

    let looper_ready = Arc::new(Condvar::new());
    let signal_looper_ready = looper_ready.clone();

    thread::spawn(move || {
        let looper = ThreadLooper::prepare();
        let foreign = looper.into_foreign();
        foreign
            .add_fd(
                PIPE[0],
                NDK_GLUE_LOOPER_EVENT_PIPE_IDENT,
                FdEvent::INPUT,
                std::ptr::null_mut(),
            )
            .unwrap();

        {
            let mut locked_looper = LOOPER.lock().unwrap();
            locked_looper.replace(foreign);
            signal_looper_ready.notify_one();
        }

        main()
    });

    // Don't return from this function (`ANativeActivity_onCreate`) until the thread
    // has created its `ThreadLooper` and assigned it to the static `LOOPER`
    // variable. It will be used from `on_input_queue_created` as soon as this
    // function returns.
    let locked_looper = LOOPER.lock().unwrap();
    let _mutex_guard = looper_ready
        .wait_while(locked_looper, |looper| looper.is_none())
        .unwrap();
}

unsafe extern "C" fn on_start(activity: *mut ANativeActivity) {
    wake(activity, Event::Start);
}

unsafe extern "C" fn on_resume(activity: *mut ANativeActivity) {
    wake(activity, Event::Resume);
}

unsafe extern "C" fn on_save_instance_state(
    activity: *mut ANativeActivity,
    _out_size: *mut ndk_sys::size_t,
) -> *mut raw::c_void {
    // TODO
    wake(activity, Event::SaveInstanceState);
    std::ptr::null_mut()
}

unsafe extern "C" fn on_pause(activity: *mut ANativeActivity) {
    wake(activity, Event::Pause);
}

unsafe extern "C" fn on_stop(activity: *mut ANativeActivity) {
    wake(activity, Event::Stop);
}

unsafe extern "C" fn on_destroy(activity: *mut ANativeActivity) {
    wake(activity, Event::Destroy);
    ndk_context::release_android_context();
}

unsafe extern "C" fn on_configuration_changed(activity: *mut ANativeActivity) {
    wake(activity, Event::ConfigChanged);
}

unsafe extern "C" fn on_low_memory(activity: *mut ANativeActivity) {
    wake(activity, Event::LowMemory);
}

unsafe extern "C" fn on_window_focus_changed(
    activity: *mut ANativeActivity,
    has_focus: raw::c_int,
) {
    let event = if has_focus == 0 {
        Event::WindowLostFocus
    } else {
        Event::WindowHasFocus
    };
    wake(activity, event);
}

unsafe extern "C" fn on_window_created(activity: *mut ANativeActivity, window: *mut ANativeWindow) {
    NATIVE_WINDOW
        .write()
        .replace(NativeWindow::clone_from_ptr(NonNull::new(window).unwrap()));
    wake(activity, Event::WindowCreated);
}

unsafe extern "C" fn on_window_resized(
    activity: *mut ANativeActivity,
    _window: *mut ANativeWindow,
) {
    wake(activity, Event::WindowResized);
}

unsafe extern "C" fn on_window_redraw_needed(
    activity: *mut ANativeActivity,
    _window: *mut ANativeWindow,
) {
    wake(activity, Event::WindowRedrawNeeded);
}

unsafe extern "C" fn on_window_destroyed(
    activity: *mut ANativeActivity,
    window: *mut ANativeWindow,
) {
    wake(activity, Event::WindowDestroyed);
    let mut native_window_guard = NATIVE_WINDOW.write();
    assert_eq!(native_window_guard.as_ref().unwrap().ptr().as_ptr(), window);
    native_window_guard.take();
}

unsafe extern "C" fn on_input_queue_created(
    activity: *mut ANativeActivity,
    queue: *mut AInputQueue,
) {
    let input_queue = InputQueue::from_ptr(NonNull::new(queue).unwrap());
    let locked_looper = LOOPER.lock().unwrap();
    // The looper should always be `Some` after `fn init()` returns, unless
    // future code cleans it up and sets it back to `None` again.
    let looper = locked_looper.as_ref().expect("Looper does not exist");
    input_queue.attach_looper(looper, NDK_GLUE_LOOPER_INPUT_QUEUE_IDENT);
    INPUT_QUEUE.write().replace(input_queue);
    wake(activity, Event::InputQueueCreated);
}

unsafe extern "C" fn on_input_queue_destroyed(
    activity: *mut ANativeActivity,
    queue: *mut AInputQueue,
) {
    wake(activity, Event::InputQueueDestroyed);
    let mut input_queue_guard = INPUT_QUEUE.write();
    assert_eq!(input_queue_guard.as_ref().unwrap().ptr().as_ptr(), queue);
    let input_queue = InputQueue::from_ptr(NonNull::new(queue).unwrap());
    input_queue.detach_looper();
    input_queue_guard.take();
}

unsafe extern "C" fn on_content_rect_changed(activity: *mut ANativeActivity, rect: *const ARect) {
    let rect = Rect {
        left: (*rect).left as _,
        top: (*rect).top as _,
        right: (*rect).right as _,
        bottom: (*rect).bottom as _,
    };
    *CONTENT_RECT.write() = rect;
    wake(activity, Event::ContentRectChanged);
}
