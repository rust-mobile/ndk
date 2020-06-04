# Rust on Android

 - Raw FFI bindings to the NDK ![ndk-sys-docs][ndk-sys-badge]
 - Safe abstraction of the bindings ![ndk-docs][ndk-badge]
 - Startup code ![ndk-glue-docs][ndk-glue-badge]
 - Everything for building apk's ![ndk-build-docs][ndk-build-badge]
 - Build tool ![cargo-apk-docs][cargo-apk-badge]

## Hello world
`Cargo.toml`
```toml
[lib]
crate-type = ["lib", "cdylib"]
```

Wraps `main` function using attribute macro `ndk::glue::main`:

`src/lib.rs`
```rust
#[cfg_attr(target_os = "android", ndk_glue::main(backtrace))]
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

```sh
cargo install cargo-apk
cargo apk run
```

## Logging and stdout
Stdout is redirected to the android log api when using `ndk-glue`. Any logger that logs to
stdout should therefore work.

### Android logger
Android logger can be setup using feature "logger" and attribute macro like so:

`src/lib.rs`
```rust
#[cfg_attr(target_os = "android", ndk_glue::main(logger(debug, "my-tag")))]
pub fn main() {
    println!("hello world");
}
```

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
[ndk-glue-badge]: https://docs.rs/ndk-glue/badge.svg
[ndk-build-docs]: https://docs.rs/ndk-build
[ndk-build-badge]: https://docs.rs/ndk-build/badge.svg
[cargo-apk-docs]: https://docs.rs/cargo-apk
[cargo-apk-badge]: https://docs.rs/cargo-apk/badge.svg
