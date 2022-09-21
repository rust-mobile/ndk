# Unreleased

# 0.7.0 (2022-07-24)

- **Breaking:** Provide a `LockReadGuard` newtype around `NativeWindow`/`InputQueue` to hide the underlying lock implementation. (#288)
- **Breaking:** Transpose `LockReadGuard<Option<T>>` into `Option<LockReadGuard<T>>` to only necessitate an `Option` unpack/`unwrap()` once. (#282)

# 0.6.2 (2022-04-19)

- Call `ndk_context::release_android_context()` function to remove `AndroidContext` when activity is destroyed. (#263)

# 0.6.1 (2022-02-14)

- Initialize `ndk-context` for cross-version access to the Java `VM` and Android `Context`.

# 0.6.0 (2022-01-05)

- **Breaking:** Update to `ndk-sys 0.3.0` and `ndk 0.6.0`. (#214)

# 0.5.2 (2022-04-19)

- Call `ndk_context::release_android_context()` function to remove `AndroidContext` when activity is destroyed. (#263)

# 0.5.1 (2022-02-15)

- Initialize `ndk-context` for cross-version access to the Java `VM` and Android `Context`.

# 0.5.0 (2021-11-22)

- Document when to lock and unlock the window/input queue when certain events are received.
- **Breaking:** Update to `ndk 0.5.0` and `ndk-macros 0.3.0`.

# 0.4.2 (2022-04-19)

- Call `ndk_context::release_android_context()` function to remove `AndroidContext` when activity is destroyed. (#263)

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

- **Breaking:** Looper `ident` not passed in `data` pointer anymore.
  If you are relying on `Poll::Event::data` to tell event fd and
  input queue apart, please use `Poll::Event::ident` and the new
  constants introduced in `ndk-glue`!

# 0.2.1 (2020-10-15)

- Fix documentation build on docs.rs

# 0.2.0 (2020-09-15)

- **Breaking:** Removed `ndk_glue` macro in favor of new `main` attribute macro.

# 0.1.0 (2020-04-22)

- Initial release! 🎉
