# ndk-macro

Implementation of the attribute procedural macro `main` which applied directly to main function.

This macro is re-exported in `ndk-glue`. Typically, it's not needed to depend on this library directly!

## Usage
```Rust
#[cfg_attr(target_os = "android", ndk_glue::main(backtrace))]
pub fn main() {
    println!("hello world");
}
```

The attribute macro supports optional input attributes:

- `backtrace`: Enables backtraces by setting the `RUST_BACKTRACE` env var
- `logger(debug, "my-tag")`: Configures android logger with the passed configuration, requires the `logger` feature.
