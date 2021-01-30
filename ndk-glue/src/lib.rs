use crossbeam_queue::SegQueue;
use lazy_static::lazy_static;
use log::Level;
use ndk::input_queue::InputQueue;
use ndk::looper::{ForeignLooper, LooperError, ThreadLooper};
use ndk::native_activity::NativeActivity;
use ndk::native_window::NativeWindow;
use ndk_sys::{AInputQueue, ANativeActivity, ANativeWindow, ARect, ALOOPER_EVENT_INPUT};
use std::ffi::{c_void, CStr, CString};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::raw;
use std::os::unix::prelude::*;
use std::ptr::NonNull;
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex, Weak,
};
use std::thread;

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

#[derive(Debug)]
pub struct EventPayload<T: 'static>(Option<&'static Mutex<Option<T>>>);

impl<T: 'static> EventPayload<T> {
    fn new(val: &'static Mutex<Option<T>>) -> Self {
        Self(Some(val))
    }

    pub fn deliver(mut self) -> Option<T> {
        let mutex = self.0.take().unwrap();
        mutex.lock().unwrap().take()
    }
}

impl<T: 'static> Drop for EventPayload<T> {
    fn drop(&mut self) {
        // Even if the user did not take this resource, we remove it from the
        // static `Mutex<Option<_>>`, to mark it as delivered.
        self.0.map(|mutex| mutex.lock().unwrap().take());
    }
}

#[derive(Debug)]
pub struct SyncEventGuard(Sender<()>);

impl SyncEventGuard {
    fn new() -> (Self, Receiver<()>) {
        let (tx, rx) = channel();
        (Self(tx), rx)
    }
}

impl Drop for SyncEventGuard {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}

pub mod ident {
    pub const ACTIVITY_CALLBACK: i32 = 0;
    pub const INPUT_QUEUE: i32 = 1;
    pub const USER: i32 = 2;
}

#[derive(Debug)]
struct EventFd {
    raw_fd: RawFd,
}

impl EventFd {
    pub fn new(init: libc::c_uint, flag: libc::c_int) -> Result<Self, std::io::Error> {
        let raw_fd = unsafe { libc::eventfd(init, flag) };
        if raw_fd < 0i32 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(Self { raw_fd })
        }
    }

    #[inline]
    pub fn to_raw(&self) -> RawFd {
        self.raw_fd
    }

    #[inline]
    pub fn write(&self, n: u64) -> Result<(), std::io::Error> {
        let size = std::mem::size_of::<u64>();
        let ptr = &n as *const _ as *const _;
        match unsafe { libc::write(self.raw_fd, ptr, size) } {
            8isize => Ok(()),
            _ => Err(std::io::Error::last_os_error()),
        }
    }

    #[inline]
    pub fn read(&self) -> Result<Option<u64>, std::io::Error> {
        let mut n = 0u64;
        let ptr = &mut n as *mut _ as *mut _;
        let size = std::mem::size_of::<u64>();
        match unsafe { libc::read(self.raw_fd, ptr, size) } {
            8isize => Ok(Some(n)),
            _ => {
                let err = std::io::Error::last_os_error();
                match err.raw_os_error().unwrap() {
                    libc::EAGAIN => Ok(None),
                    _ => Err(err),
                }
            }
        }
    }

    pub fn add_looper(&self, looper: &ForeignLooper, ident: i32) -> Result<(), LooperError> {
        unsafe {
            looper.add_fd(
                self.raw_fd,
                ident,
                ALOOPER_EVENT_INPUT as _,
                std::ptr::null_mut::<c_void>(),
            )
        }
    }
}

impl Drop for EventFd {
    fn drop(&mut self) {
        let _ = unsafe { libc::close(self.raw_fd) };
    }
}

unsafe impl Send for EventFd {}
unsafe impl Sync for EventFd {}

// Unbounded eventfd based mpsc
// MUST be attached to the LOOPER
// Does not care if all Senders are gone,
// but tries to prevent sends to a dropped Receiver
// and to drop all outstanding events on Receiver drop
#[derive(Debug)]
struct EventQueueInner<E> {
    event_fd: EventFd,
    queue: SegQueue<E>,
}

impl<E> EventQueueInner<E> {
    fn new() -> Self {
        Self {
            event_fd: EventFd::new(0u32, libc::EFD_SEMAPHORE | libc::EFD_NONBLOCK)
                .expect("Could not open eventfd for ndk_glue::event_queue, cannot proceed"),
            queue: SegQueue::new(),
        }
    }
}

#[derive(Debug)]
pub struct EventQueueSender<E>(Weak<EventQueueInner<E>>);

