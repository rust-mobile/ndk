//! # Android NDK
//!
//! Bindings to the Android NDK.
//!
//! Currently has bindings:
//!  * `InputEvent`, `KeyEvent`, and `MotionEvent`, in the `event` module
//!  * `Looper`, in the `looper` module
//!  * `InputQueue`, in the `input_queue` module

pub mod event;
pub mod input_queue;
pub mod looper;
