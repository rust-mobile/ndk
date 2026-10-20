#![cfg(target_os = "android")]

use ndk::looper::{FdEvent, Poll, ThreadLooper};
use std::{
    io::{Read, Write},
    os::{
        fd::{AsFd, AsRawFd},
        unix::net::UnixStream,
    },
    ptr,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

fn polling_rejected() -> bool {
    let Some(looper) = ThreadLooper::for_thread() else {
        return false;
    };
    let once = looper.poll_once_timeout(Duration::ZERO).is_err();
    let all = looper.poll_all_timeout(Duration::ZERO).is_err();
    looper.as_foreign().wake();
    let blocking_once = looper.poll_once().is_err();
    looper.as_foreign().wake();
    let blocking_all = looper.poll_all().is_err();
    once & all & blocking_once & blocking_all
}

struct PollOnDrop(Arc<AtomicBool>);

impl Drop for PollOnDrop {
    fn drop(&mut self) {
        self.0.store(polling_rejected(), Ordering::Relaxed);
    }
}

#[test]
fn callbacks_reject_reentrant_polling() {
    for poll in [ffi::ALooper_pollOnce, ffi::ALooper_pollAll] {
        let looper = ThreadLooper::prepare();
        let (reader, mut writer) = UnixStream::pair().unwrap();
        reader.set_nonblocking(true).unwrap();
        let mut callback_reader = reader.try_clone().unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let callback_calls = calls.clone();
        let dropped = Arc::new(AtomicBool::new(false));
        let drop_probe = PollOnDrop(dropped.clone());
        looper
            .add_fd_with_callback(reader.as_fd(), FdEvent::INPUT, move |_, _| {
                let _ = &drop_probe;
                callback_reader.read_exact(&mut [0]).unwrap();
                assert!(polling_rejected());
                callback_calls.fetch_add(1, Ordering::Relaxed) == 0
            })
            .unwrap();
        for expected in 1..=2 {
            writer.write_all(b"x").unwrap();
            // SAFETY: Live looper, open fds, current owner thread.
            let result = unsafe { poll(100, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) };
            assert!(matches!(
                result,
                ffi::ALOOPER_POLL_CALLBACK | ffi::ALOOPER_POLL_WAKE | ffi::ALOOPER_POLL_TIMEOUT
            ));
            assert_eq!(calls.load(Ordering::Relaxed), expected);
            assert_eq!(dropped.load(Ordering::Relaxed), expected == 2);
        }
        assert!(!looper.as_foreign().remove_fd(reader.as_fd()).unwrap());
        assert!(matches!(
            looper.poll_all_timeout(Duration::ZERO),
            Ok(Poll::Wake | Poll::Timeout)
        ));
    }
}

#[test]
fn polling_disabled_scopes_restore_state() {
    let looper = ThreadLooper::prepare();
    looper.with_polling_disabled(|| {
        assert!(polling_rejected());
        let nested = ThreadLooper::prepare();
        nested.with_polling_disabled(|| assert!(polling_rejected()));
        let panic = std::panic::catch_unwind(|| {
            nested.with_polling_disabled(|| panic!("scope unwind"));
        });
        assert!(panic.is_err());
        assert!(polling_rejected());
    });
    let dropped = Arc::new(AtomicBool::new(false));
    let drop_probe = PollOnDrop(dropped.clone());
    let panic = std::panic::catch_unwind(|| {
        looper.with_polling_disabled(move || {
            let _ = &drop_probe;
            panic!("outer scope unwind");
        });
    });
    assert!(panic.is_err());
    assert!(dropped.load(Ordering::Relaxed));
    assert!(matches!(
        looper.poll_all_timeout(Duration::ZERO),
        Ok(Poll::Wake)
    ));
}

#[test]
fn polling_disabled_scopes_are_thread_local() {
    let looper = ThreadLooper::prepare();
    looper.with_polling_disabled(|| {
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let independent = ThreadLooper::prepare();
            let result = matches!(
                independent.poll_once_timeout(Duration::ZERO),
                Ok(Poll::Timeout)
            );
            let _ = sender.send(("independent-looper", result));
        });
        assert_eq!(
            receiver.recv_timeout(Duration::from_secs(5)).unwrap(),
            ("independent-looper", true)
        );
        assert!(polling_rejected());
    });
}

#[test]
fn raw_native_callbacks_reject_reentrant_polling() {
    struct Callback {
        reader: UnixStream,
        rejected: bool,
    }

    extern "C" fn callback(_: i32, _: i32, data: *mut std::ffi::c_void) -> i32 {
        // SAFETY: Registration exclusively borrows the live stack state.
        let state = unsafe { &mut *data.cast::<Callback>() };
        state.rejected = state.reader.read_exact(&mut [0]).is_ok() && polling_rejected();
        0
    }

    for all in [false, true] {
        let looper = ThreadLooper::prepare();
        let (reader, mut writer) = UnixStream::pair().unwrap();
        reader.set_nonblocking(true).unwrap();
        let mut state = Callback {
            reader,
            rejected: false,
        };
        // SAFETY: State and `fd` outlive this self-unregistering callback.
        let added = unsafe {
            ffi::ALooper_addFd(
                looper.as_foreign().ptr().as_ptr(),
                state.reader.as_raw_fd(),
                ffi::ALOOPER_POLL_CALLBACK,
                i32::try_from(FdEvent::INPUT.bits()).unwrap(),
                Some(callback),
                ptr::addr_of_mut!(state).cast(),
            )
        };
        assert_eq!(added, 1);
        writer.write_all(b"x").unwrap();
        let result = if all {
            looper.poll_all_timeout(Duration::ZERO)
        } else {
            looper.poll_once_timeout(Duration::ZERO)
        };
        assert!(matches!(
            result,
            Ok(Poll::Callback | Poll::Wake | Poll::Timeout)
        ));
        assert!(state.rejected);
        assert!(!looper.as_foreign().remove_fd(state.reader.as_fd()).unwrap());
    }
}