impl<E> Clone for EventQueueSender<E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<E> EventQueueSender<E> {
    pub fn send(&self, event: E) -> Result<(), E> {
        match self.0.upgrade() {
            Some(inner) => {
                inner.queue.push(event);
                inner.event_fd.write(1).unwrap();
                Ok(())
            }
            None => Err(event),
        }
    }
}

#[derive(Debug)]
pub struct EventQueueReceiver<E>(Arc<EventQueueInner<E>>);

impl<E> EventQueueReceiver<E> {
    pub fn try_recv(&self) -> Option<E> {
        let _ = self.0.event_fd.read().unwrap()?;
        self.0.queue.pop()
    }
}

impl<E> Drop for EventQueueReceiver<E> {
    fn drop(&mut self) {
        unsafe {
            let _ = LOOPER.as_ref().unwrap().remove_fd(self.0.event_fd.to_raw());
        }
    }
}

// Unsafe because caller MUST ensure that the eventfd is attached to LOOPER
// and that the Receiver does not drop before then.
unsafe fn event_queue_impl<E>() -> (EventQueueSender<E>, EventQueueReceiver<E>) {
    let inner = Arc::new(EventQueueInner::new());
    let weak_inner = Arc::downgrade(&inner);
    (EventQueueSender(weak_inner), EventQueueReceiver(inner))
}

pub fn event_queue<E>(ident: i32) -> (EventQueueSender<E>, EventQueueReceiver<E>) {
    let (tx, rx) = unsafe { event_queue_impl() };
    unsafe {
        rx.0.event_fd
            .add_looper(LOOPER.as_ref().unwrap(), ident)
            .unwrap();
    }
    (tx, rx)
}

lazy_static! {
    static ref NATIVE_WINDOW: Mutex<Option<NativeWindow>> = Default::default();
    static ref INPUT_QUEUE: Mutex<Option<InputQueue>> = Default::default();
    static ref CONTENT_RECT: Mutex<Option<Rect>> = Mutex::new(Some(Default::default()));
}

static mut NATIVE_ACTIVITY: Option<NativeActivity> = None;
static mut LOOPER: Option<ForeignLooper> = None;
static mut SENDER: Option<EventQueueSender<Event>> = None;
static mut RECEIVER: Option<EventQueueReceiver<Event>> = None;

pub fn native_activity() -> &'static NativeActivity {
    unsafe { NATIVE_ACTIVITY.as_ref().unwrap() }
}

pub fn activity_event_rx() -> &'static EventQueueReceiver<Event> {
    unsafe { RECEIVER.as_ref().unwrap() }
}

pub fn request_window_redraw() {
    unsafe {
        SENDER
            .as_ref()
            .unwrap()
            .send(Event::WindowRedrawNeeded)
            .unwrap()
    };
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Rect {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

#[derive(Debug)]
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
    WindowCreated(EventPayload<NativeWindow>),
    WindowResized,
    WindowRedrawNeeded,
    // Will be sent if and only if `WindowCreated` payload was not `None`
    WindowDestroyed(SyncEventGuard),
    InputQueueCreated(EventPayload<InputQueue>),
    // Will be sent if and only if `InputQueueCreated` payload was not `None` 
    InputQueueDestroyed(SyncEventGuard), 
    ContentRectChanged(EventPayload<Rect>),
}

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
    NATIVE_ACTIVITY = Some(activity);

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

    let (tx, rx) = event_queue_impl();
    SENDER = Some(tx);
    RECEIVER = Some(rx);
    thread::spawn(move || {
        let looper = ThreadLooper::prepare();
        let foreign = looper.into_foreign();
        RECEIVER
            .as_ref()
            .unwrap()
            .0
            .event_fd
            .add_looper(&foreign, ident::ACTIVITY_CALLBACK)
            .unwrap();
        LOOPER = Some(foreign);
        main()
    });
}

unsafe extern "C" fn on_start(_activity: *mut ANativeActivity) {
    SENDER.as_ref().unwrap().send(Event::Start).unwrap();
}

unsafe extern "C" fn on_resume(_activity: *mut ANativeActivity) {
    SENDER.as_ref().unwrap().send(Event::Resume).unwrap();
}

unsafe extern "C" fn on_save_instance_state(
    _activity: *mut ANativeActivity,
    _out_size: *mut ndk_sys::size_t,
) -> *mut raw::c_void {
    // TODO
    SENDER
        .as_ref()
        .unwrap()
        .send(Event::SaveInstanceState)
        .unwrap();
    std::ptr::null_mut()
}

