# Rust on Android

 - Raw FFI bindings to the NDK ![ndk-sys-docs][ndk-sys-badge]
 - Safe abstraction of the bindings ![ndk-docs][ndk-badge]
 - Startup code ![ndk-glue-docs][ndk-glue-badge]
 - Everything for building apk's ![ndk-build-docs][ndk-build-badge]
 - Build tool ![cargo-apk-docs][cargo-apk-badge]

## Hello world

```toml
[lib]
crate-type = ["lib", "cdylib"]
```

```rust
#[cfg(target_os = "android")]
ndk_glue::ndk_glue!(main);

fn main() {
    println!("hello world");
}
```

```sh
cargo install cargo-apk
cargo apk run
```

## Logging and stdout
Stdout is redirected to the android log api when using `ndk-glue`. Any logger that logs to
stdout should therefore work.

## JNI
TODO: talk more about jni and add some examples

- [`jni`](https://crates.io/crates/jni), JNI bindings for Rust

## Winit and glutin
TODO shameless plug

## Flutter
TODO shameless plug

[ndk-sys-docs]: https://docs.rs/ndk-sys
[ndk-sys-badge]: https://docs.rs/ndk-sys/badge.svg
[ndk-docs]: https://docs.rs/ndk
[ndk-badge]: https://docs.rs/ndk/badge.svg
[ndk-glue-docs]: https://docs.rs/ndk-glue
[ndk-badge]: https://docs.rs/ndk-glue/badge.svg
[ndk-build-docs]: https://docs.rs/ndk-build
[ndk-build-badge]: https://docs.rs/ndk-build/badge.svg
[cargo-apk-docs]: https://docs.rs/cargo-apk
[cargo-apk-badge]: https://docs.rs/cargo-apk/badge.svg
