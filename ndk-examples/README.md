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
