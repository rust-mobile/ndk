# Unreleased

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
