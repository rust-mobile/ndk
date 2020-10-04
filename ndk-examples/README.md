# ndk-examples

Collection of examples showing different parts of the libraries.

## Examples

In order to see logs of the sample apps execute in a console:
```
adb logcat RustStdoutStderr:D '*:S'
```

### hello_world

Prints `hello world` in the console

```
cargo apk build --example hello_world
```

### jni_audio

Prints output audio devices in the console

```
cargo apk run --example jni_audio
```

### camera

Opens camera feed, and takes a JPEG capture

```
cargo apk run --example camera

adb shell pm grant rust.example.camera android.permission.WRITE_EXTERNAL_STORAGE
adb shell pm grant rust.example.camera android.permission.CAMERA

# run again, with the permissions enabled
cargo apk run --example camera

adb pull /sdcard/DCIM/ndk-rs-image.jpeg
```
