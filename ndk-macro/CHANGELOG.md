# Unreleased

# 0.3.0 (2021-11-16)

- **Breaking:** Removed `android_logger` and `log` crate path overrides from macro input attributes in favour of using the reexports from `ndk-glue`.
  Applications no longer have to provide these crates in scope of the `ndk_glue::main` macro when logging is enabled.

# 0.2.0 (2020-09-15)

- Added crate name override option
- **Breaking:** Changed macro attribute syntax

# 0.1.0 (2020-07-29)

- Initial release! 🎉
