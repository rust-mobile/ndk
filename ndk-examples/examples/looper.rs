//! Demonstrates how to manage application lifetime using Android's `Looper`

use std::mem::MaybeUninit;
use std::os::unix::prelude::RawFd;
use std::time::Duration;

use log::info;
use ndk::event::{InputEvent, Keycode};
use ndk::looper::{FdEvent, Poll, ThreadLooper};

const U32_SIZE: usize = std::mem::size_of::<u32>();

#[cfg_attr(
    target_os = "android",
    ndk_glue::main(backtrace = "on", logger(level = "debug"))
)]
fn main() {
    // Retrieve the Looper that ndk_glue created for us on the current thread.
    // Android uses this to block on events and poll file descriptors with a single mechanism.
    let looper =
        ThreadLooper::for_thread().expect("ndk-glue did not attach thread looper before main()!");

    // First free number after ndk_glue::NDK_GLUE_LOOPER_INPUT_QUEUE_IDENT. This might be fragile.
    const CUSTOM_EVENT_IDENT: i32 = ndk_glue::NDK_GLUE_LOOPER_INPUT_QUEUE_IDENT + 1;

    fn create_pipe() -> [RawFd; 2] {
        let mut ends = MaybeUninit::<[RawFd; 2]>::uninit();
        assert_eq!(unsafe { libc::pipe(ends.as_mut_ptr().cast()) }, 0);
        unsafe { ends.assume_init() }
    }

    // Create a Unix pipe to send custom events to the Looper. ndk-glue uses a similar mechanism to deliver
    // ANativeActivityCallbacks asynchronously to the Looper through NDK_GLUE_LOOPER_EVENT_PIPE_IDENT.
    let custom_event_pipe = create_pipe();
    let custom_callback_pipe = create_pipe();

    // Attach the reading end of the pipe to the looper, so that it wakes up
    // whenever data is available for reading (FdEvent::INPUT)
    looper
        .as_foreign()
        .add_fd(
            custom_event_pipe[0],
            CUSTOM_EVENT_IDENT,
            FdEvent::INPUT,
            std::ptr::null_mut(),
        )
        .expect("Failed to add file descriptor to Looper");

    // Attach the reading end of a pipe to a callback, too
    looper
        .as_foreign()
        .add_fd_with_callback(custom_callback_pipe[0], FdEvent::INPUT, |fd| {
            let mut recv = !0u32;
            assert_eq!(
                unsafe { libc::read(fd, &mut recv as *mut _ as *mut _, U32_SIZE) } as usize,
                U32_SIZE
            );
            info!("Read custom event from pipe, in callback: {}", recv);
            // Detach this handler by returning `false` once the count reaches 5
            recv < 5
        })
        .expect("Failed to add file descriptor to Looper");

    std::thread::spawn(move || {
        // Send a "custom event" to the looper every second
        for i in 0.. {
            let i_addr = &i as *const _ as *const _;
            std::thread::sleep(Duration::from_secs(1));
            assert_eq!(
                unsafe { libc::write(custom_event_pipe[1], i_addr, U32_SIZE) },
                U32_SIZE as isize
            );
            assert_eq!(
                unsafe { libc::write(custom_callback_pipe[1], i_addr, U32_SIZE,) },
                U32_SIZE as isize
            );
        }
    });

    let mut exit = false;

    while !exit {
        // looper.poll_*_timeout(timeout) to not block indefinitely.
        // Pass a timeout of Duration::ZERO to never block.
        match looper.poll_all().unwrap() {
            Poll::Wake => { /* looper.as_foreign().wake() was called */ }
            Poll::Callback => {
                /* An event with a registered callback was received.
                 * Only received when polling for single events with poll_once_*
                 */
                unreachable!()
            }
            Poll::Timeout => {
                /* Timed out as per poll_*_timeout */
                unreachable!()
            }
            Poll::Event {
                ident,
                fd,
                events: _,
                data: _,
            } => {
                info!("File descriptor event on identifier {}", ident);
                match ident {
                    ndk_glue::NDK_GLUE_LOOPER_EVENT_PIPE_IDENT => {
                        // One of the callbacks in ANativeActivityCallbacks is called, and delivered
                        // to this application asynchronously by ndk_glue through a pipe.
                        // These consist mostly of important lifecycle and window events! Graphics
                        // applications will create and destroy their output surface/swapchain here.
                        info!(
                            "Event pipe yields: {:?}",
                            ndk_glue::poll_events()
                                .expect("Looper says event-pipe has data available!")
                        )
                    }
                    ndk_glue::NDK_GLUE_LOOPER_INPUT_QUEUE_IDENT => {
                        let input_queue = ndk_glue::input_queue();
                        let input_queue = input_queue.as_ref().expect("Input queue not attached");
                        assert!(input_queue.has_events().unwrap());
                        // Consume as many events as possible
                        while let Some(event) = input_queue.get_event().unwrap() {
                            // Pass the event by a possible IME (Input Method Editor, ie. an open keyboard) first
                            if let Some(event) = input_queue.pre_dispatch(event) {
                                info!("Input event {:?}", event);
                                let mut handled = false;
                                if let InputEvent::KeyEvent(key_event) = &event {
                                    if key_event.key_code() == Keycode::Back {
                                        // Gracefully stop the app when the user presses the back button
                                        exit = true;
                                        handled = true;
                                    }
                                }
                                // Let Android know that we did not consume the event
                                // (Pass true here if you did)
                                input_queue.finish_event(event, handled);
                            }
                        }
                    }
                    CUSTOM_EVENT_IDENT => {
                        // Expect to receive 32-bit numbers to describe events,
                        // as sent by the thread above
                        let mut recv = !0u32;
                        assert_eq!(
                            unsafe { libc::read(fd, &mut recv as *mut _ as *mut _, U32_SIZE) }
                                as usize,
                            U32_SIZE
                        );
                        info!("Read custom event from pipe: {}", recv);
                    }
                    i => panic!("Unexpected event identifier {}", i),
                }
            }
        }
    }

    // Stop the activity
    #[allow(deprecated)]
    ndk_glue::native_activity().finish()
}
