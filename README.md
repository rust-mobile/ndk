# Rust on Android

 - Raw FFI bindings to the NDK ![android-ndk-sys-docs][android-ndk-sys-badge]
 - Safe abstraction of the bindings ![android-ndk-docs][android-ndk-badge]
 - Startup code ![android-glue-docs][android-glue-badge]
 - Everything for building apk's ![android-build-tools-docs][android-build-tools-badge]
 - Build tool ![cargo-apk-docs][cargo-apk-badge]

## Hello world

```toml
[lib]
crate-type = ["cdylib"]
```

```rust
#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn ANativeActivity_onCreate(
    activity: *mut std::os::raw::c_void,
    saved_state: *mut std::os::raw::c_void,
    saved_state_size: usize,
) {
    android_glue::init(
        activity as _,
        saved_state as _,
        saved_state_size as _,
        app_main,
    );
}

pub fn app_main() {
    println!("hello world");
}
```

```sh
cargo install cargo-apk
cargo apk run
```

## Logging and stdout
Stdout is redirected to the android log api when using `android-glue`. Any logger that logs to
stdout should therefore work.

## JNI
TODO: talk more about jni and add some examples

- [`jni`](https://crates.io/crates/jni), JNI bindings for Rust

## Winit and glutin
TODO shameless plug

## Flutter
TODO shameless plug

[android-ndk-sys-docs]: https://docs.rs/android-ndk-sys
[android-ndk-sys-badge]: https://docs.rs/android-ndk-sys/badge.svg
[android-ndk-docs]: https://docs.rs/android-ndk
[android-ndk-badge]: https://docs.rs/android-ndk/badge.svg
[android-glue-docs]: https://docs.rs/android-glue
[android-glue-badge]: https://docs.rs/android-glue/badge.svg
[android-build-tools-docs]: https://docs.rs/android-build-tools
[android-build-tools-badge]: https://docs.rs/android-build-tools/badge.svg
[cargo-apk-docs]: https://docs.rs/cargo-apk
[cargo-apk-badge]: https://docs.rs/cargo-apk/badge.svg
