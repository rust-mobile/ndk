# Unreleased

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
