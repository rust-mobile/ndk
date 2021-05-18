# Rust on Android

[![Rust](https://github.com/rust-windowing/android-ndk-rs/workflows/Rust/badge.svg)](https://github.com/rust-windowing/android-ndk-rs/actions) ![MIT license](https://img.shields.io/badge/License-MIT-green.svg) ![APACHE2 license](https://img.shields.io/badge/License-APACHE2-green.svg)

Libraries and tools for Rust programming on Android targets:

Name | Description | Badges
--- | --- | ---
`ndk-sys` | Raw FFI bindings to the NDK | [![crates.io](https://img.shields.io/crates/v/ndk-sys.svg)](https://crates.io/crates/ndk-sys) [![crates.io](https://docs.rs/ndk-sys/badge.svg)](https://docs.rs/ndk-sys)
`ndk` | Safe abstraction of the bindings | [![crates.io](https://img.shields.io/crates/v/ndk.svg)](https://crates.io/crates/ndk) [![crates.io](https://docs.rs/ndk/badge.svg)](https://docs.rs/ndk)
`ndk-glue`| Startup code | [![crates.io](https://img.shields.io/crates/v/ndk-glue.svg)](https://crates.io/crates/ndk-glue) [![crates.io](https://docs.rs/ndk-glue/badge.svg)](https://docs.rs/ndk-glue)
`ndk-build` | Everything for building apk's | [![crates.io](https://img.shields.io/crates/v/ndk-build.svg)](https://crates.io/crates/ndk-build) [![crates.io](https://docs.rs/ndk-build/badge.svg)](https://docs.rs/ndk-build)
`cargo-apk` | Build tool | [![crates.io](https://img.shields.io/crates/v/cargo-apk.svg)](https://crates.io/crates/cargo-apk) [![crates.io](https://docs.rs/cargo-apk/badge.svg)](https://docs.rs/cargo-apk)

See [`ndk-examples`](./ndk-examples) for examples using the NDK and the README files of the crates for more details.

## Hello world

Quick start for setting up a new project with support for Android. For communication with the Android framework in our native Rust application we require a `NativeActivity`. `ndk-glue` will do the necessary initialization when calling `main` but requires a few adjustments:

`Cargo.toml`
```toml
[lib]
crate-type = ["lib", "cdylib"]
```

Wraps `main` function using attribute macro `ndk::glue::main`:

`src/lib.rs`
```rust
#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    println!("hello world");
}
```

`src/main.rs`
```rust
fn main() {
    $crate::main();
}
```

Install `cargo apk` for building, running and debugging your application:
```sh
cargo install cargo-apk
```

We can now directly execute our `Hello World` application on a real connected device or an emulator:
```sh
cargo apk run
```

## Logging and stdout
Stdout is redirected to the android log api when using `ndk-glue`. Any logger that logs to
stdout, like `println!`, should therefore work.

Use can filter the output in logcat
```
adb logcat RustStdoutStderr:D *:S
```

### Android logger
Android logger can be setup using feature "logger" and attribute macro like so:

`src/lib.rs`
```rust
#[cfg_attr(target_os = "android", ndk_glue::main(logger(level = "debug", tag = "my-tag")))]
pub fn main() {
    log!("hello world");
}
```

## Overriding crate paths
The macro `ndk_glue::main` tries to determine crate names from current _Cargo.toml_.
In cases when it is not possible the default crate names will be used.
You can override this names with specific paths like so:
```rust
#[ndk_glue::main(
  ndk_glue = "path::to::ndk_glue",
)]
fn main() {}
```

## JNI
Java Native Interface (JNI) allows executing Java code in a VM from native applications.
`ndk-examples` contains an `jni_audio` example which will print out all output audio devices in the log.

- [`jni`](https://crates.io/crates/jni), JNI bindings for Rust

## Winit and glutin
TODO shameless plug

## Flutter
TODO shameless plug
