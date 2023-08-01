//! Internal utilities
use log::{error, log_enabled, Level};
use std::ffi::{c_int, CStr, CString};
use std::io::{Error, Result};

/// Turns standard `<errno.h>` status codes - typically rewrapped by Android's [`Errors.h`] - into
/// Rust's [`std::io::Error`].
///
/// [`Errors.h`]: https://cs.android.com/android/platform/superproject/+/master:system/core/libutils/include/utils/Errors.h
pub(crate) fn status_to_io_result<T>(status: i32, value: T) -> Result<T> {
    match status {
        0 => Ok(value),
        r if r < 0 => Err(Error::from_raw_os_error(-r)),
        r => unreachable!("Status is positive integer {}", r),
    }
}

pub(crate) fn android_log(level: Level, tag: &CStr, msg: &CStr) {
    let prio = match level {
        Level::Error => ffi::android_LogPriority::ANDROID_LOG_ERROR,
        Level::Warn => ffi::android_LogPriority::ANDROID_LOG_WARN,
        Level::Info => ffi::android_LogPriority::ANDROID_LOG_INFO,
        Level::Debug => ffi::android_LogPriority::ANDROID_LOG_DEBUG,
        Level::Trace => ffi::android_LogPriority::ANDROID_LOG_VERBOSE,
    };
    unsafe {
        ffi::__android_log_write(prio.0 as c_int, tag.as_ptr(), msg.as_ptr());
    }
}

pub(crate) fn log_panic(panic: Box<dyn std::any::Any + Send>) {
    let rust_panic = unsafe { CStr::from_bytes_with_nul_unchecked(b"RustPanic\0") };

    // Use the Rust logger if installed and enabled, otherwise fall back to the Android system
    // logger so there is at least some record of the panic
    let use_log = log_enabled!(Level::Error);

    match panic.downcast::<String>() {
        Ok(panic_string) => {
            if use_log {
                error!("RustPanic: {}", panic_string);
            } else if let Ok(msg) = CString::new(*panic_string) {
                android_log(Level::Error, rust_panic, &msg);
            }
        }
        Err(panic) => match panic.downcast::<&str>() {
            Ok(panic_str) => {
                if use_log {
                    error!("RustPanic: {}", panic_str);
                } else if let Ok(msg) = CString::new(*panic_str) {
                    android_log(Level::Error, rust_panic, &msg);
                }
            }
            Err(_) => {
                if use_log {
                    error!("UnknownPanic");
                } else {
                    let unknown_panic =
                        unsafe { CStr::from_bytes_with_nul_unchecked(b"UnknownPanic\0") };
                    android_log(Level::Error, unknown_panic, Default::default());
                }
            }
        },
    }

    if use_log {
        log::logger().flush();
    }
}

/// Run a closure and abort the program if it panics.
///
/// This is generally used to ensure Rust callbacks won't unwind past the FFI boundary, which leads
/// to undefined behaviour.
pub(crate) fn abort_on_panic<R>(f: impl FnOnce() -> R) -> R {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|panic| {
        // Try logging the panic before aborting
        //
        // Just in case our attempt to log a panic could itself cause a panic we use a
        // second catch_unwind here.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| log_panic(panic)));
        std::process::abort();
    })
}
