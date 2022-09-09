# Unreleased

- **Breaking:** Postpone APK library packaging until before zip alignment, to deduplicate possibly overlapping entries. ([#333](https://github.com/rust-windowing/android-ndk-rs/pull/333))
- Add `adb` device serial parameter to `detect_abi()` and `Apk::{install,start}()`. ([#329](https://github.com/rust-windowing/android-ndk-rs/pull/329))
- Fix missing `.exe` extension for `adb` on Windows inside `detect_abi()`. ([#339](https://github.com/rust-windowing/android-ndk-rs/pull/339))
- `start()` now returns the PID of the started app process (useful for passing to `adb logcat --pid`). ([#331](https://github.com/rust-windowing/android-ndk-rs/pull/331))
- Inherit `ndk_gdb()` function from `cargo-apk` with the appropriate script extension across platforms. ([#330](https://github.com/rust-windowing/android-ndk-rs/pull/330), [#258](https://github.com/rust-windowing/android-ndk-rs/pull/258))
- Provide `adb` path to `ndk-gdb`, allowing it to run without `adb` in `PATH`. ([#343](https://github.com/rust-windowing/android-ndk-rs/pull/343))

# 0.7.0 (2022-07-05)

- Fix NDK r23 `-lgcc` workaround for target directories containing spaces. ([#298](https://github.com/rust-windowing/android-ndk-rs/pull/298))
- Invoke `clang` directly instead of through the NDK's wrapper scripts. ([#306](https://github.com/rust-windowing/android-ndk-rs/pull/306))
- **Breaking:** Rename `Activity::intent_filters` back to `Activity::intent_filter`. ([#305](https://github.com/rust-windowing/android-ndk-rs/pull/305))

# 0.6.0 (2022-06-11)

- **Breaking:** Provide NDK r23 `-lgcc` workaround in `cargo_ndk()` function, now requiring `target_dir` as argument. ([#286](https://github.com/rust-windowing/android-ndk-rs/pull/286))
- **Breaking:** Add `disable_aapt_compression` field to `ApkConfig` to disable `aapt` compression. ([#283](https://github.com/rust-windowing/android-ndk-rs/pull/283))

# 0.5.0 (2022-05-07)

- **Breaking:** Default `target_sdk_version` to `30` or lower (instead of the highest supported SDK version by the detected NDK toolchain)
  for more consistent interaction with Android backwards compatibility handling and its increasingly strict usage rules:
  https://developer.android.com/distribute/best-practices/develop/target-sdk
- **Breaking:** Remove default insertion of `MAIN` intent filter through a custom serialization function, this is better filled in by
  the default setup in `cargo-apk`. ([#241](https://github.com/rust-windowing/android-ndk-rs/pull/241))
- Add `android:exported` attribute to the manifest's `Activity` element. ([#242](https://github.com/rust-windowing/android-ndk-rs/pull/242))
- Add `android:sharedUserId` attribute to the manifest's top-level `manifest` element. ([#252](https://github.com/rust-windowing/android-ndk-rs/pull/252))
- Add `queries` element to the manifest's top-level `manifest` element. ([#259](https://github.com/rust-windowing/android-ndk-rs/pull/259))

# 0.4.3 (2021-11-22)

- Provide NDK `build_tag` version from `source.properties` in the NDK root.

# 0.4.2 (2021-08-06)

- Pass UNIX path separators to `aapt` on non-UNIX systems, ensuring the resulting separator is compatible with the target device instead of the host platform.

# 0.4.1 (2021-08-02)

- Only the highest platform supported by the NDK is now selected as default platform.

# 0.4.0 (2021-07-06)

- Added `add_runtime_libs` function for including extra dynamic libraries in the APK.

# 0.3.0 (2021-05-10)

- New `ApkConfig` field `apk_name` is now used for APK file naming, instead of the application label.
- Renamed `cargo_apk` utility to `cargo_ndk`.

# 0.2.0 (2021-04-20)

- **Breaking:** refactored `Manifest` into a proper (de)serialization struct. `Manifest` now closely matches [an android manifest file](https://developer.android.com/guide/topics/manifest/manifest-element).
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
