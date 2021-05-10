# Unreleased

- New `ApkConfig` field `apk_name` is now used for APK file naming, instead of the application label.
- Renamed `cargo_apk` utility to `cargo_ndk`.

# 0.2.0 (2021-04-20)

- **Breaking:** refactored `Manifest` into a proper (de)serialization struct. `Manifest` now closely matches [`an android manifest file`](https://developer.android.com/guide/topics/manifest/manifest-element).
- **Breaking:** removed `Config` in favor of using the new `Manifest` struct directly. Instead of using `Config::from_config` to create a `Manifest`, now you instantiate `Manifest` directly using, almost all, the same values.

# 0.1.4 (2020-11-25)

- On Windows, fixed UNC path handling for resource folder.

# 0.1.3 (2020-11-21)

- `android:launchMode` is configurable.

# 0.1.2 (2020-09-15)

- `android:label` is configurable.
- Library search paths are much more intelligent.
- `android:screenOrientation` is configurable.

# 0.1.1 (2020-07-15)

- Added support for custom intent filters.
- On Windows, fixed UNC path handling.
- Fixed toolchain path handling when the NDK installation has no host arch suffix on its prebuilt LLVM directories.

# 0.1.0 (2020-04-22)

- Initial release! 🎉
