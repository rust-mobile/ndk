# cargo apk

Tool for creating Android packages.

## Installation

From crates.io:
```
cargo install cargo-apk
```

From source:
```
cargo install --path .
```

## Commands

- `build`: Compiles the current package
- `run`: Run a binary or example of the local package
- `gdb`: Start a gdb session attached to an adb device with symbols loaded

## Manifest

`cargo` supports the `metadata` table for configurations for external tools like `cargo apk`.
Following configuration options are supported by `cargo apk` under `[package.metadata.android]`:

```toml
# The target Android API level.
target_sdk_version = 29
min_sdk_version = 26

# Virtual path your application's icon for any mipmap level.
# If not specified, an icon will not be included in the APK.
icon = "@mipmap/ic_launcher"

# If set to true, makes the app run in full-screen, by adding the following line
# as an XML attribute to the manifest's <application> tag :
#     android:theme="@android:style/Theme.DeviceDefault.NoActionBar.Fullscreen
# Defaults to false.
fullscreen = false

# Adds a uses-feature element to the manifest
# Supported keys: name, required
# See https://developer.android.com/guide/topics/manifest/uses-feature-element
[[package.metadata.android.feature]]
name = "android.hardware.camera"

[[package.metadata.android.feature]]
name = "android.hardware.vulkan.level"
version = "1"

# Adds a uses-permission element to the manifest.
# Note that android_version 23 and higher, Android requires the application to request permissions at runtime.
# There is currently no way to do this using a pure NDK based application.
# See https://developer.android.com/guide/topics/manifest/uses-permission-element
[[package.metadata.android.permission]]
name = "android.permission.WRITE_EXTERNAL_STORAGE"
max_sdk_version = 18

# Specifies the array of targets to build for.
build_targets = [ "armv7-linux-androideabi", "aarch64-linux-android", "i686-linux-android", "x86_64-linux-android" ]

# Path to your application's resources folder.
# If not specified, resources will not be included in the APK
res = "path/to/res_folder"

# Path to the folder containing your application's assets.
# If not specified, assets will not be included in the APK
assets = "path/to/assets_folder"
```

TODO: opengles version, intent filters and application metadatas