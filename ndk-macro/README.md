# ndk-macro

Implementation of the attribute procedural macro `main` which applied directly to main function.

This macro is re-exported in `ndk-glue`. Typically, it's not needed to depend on this library directly!

## Usage
```Rust
#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    println!("hello world");
}
```

The attribute macro supports optional input attributes:

- `backtrace = "on|full"`: Enables backtraces by setting the `RUST_BACKTRACE` env var
- `ndk_glue = "path::to::ndk_glue"`: Overrides default path to __ndk_glue__ crate
- `logger(...props)`: Configures android logger with the passed configuration (requires the `logger` feature):
  - `level = "error|warn|info|debug|trace"`: Changes log level for logger
  - `tag = "my-tag"`: Assigns tag to logger
  - `filter = "filtering-rules"`: Changes default filtering rules
  - `android_logger = "path::to::android_logger"`: Overrides default path to __android_logger__ crate
  - `log = "path::to::log"`: Overrides default path to __log__ crate
