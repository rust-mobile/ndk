# ndk-context

Provides a stable api to rust crates for interfacing with the android platform. It is
initialized by the runtime, usually [__ndk-glue__](https://crates.io/crates/ndk-glue),
but could also be initialized by java or kotlin code when embedding in an existing android
project.
