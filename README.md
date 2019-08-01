# `android-ndk`: Rust bindings of the Android NDK

[![Build Status](https://travis-ci.org/mb64/android-ndk-rs.svg?branch=master)](https://travis-ci.org/mb64/android-ndk-rs)
[![Crates.io Status](https://meritbadge.herokuapp.com/android-ndk-sys)](https://crates.io/crates/android-ndk-sys)

This is a work in progress at the moment.

`android-ndk-sys` contains the raw FFI bindings, pre-generated from NDK r20, and `android-ndk`
provides a safe API over it.

Other helpful crates for Android:

 * [`jni`](https://crates.io/crates/jni), JNI bindings for Rust
 * [`ndk-logger`](https://crates.io/crates/ndk-logger), an Android backend for the `log` crate
