//! # Android NDK
//!
//! Bindings to the Android NDK.
//!
//! Currently has bindings:
//!  * `InputEvent`, `KeyEvent`, and `MotionEvent`, in the `event` module
//!  * `Looper`, in the `looper` module
//!  * `InputQueue`, in the `input_queue` module
//!  * `AssetManager`, `AssetDir`, and `Asset`, in the `asset` module
//!  * `NativeActivity`, in the `native_activity` module
//!  * `Configuration`, in the `configuration` module
#![cfg_attr(
    feature = "native_app_glue",
    doc = "  * `native_app_glue`'s `AndroidApp`, in the `android_app` module"
)]

pub mod asset;
pub mod configuration;
pub mod event;
pub mod input_queue;
pub mod looper;
pub mod native_activity;

#[cfg(feature = "native_app_glue")]
pub mod android_app;
