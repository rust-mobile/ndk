use lazy_static::lazy_static;
use log::Level;
use ndk::input_queue::InputQueue;
use ndk::looper::{FdEvent, ForeignLooper, ThreadLooper};
use ndk::native_activity::NativeActivity;
use ndk::native_window::NativeWindow;
use ndk_sys::{AInputQueue, ANativeActivity, ANativeWindow, ARect};
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::raw;
use std::os::unix::prelude::*;
use std::ptr::NonNull;
use std::sync::{Arc, Condvar, Mutex, RwLock, RwLockReadGuard};
use std::thread;

#[cfg(feature = "logger")]
pub use android_logger;
#[cfg(feature = "logger")]
pub use log;

pub use ndk_macro::main;

/// `ndk-glue` macros register the reading end of an event pipe with the
/// main [`ThreadLooper`] under this `ident`.
/// When returned from [`ThreadLooper::poll_*`](ThreadLooper::poll_once)
/// an event can be retrieved from [`poll_events()`].
pub const NDK_GLUE_LOOPER_EVENT_PIPE_IDENT: i32 = 0;

/// The [`InputQueue`] received from Android is registered with the main
/// [`ThreadLooper`] under this `ident`.
/// When returned from [`ThreadLooper::poll_*`](ThreadLooper::poll_once)
/// an event can be retrieved from [`input_queue()`].
pub const NDK_GLUE_LOOPER_INPUT_QUEUE_IDENT: i32 = 1;

pub fn android_log(level: Level, tag: &CStr, msg: &CStr) {
    let prio = match level {
        Level::Error => ndk_sys::android_LogPriority_ANDROID_LOG_ERROR,
        Level::Warn => ndk_sys::android_LogPriority_ANDROID_LOG_WARN,
        Level::Info => ndk_sys::android_LogPriority_ANDROID_LOG_INFO,
        Level::Debug => ndk_sys::android_LogPriority_ANDROID_LOG_DEBUG,
        Level::Trace => ndk_sys::android_LogPriority_ANDROID_LOG_VERBOSE,
    };
    unsafe {
        ndk_sys::__android_log_write(prio as raw::c_int, tag.as_ptr(), msg.as_ptr());
    }
}

lazy_static! {
    static ref NATIVE_WINDOW: RwLock<Option<NativeWindow>> = Default::default();
    static ref INPUT_QUEUE: RwLock<Option<InputQueue>> = Default::default();
    static ref CONTENT_RECT: RwLock<Rect> = Default::default();
    static ref LOOPER: Mutex<Option<ForeignLooper>> = Default::default();
}

static mut NATIVE_ACTIVITY: Option<NativeActivity> = None;

#[deprecated = "Use `ndk_context::android_context().vm()` instead."]
pub fn native_activity() -> &'static NativeActivity {
    unsafe { NATIVE_ACTIVITY.as_ref().unwrap() }
}

pub fn native_window() -> RwLockReadGuard<'static, Option<NativeWindow>> {
    NATIVE_WINDOW.read().unwrap()
}

pub fn input_queue() -> RwLockReadGuard<'static, Option<InputQueue>> {
    INPUT_QUEUE.read().unwrap()
}

pub fn content_rect() -> Rect {
    CONTENT_RECT.read().unwrap().clone()
}

lazy_static! {
    static ref PIPE: [RawFd; 2] = {
        let mut pipe: [RawFd; 2] = Default::default();
        unsafe { libc::pipe(pipe.as_mut_ptr()) };
        pipe
    };
}

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
    WindowCreated,
    WindowResized,
    WindowRedrawNeeded,
    /// If the window is in use by ie. a graphics API, make sure the lock from
    /// [`native_window()`] is held on to until after freeing those resources.
    ///
    /// After receiving this [`Event`] `ndk_glue` will block until that read-lock
    /// is released before returning to Android and allowing it to free up the window.
    WindowDestroyed,
    InputQueueCreated,
    /// After receiving this [`Event`] `ndk_glue` will block until the read-lock from
    /// [`input_queue()`] is released before returning to Android and allowing it to
    /// free up the input queue.
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
        .unwrap()
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
    let mut native_window_guard = NATIVE_WINDOW.write().unwrap();
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
    INPUT_QUEUE.write().unwrap().replace(input_queue);
    wake(activity, Event::InputQueueCreated);
}

unsafe extern "C" fn on_input_queue_destroyed(
    activity: *mut ANativeActivity,
    queue: *mut AInputQueue,
) {
    wake(activity, Event::InputQueueDestroyed);
    let mut input_queue_guard = INPUT_QUEUE.write().unwrap();
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
    *CONTENT_RECT.write().unwrap() = rect;
    wake(activity, Event::ContentRectChanged);
}
