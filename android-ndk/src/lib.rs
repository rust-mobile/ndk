//! # Android NDK
//!
//! Bindings to the Android NDK.
//!
//! Currently has bindings:
//!  * `AInputEvent`, `AKeyEvent`, and `AMotionEvent`, in the `event` module
//!  * `ALooper`, in the `looper` module
//!  * `AInputQueue`, in the `queue` module

pub mod event;
pub mod looper;
pub mod queue;
