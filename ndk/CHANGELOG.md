# Unreleased

# 0.4.0 (2021-08-02)

- **Breaking** Model looper file descriptor events integer as `bitflags`.

# 0.3.0 (2021-01-30)

- **Breaking** Looper `ident` not passed in `data` pointer anymore.
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
