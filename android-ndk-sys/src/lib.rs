// Bindgen lints
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
// Test setup lints
#![cfg_attr(test, allow(dead_code))]

#[cfg(all(not(target_os = "android"), not(test), not(feature = "rustdoc")))]
compile_error!("android-ndk-sys only supports compiling for Android");

#[cfg(any(all(target_os = "android", target_arch = "arm"), feature = "rustdoc"))]
include!("ffi_arm.rs");

#[cfg(all(target_os = "android", target_arch = "armv7"))]
include!("ffi_armv7.rs");

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
include!("ffi_aarch64.rs");

#[cfg(all(target_os = "android", target_arch = "x86"))]
include!("ffi_i686.rs");

#[cfg(all(target_os = "android", target_arch = "x86_64"))]
include!("ffi_x86_64.rs");

#[cfg(all(test, target_arch = "aarch64"))]
mod ffi_aarch64;
#[cfg(all(test, target_arch = "arm"))]
mod ffi_arm;
#[cfg(all(test, target_arch = "armv7"))]
mod ffi_armv7;
#[cfg(all(test, target_arch = "x86"))]
mod ffi_i686;
#[cfg(all(test, target_arch = "x86_64"))]
mod ffi_x86_64;
