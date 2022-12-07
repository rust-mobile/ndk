//! Demonstrates how to manage application lifetime using Android's `Looper`

use std::mem::MaybeUninit;
use std::os::unix::prelude::RawFd;
use std::time::Duration;

use log::info;
use ndk::looper::{FdEvent, ThreadLooper};

const U32_SIZE: usize = std::mem::size_of::<u32>();

use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Info));

    // Retrieve the Looper that android-activity created for us on the current thread.
    // android-activity uses this to block on events and poll file descriptors with a single mechanism.
    let looper =
        ThreadLooper::for_thread().expect("ndk-glue did not attach thread looper before main()!");

    fn create_pipe() -> [RawFd; 2] {
        let mut ends = MaybeUninit::<[RawFd; 2]>::uninit();
        assert_eq!(unsafe { libc::pipe(ends.as_mut_ptr().cast()) }, 0);
        unsafe { ends.assume_init() }
    }

    // Create a Unix pipe to send custom events to the Looper. ndk-glue uses a similar mechanism to deliver
    // ANativeActivityCallbacks asynchronously to the Looper through NDK_GLUE_LOOPER_EVENT_PIPE_IDENT.
    let custom_event_pipe = create_pipe();
    let custom_callback_pipe = create_pipe();

    // Attach the reading end of a pipe to a callback, too
    looper
        .as_foreign()
        .add_fd_with_callback(custom_callback_pipe[0], FdEvent::INPUT, |fd| {
            let mut recv = !0u32;
            assert_eq!(
                unsafe { libc::read(fd, &mut recv as *mut _ as *mut _, U32_SIZE) } as usize,
                U32_SIZE
            );
            println!("Read custom event from pipe, in callback: {}", recv);
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
    let mut redraw_pending = true;
    let mut render_state: Option<()> = Default::default();

    while !exit {
        app.poll_events(
            Some(std::time::Duration::from_secs(1)), /* timeout */
            |event| {
                match event {
                    PollEvent::Wake => {
                        info!("Early wake up");
                    }
                    PollEvent::Timeout => {
                        info!("Timed out");
                        // Real app would probably rely on vblank sync via graphics API...
                        redraw_pending = true;
                    }
                    PollEvent::Main(main_event) => {
                        info!("Main event: {:?}", main_event);
                        match main_event {
                            MainEvent::SaveState { saver, .. } => {
                                saver.store("foo://bar".as_bytes());
                            }
                            MainEvent::Pause => {}
                            MainEvent::Resume { loader, .. } => {
                                if let Some(state) = loader.load() {
                                    if let Ok(uri) = String::from_utf8(state) {
                                        info!("Resumed with saved state = {uri:#?}");
                                    }
                                }
                            }
                            MainEvent::InitWindow { .. } => {
                                render_state = Some(());
                                redraw_pending = true;
                            }
                            MainEvent::TerminateWindow { .. } => {
                                render_state = None;
                            }
                            MainEvent::WindowResized { .. } => {
                                redraw_pending = true;
                            }
                            MainEvent::RedrawNeeded { .. } => {
                                redraw_pending = true;
                            }
                            MainEvent::InputAvailable { .. } => {
                                redraw_pending = true;
                            }
                            MainEvent::ConfigChanged { .. } => {
                                info!("Config Changed: {:#?}", app.config());
                            }
                            MainEvent::LowMemory => {}

                            MainEvent::Destroy => exit = true,
                            _ => { /* ... */ }
                        }
                    }
                    _ => {}
                }

                if redraw_pending {
                    if let Some(_rs) = render_state {
                        redraw_pending = false;

                        // Handle input
                        app.input_events(|event| {
                            info!("Input Event: {event:?}");
                            InputStatus::Unhandled
                        });

                        info!("Render...");
                    }
                }
            },
        );
    }
}
