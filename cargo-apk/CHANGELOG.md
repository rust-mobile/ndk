# Unreleased

- Reimplement NDK r23 `-lgcc` workaround using `RUSTFLAGS`, to apply to transitive `cdylib` compilations (#270)

# 0.9.0 (2022-05-07)

- **Breaking:** Use `min_sdk_version` to select compiler target instead of `target_sdk_version`. ([#197](https://github.com/rust-windowing/android-ndk-rs/pull/197))
  See https://developer.android.com/ndk/guides/sdk-versions#minsdkversion for more details.
- **Breaking:** Default `target_sdk_version` to `30` or lower (instead of the highest supported SDK version by the detected NDK toolchain)
  for more consistent interaction with Android backwards compatibility handling and its increasingly strict usage rules:
  https://developer.android.com/distribute/best-practices/develop/target-sdk
  ([#203](https://github.com/rust-windowing/android-ndk-rs/pull/203))
- Allow manifest `package` property to be provided in `Cargo.toml`. ([#236](https://github.com/rust-windowing/android-ndk-rs/pull/236))
- Add `MAIN` intent filter in `from_subcommand` instead of relying on a custom serialization function in `ndk-build`. ([#241](https://github.com/rust-windowing/android-ndk-rs/pull/241))
- Export the sole `NativeActivity` (through `android:exported="true"`) to allow it to be started by default if targeting Android S or higher. ([#242](https://github.com/rust-windowing/android-ndk-rs/pull/242))
- `cargo-apk` version can now be queried through `cargo apk version`. ([#218](https://github.com/rust-windowing/android-ndk-rs/pull/218))
- Environment variables from `.cargo/config.toml`'s `[env]` section are now propagated to the process environment. ([#249](https://github.com/rust-windowing/android-ndk-rs/pull/249))

# 0.8.2 (2021-11-22)

- Fixed the library name in case of multiple build artifacts in the Android manifest.
- Work around missing `libgcc` on NDK r23 beta 3 and above, by providing linker script that "redirects" to `libunwind`.
  See https://github.com/rust-windowing/android-ndk-rs/issues/149 and https://github.com/rust-lang/rust/pull/85806 for more details.

# 0.8.1 (2021-08-06)

- Updated to use [ndk-build 0.4.2](../ndk-build/CHANGELOG.md#042-2021-08-06)

# 0.8.0 (2021-07-06)

- Added `runtime_libs` path to android metadata for packaging extra dynamic libraries into the apk.

# 0.7.0 (2021-05-10)

- Added `cargo apk check`. Useful for compile-testing crates that contain C/C++ dependencies or
  target-specific conditional compilation, but do not provide a cdylib target.
- Added `apk_name` field to android metadata for APK file naming (defaults to Rust library name if unspecified).
  The application label is now no longer used for this purpose, and can contain a string resource ID from now on.

# 0.6.0 (2021-04-20)

- **Breaking:** uses `ndk-build`'s new (de)serialized `Manifest` struct to properly serialize a toml's `[package.metadata.android]` to an `AndroidManifest.xml`. The `[package.metadata.android]` now closely resembles the structure of [an android manifest file](https://developer.android.com/guide/topics/manifest/manifest-element). See [README](README.md) for an example of the new `[package.metadata.android]` structure and all manifest attributes that are currently supported.

# 0.5.6 (2020-11-25)

- Use `dunce::simplified` when extracting the manifest's assets and resource folder
- Updated to use [ndk-build 0.1.4](../ndk-build/CHANGELOG.md#014-2020-11-25)

# 0.5.5 (2020-11-21)

- Updated to use [ndk-build 0.1.3](../ndk-build/CHANGELOG.md#013-2020-11-21)

# 0.5.4 (2020-11-01)

- Added support for activity metadata entries.
- Fix glob member resolution in workspaces.

# 0.5.3 (2020-10-15)

- Fix `res` folder resolve.

# 0.5.2 (2020-09-15)

- Updated to use [ndk-build 0.1.2](../ndk-build/CHANGELOG.md#012-2020-09-15)

# 0.5.1 (2020-07-15)

- Updated to use [ndk-build 0.1.1](../ndk-build/CHANGELOG.md#011-2020-07-15)

# 0.5.0 (2020-04-22)

- Updated to use [ndk-build 0.1.0](../ndk-build/CHANGELOG.md#010-2020-04-22)
- First release in almost 3 years! 🎉
- **Breaking:** A ton of things changed!
