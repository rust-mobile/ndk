# cargo apk

Tool for creating Android packages.

## Installation

From crates.io:
```console
$ cargo install cargo-apk
```

From source:
```console
$ cargo install --path .
```

## Commands

- `build`: Compiles the current package
- `run`: Run a binary or example of the local package
- `gdb`: Start a gdb session attached to an adb device with symbols loaded

## Manifest

`cargo` supports the `metadata` table for configurations for external tools like `cargo apk`.
Following configuration options are supported by `cargo apk` under `[package.metadata.android]`:

```toml
[package.metadata.android]
# Specifies the array of targets to build for.
build_targets = [ "armv7-linux-androideabi", "aarch64-linux-android", "i686-linux-android", "x86_64-linux-android" ]

# Path to your application's resources folder.
# If not specified, resources will not be included in the APK.
resources = "path/to/resources_folder"

# Path to the folder containing your application's assets.
# If not specified, assets will not be included in the APK.
assets = "path/to/assets_folder"

# Name for final APK file.
# Defaults to package name.
apk_name = "myapp"

# Folder containing extra shared libraries intended to be dynamically loaded at runtime.
# Files matching `libs_folder/${android_abi}/*.so` are added to the apk
# according to the specified build_targets.
runtime_libs = "path/to/libs_folder"

# See https://developer.android.com/guide/topics/manifest/uses-sdk-element
#
# Defaults to a `min_sdk_version` of 23 and `target_sdk_version` of 30 (or lower if the detected NDK doesn't support this).
[package.metadata.android.sdk]
min_sdk_version = 23
target_sdk_version = 30
max_sdk_version = 29

# See https://developer.android.com/guide/topics/manifest/uses-feature-element
#
# Note: there can be multiple .uses_feature entries.
[[package.metadata.android.uses_feature]]
name = "android.hardware.vulkan.level"
required = true
version = 1

# See https://developer.android.com/guide/topics/manifest/uses-permission-element
#
# Note: there can be multiple .uses_permission entries.
[[package.metadata.android.uses_permission]]
name = "android.permission.WRITE_EXTERNAL_STORAGE"
max_sdk_version = 18

# See https://developer.android.com/guide/topics/manifest/application-element
[package.metadata.android.application]

# See https://developer.android.com/guide/topics/manifest/application-element#debug
#
# Defaults to false.
debuggable = false

# See https://developer.android.com/guide/topics/manifest/application-element#theme
#
# Example shows setting the theme of an application to fullscreen.
theme = "@android:style/Theme.DeviceDefault.NoActionBar.Fullscreen"

# Virtual path your application's icon for any mipmap level.
# If not specified, an icon will not be included in the APK.
icon = "@mipmap/ic_launcher"

# See https://developer.android.com/guide/topics/manifest/application-element#label
#
# Defaults to the compiled artifact's name.
label = "Application Name"

# See https://developer.android.com/guide/topics/manifest/meta-data-element
#
# Note: there can be several .meta_data entries.
# Note: the `resource` attribute is currently not supported.
[[package.metadata.android.application.meta_data]]
name = "com.samsung.android.vr.application.mode"
value = "vr_only"

# See https://developer.android.com/guide/topics/manifest/activity-element
[package.metadata.android.application.activity]

# See https://developer.android.com/guide/topics/manifest/activity-element#config
#
# Defaults to "orientation|keyboardHidden|screenSize".
config_changes = "orientation"

# See https://developer.android.com/guide/topics/manifest/activity-element#label
#
# Defaults to the application's label.
label = "Activity Name"

# See https://developer.android.com/guide/topics/manifest/activity-element#lmode
#
# Defaults to "standard".
launch_mode = "singleTop"

# See https://developer.android.com/guide/topics/manifest/activity-element#screen
#
# Defaults to "unspecified".
orientation = "landscape"

# See https://developer.android.com/guide/topics/manifest/meta-data-element
#
# Note: there can be several .meta_data entries.
# Note: the `resource` attribute is currently not supported.
[[package.metadata.android.application.activity.meta_data]]
name = "com.oculus.vr.focusaware"
value = "true"

# See https://developer.android.com/guide/topics/manifest/intent-filter-element
#
# Note: there can be several .intent_filter entries.
[[package.metadata.android.application.activity.intent_filter]]
# See https://developer.android.com/guide/topics/manifest/action-element
actions = ["android.intent.action.VIEW", "android.intent.action.WEB_SEARCH"]
# See https://developer.android.com/guide/topics/manifest/category-element
categories = ["android.intent.category.DEFAULT", "android.intent.category.BROWSABLE"]

# See https://developer.android.com/guide/topics/manifest/data-element
#
# Note: there can be several .data entries.
# Note: not specifying an attribute excludes it from the final data specification.
[[package.metadata.android.application.activity.intent_filter.data]]
scheme = "https"
host = "github.com"
port = "8080"
path = "/rust-windowing/android-ndk-rs/tree/master/cargo-apk"
path_prefix = "/rust-windowing/"
mime_type = "image/jpeg"
```

If a manifest attribute is not supported by `cargo apk` feel free to create a PR that adds the missing attribute.