unsafe extern "C" fn on_pause(_activity: *mut ANativeActivity) {
    SENDER.as_ref().unwrap().send(Event::Pause).unwrap();
}

unsafe extern "C" fn on_stop(_activity: *mut ANativeActivity) {
    SENDER.as_ref().unwrap().send(Event::Stop).unwrap();
}

unsafe extern "C" fn on_destroy(_activity: *mut ANativeActivity) {
    SENDER.as_ref().unwrap().send(Event::Destroy).unwrap();
}

unsafe extern "C" fn on_configuration_changed(_activity: *mut ANativeActivity) {
    SENDER.as_ref().unwrap().send(Event::ConfigChanged).unwrap();
}

unsafe extern "C" fn on_low_memory(_activity: *mut ANativeActivity) {
    SENDER.as_ref().unwrap().send(Event::LowMemory).unwrap();
}

unsafe extern "C" fn on_window_focus_changed(
    _activity: *mut ANativeActivity,
    has_focus: raw::c_int,
) {
    let event = if has_focus == 0 {
        Event::WindowLostFocus
    } else {
        Event::WindowHasFocus
    };
    SENDER.as_ref().unwrap().send(event).unwrap();
}

unsafe extern "C" fn on_window_created(
    _activity: *mut ANativeActivity,
    window: *mut ANativeWindow,
) {
    *NATIVE_WINDOW.lock().unwrap() = Some(NativeWindow::from_ptr(NonNull::new(window).unwrap()));
    let event = Event::WindowCreated(EventPayload::new(&NATIVE_WINDOW));
    SENDER.as_ref().unwrap().send(event).unwrap();
}

unsafe extern "C" fn on_window_resized(
    _activity: *mut ANativeActivity,
    _window: *mut ANativeWindow,
) {
    SENDER.as_ref().unwrap().send(Event::WindowResized).unwrap();
}

unsafe extern "C" fn on_window_redraw_needed(
    _activity: *mut ANativeActivity,
    _window: *mut ANativeWindow,
) {
    SENDER
        .as_ref()
        .unwrap()
        .send(Event::WindowRedrawNeeded)
        .unwrap();
}

unsafe extern "C" fn on_window_destroyed(
    _activity: *mut ANativeActivity,
    _window: *mut ANativeWindow,
) {
    // If the window was delivered (WindowCreated event has already been handled)
    // then we need to send a destroyed event. Otherwise -- just preemptively
    // empty out the `Option<_>` which ensures the user won't see it.
    let was_delivered = NATIVE_WINDOW.lock().unwrap().take().is_none();
    if was_delivered {
        let (event_guard, rx) = SyncEventGuard::new();
        SENDER
            .as_ref()
            .unwrap()
            .send(Event::WindowDestroyed(event_guard))
            .unwrap();
        rx.recv().unwrap();
    }
}

unsafe extern "C" fn on_input_queue_created(
    _activity: *mut ANativeActivity,
    queue: *mut AInputQueue,
) {
    let input_queue = InputQueue::from_ptr(NonNull::new(queue).unwrap());
    let looper = LOOPER.as_ref().unwrap();
    input_queue.attach_looper(looper, ident::INPUT_QUEUE as _);
    *INPUT_QUEUE.lock().unwrap() = Some(input_queue);
    let event = Event::InputQueueCreated(EventPayload::new(&INPUT_QUEUE));
    SENDER.as_ref().unwrap().send(event).unwrap();
}

unsafe extern "C" fn on_input_queue_destroyed(
    _activity: *mut ANativeActivity,
    queue: *mut AInputQueue,
) {
    let input_queue = InputQueue::from_ptr(NonNull::new(queue).unwrap());
    input_queue.detach_looper();
    let was_delivered = INPUT_QUEUE.lock().unwrap().take().is_none();
    if was_delivered {
        let (event_guard, rx) = SyncEventGuard::new();
        SENDER
            .as_ref()
            .unwrap()
            .send(Event::InputQueueDestroyed(event_guard))
            .unwrap();
        rx.recv().unwrap();
    }
}

unsafe extern "C" fn on_content_rect_changed(_activity: *mut ANativeActivity, rect: *const ARect) {
    let rect = Rect {
        left: (*rect).left as _,
        top: (*rect).top as _,
        right: (*rect).right as _,
        bottom: (*rect).bottom as _,
    };
    *CONTENT_RECT.lock().unwrap() = Some(rect);
    let event = Event::ContentRectChanged(EventPayload::new(&CONTENT_RECT));
    SENDER.as_ref().unwrap().send(event).unwrap();
}
