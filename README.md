# `android-ndk`: Rust bindings of the Android NDK

This is a work in progress at the moment.

To set up your environment to use `android-ndk`:
 * Download and unpack the latest [Android NDK](https://developer.android.com/ndk/downloads)
 * Set the environment variable `NDK_HOME` to the path of the extracted NDK:
   ```
   export NDK_HOME=/path/to/android-ndk-r20
   ```

Other helpful crates for Android:

 * [`jni`](https://crates.io/crates/jni), JNI bindings for Rust
 * [`ndk-logger`](https://crates.io/crates/ndk-logger), an Android backend for the `log` crate
