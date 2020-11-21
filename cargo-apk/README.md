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
# Name of your APK as shown in the app drawer and in the app switcher
apk_label = "APK Name"

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

# Set the instruction of how the activity should be launched
# See https://developer.android.com/guide/topics/manifest/activity-element#lmode
# Defaults to standard.
launch_mode = "standard"

# Set the minimum required OpenGL ES version.
# Defaults to [3, 1]
opengles_version = [3, 0]

# Sets the applications screenOrientation.
# See https://developer.android.com/guide/topics/manifest/activity-element
# and look for `android:screenOrientation` for possible values
# Defaults to "unspecified" which makes the system pick an orientation and
# doesn't give you help with rotation.
orientation = "sensorLandscape"

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

# Adds application metadata to the manifest
# Note that there can be several application_metadatas entries
# this will add: <meta-data android:name="com.samsung.android.vr.application.mode" android:value="vr_only"/>
[[package.metadata.android.application_metadatas]]
name = "com.samsung.android.vr.application.mode"
value = "vr_only"

# Adds activity metadata to the manifest
# Note that there can be several activity_metadatas entries
# this will add: <meta-data android:name="com.oculus.vr.focusaware" android:value="true"/>
[[package.metadata.android.activity_metadatas]]
name = "com.oculus.vr.focusaware"
value = "true"
```

TODO: intent filters
