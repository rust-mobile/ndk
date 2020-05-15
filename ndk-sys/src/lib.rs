//! Raw FFI bindings to the Android NDK.
//!
//! The bindings are pre-generated and the right one for the platform is selected at compile time.
//!
//! If you are including `android_native_app_glue.c`, the [`android_native_app_glue`
//! module](android_native_app_glue/index.html) contains the interface for that.

// Bindgen lints
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(clippy::all)]
// Test setup lints
#![cfg_attr(test, allow(dead_code))]

#[cfg(not(any(target_os = "android", feature = "test", feature = "rustdoc")))]
compile_error!("android-ndk-sys only supports compiling for Android");

#[cfg(any(
    all(
        any(target_os = "android", feature = "test"),
        any(target_arch = "arm", target_arch = "armv7")
    ),
    feature = "rustdoc"
))]
include!("ffi_arm.rs");

#[cfg(all(any(target_os = "android", feature = "test"), target_arch = "aarch64"))]
include!("ffi_aarch64.rs");

#[cfg(all(any(target_os = "android", feature = "test"), target_arch = "x86"))]
include!("ffi_i686.rs");

#[cfg(all(any(target_os = "android", feature = "test"), target_arch = "x86_64"))]
include!("ffi_x86_64.rs");

#[cfg(target_os = "android")]
#[link(name = "android")]
extern "C" {}

#[cfg(all(feature = "media", target_os = "android"))]
#[link(name = "mediandk")]
extern "C" {}
