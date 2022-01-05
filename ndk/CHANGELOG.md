# Unreleased

# 0.6.0 (2022-01-05)

- **Breaking:** Upgrade to `ndk-sys 0.3.0` and migrate to `jni-sys` types that it now directly uses in its bindings.

# 0.5.0 (2021-11-22)

- **Breaking:** Replace `add_fd_with_callback` `ident` with constant value `ALOOPER_POLL_CALLBACK`,
  as per https://developer.android.com/ndk/reference/group/looper#alooper_addfd.
- **Breaking:** Accept unboxed closure in `add_fd_with_callback`.
- ndk/aaudio: Replace "Added in" comments with missing `#[cfg(feature)]`
- ndk/aaudio: Add missing `fn get_allowed_capture_policy()`
- ndk/configuration: Add missing `api-level-30` feature to `fn screen_round()`

# 0.4.0 (2021-08-02)

- **Breaking:** Model looper file descriptor events integer as `bitflags`.

# 0.3.0 (2021-01-30)

- **Breaking:** Looper `ident` not passed in `data` pointer anymore.
  `attach_looper` now only sets the `ident` field when attaching an
  `InputQueue` to a `ForeignLooper`.
  If you are relying on `Poll::Event::data` to tell event fd and
  input queue apart, please use `Poll::Event::ident` and the new
  constants introduced in `ndk-glue`!

# 0.2.1 (2020-10-15)

- Fix documentation build on docs.rs

# 0.2.0 (2020-09-15)

- **Breaking:** Updated to use [ndk-sys 0.2.0](../ndk-sys/CHANGELOG.md#020-2020-09-15)
- Added `media` bindings
- Added `bitmap` and `hardware_buffer` bindings
- Added `aaudio` bindings
- Fixed assets directory path to be relative to the manifest
- Added `trace` feature for native tracing

# 0.1.0 (2020-04-22)

- Initial release! 🎉
