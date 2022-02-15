# Unreleased

# 0.4.1 (2022-02-15)

- Initialize `ndk-context` for cross-version access to the Java `VM` and Android `Context`.

# 0.4.0 (2021-08-02)

- Looper is now created before returning from `ANativeActivity_onCreate`, solving
  race conditions in `onInputQueueCreated`.
- Event pipe and looper are now notified of removal _before_ destroying `NativeWindow`
  and `InputQueue`. This allows applications to unlock their read-locks of these instances
  first (which they are supposed to hold on to during use) instead of deadlocking in
  Android callbacks.
- Reexport `android_logger` and `log` from the crate root for `ndk-macro` to use.
- Use new `FdEvents` `bitflags` for looper file descriptor events.
- Update to `ndk` 0.4.0.
  This minor dependency bump causes a minor bump for `ndk-glue` too.

# 0.3.0 (2021-01-30)

- **Breaking** Looper `ident` not passed in `data` pointer anymore.
  If you are relying on `Poll::Event::data` to tell event fd and
  input queue apart, please use `Poll::Event::ident` and the new
  constants introduced in `ndk-glue`!

# 0.2.1 (2020-10-15)

- Fix documentation build on docs.rs

# 0.2.0 (2020-09-15)

- **Breaking:** Removed `ndk_glue` macro in favor of new `main` attribute macro.

# 0.1.0 (2020-04-22)

- Initial release! 🎉
