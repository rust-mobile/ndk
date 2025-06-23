//! Bindings for [`AInputEvent`, `AKeyEvent` and `AMotionEvent`]
//!
//! Most of these operations directly wrap functions in the NDK.
//!
//! See also the Java docs for [`android.view.InputEvent`], [`android.view.MotionEvent`], and
//! [`android.view.KeyEvent`].
//!
//! [`AInputEvent`, `AKeyEvent` and `AMotionEvent`]: https://developer.android.com/ndk/reference/group/input
//! [`android.view.InputEvent`]: https://developer.android.com/reference/android/view/InputEvent
//! [`android.view.MotionEvent`]: https://developer.android.com/reference/android/view/MotionEvent
//! [`android.view.KeyEvent`]: https://developer.android.com/reference/android/view/KeyEvent

use std::ptr::NonNull;

#[cfg(feature = "api-level-31")]
use jni_sys::{jobject, JNIEnv};
use num_enum::{FromPrimitive, IntoPrimitive};

/// A native [`AInputEvent *`]
///
/// [`AInputEvent *`]: https://developer.android.com/ndk/reference/group/input#ainputevent
#[derive(Debug)]
#[non_exhaustive]
#[doc(alias = "AInputEvent")]
pub enum InputEvent {
    MotionEvent(MotionEvent),
    KeyEvent(KeyEvent),
}

/// Wraps a Java [`InputEvent`] acquired from [`KeyEvent::from_java()`] or
/// [`MotionEvent::from_java()`] with respective [`Drop`] semantics.
#[cfg(feature = "api-level-31")]
#[derive(Debug)]
#[doc(alias = "AInputEvent")]
pub struct InputEventJava(InputEvent);

#[cfg(feature = "api-level-31")]
impl Drop for InputEventJava {
    /// Releases interface objects created by [`KeyEvent::from_java()`] or
    /// [`MotionEvent::from_java()`].
    ///
    /// The underlying Java object remains valid and does not change its state.
    #[doc(alias = "AInputEvent_release")]
    fn drop(&mut self) {
        let ptr = match self.0 {
            InputEvent::MotionEvent(MotionEvent { ptr })
            | InputEvent::KeyEvent(KeyEvent { ptr }) => ptr.as_ptr().cast(),
        };
        unsafe { ffi::AInputEvent_release(ptr) }
    }
}

/// An enum representing the source of an [`InputEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(i32)]
#[non_exhaustive]
pub enum Source {
    #[doc(alias = "AINPUT_SOURCE_UNKNOWN")]
    Unknown = ffi::AINPUT_SOURCE_UNKNOWN as i32,
    #[doc(alias = "AINPUT_SOURCE_KEYBOARD")]
    Keyboard = ffi::AINPUT_SOURCE_KEYBOARD as i32,
    #[doc(alias = "AINPUT_SOURCE_DPAD")]
    Dpad = ffi::AINPUT_SOURCE_DPAD as i32,
    #[doc(alias = "AINPUT_SOURCE_GAMEPAD")]
    Gamepad = ffi::AINPUT_SOURCE_GAMEPAD as i32,
    #[doc(alias = "AINPUT_SOURCE_TOUCHSCREEN")]
    Touchscreen = ffi::AINPUT_SOURCE_TOUCHSCREEN as i32,
    #[doc(alias = "AINPUT_SOURCE_MOUSE")]
    Mouse = ffi::AINPUT_SOURCE_MOUSE as i32,
    #[doc(alias = "AINPUT_SOURCE_STYLUS")]
    Stylus = ffi::AINPUT_SOURCE_STYLUS as i32,
    #[doc(alias = "AINPUT_SOURCE_BLUETOOTH_STYLUS")]
    BluetoothStylus = ffi::AINPUT_SOURCE_BLUETOOTH_STYLUS as i32,
    #[doc(alias = "AINPUT_SOURCE_TRACKBALL")]
    Trackball = ffi::AINPUT_SOURCE_TRACKBALL as i32,
    #[doc(alias = "AINPUT_SOURCE_MOUSE_RELATIVE")]
    MouseRelative = ffi::AINPUT_SOURCE_MOUSE_RELATIVE as i32,
    #[doc(alias = "AINPUT_SOURCE_TOUCHPAD")]
    Touchpad = ffi::AINPUT_SOURCE_TOUCHPAD as i32,
    #[doc(alias = "AINPUT_SOURCE_TOUCH_NAVIGATION")]
    TouchNavigation = ffi::AINPUT_SOURCE_TOUCH_NAVIGATION as i32,
    #[doc(alias = "AINPUT_SOURCE_JOYSTICK")]
    Joystick = ffi::AINPUT_SOURCE_JOYSTICK as i32,
    #[doc(alias = "AINPUT_SOURCE_HDMI")]
    Hdmi = ffi::AINPUT_SOURCE_HDMI as i32,
    #[doc(alias = "AINPUT_SOURCE_SENSOR")]
    Sensor = ffi::AINPUT_SOURCE_SENSOR as i32,
    #[doc(alias = "AINPUT_SOURCE_ROTARY_ENCODER")]
    RotaryEncoder = ffi::AINPUT_SOURCE_ROTARY_ENCODER as i32,
    #[doc(alias = "AINPUT_SOURCE_ANY")]
    Any = ffi::AINPUT_SOURCE_ANY as i32,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(i32),
}

impl Source {
    #[doc(alias = "AINPUT_SOURCE_CLASS_MASK")]
    pub fn class(self) -> SourceClass {
        let class = i32::from(self) & ffi::AINPUT_SOURCE_CLASS_MASK as i32;
        // The mask fits in a u8.
        SourceClass::from_bits_retain(class as u8)
    }
}

bitflags::bitflags! {
    /// Flags representing the class of an [`InputEvent`] [`Source`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SourceClass : u8 {
        #[doc(alias = "AINPUT_SOURCE_CLASS_BUTTON")]
        const BUTTON = ffi::AINPUT_SOURCE_CLASS_BUTTON as u8;
        #[doc(alias = "AINPUT_SOURCE_CLASS_POINTER")]
        const POINTER = ffi::AINPUT_SOURCE_CLASS_POINTER as u8;
        #[doc(alias = "AINPUT_SOURCE_CLASS_NAVIGATION")]
        const NAVIGATION = ffi::AINPUT_SOURCE_CLASS_NAVIGATION as u8;
        #[doc(alias = "AINPUT_SOURCE_CLASS_POSITION")]
        const POSITION = ffi::AINPUT_SOURCE_CLASS_POSITION as u8;
        #[doc(alias = "AINPUT_SOURCE_CLASS_JOYSTICK")]
        const JOYSTICK = ffi::AINPUT_SOURCE_CLASS_JOYSTICK as u8;

        // https://docs.rs/bitflags/latest/bitflags/#externally-defined-flags
        const _ = ffi::AINPUT_SOURCE_CLASS_MASK as u8;
    }
}

impl InputEvent {
    /// Initialize an [`InputEvent`] from a pointer
    ///
    /// # Safety
    /// By calling this function, you assert that the pointer is a valid pointer to a
    /// native [`ffi::AInputEvent`].
    #[inline]
    #[doc(alias = "AInputEvent_getType")]
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AInputEvent>) -> Self {
        match ffi::AInputEvent_getType(ptr.as_ptr()) as u32 {
            ffi::AINPUT_EVENT_TYPE_KEY => InputEvent::KeyEvent(KeyEvent::from_ptr(ptr)),
            ffi::AINPUT_EVENT_TYPE_MOTION => InputEvent::MotionEvent(MotionEvent::from_ptr(ptr)),
            x => panic!("Bad event type received: {}", x),
        }
    }

    /// Returns a pointer to the native [`ffi::AInputEvent`].
    #[inline]
    pub fn ptr(&self) -> NonNull<ffi::AInputEvent> {
        match self {
            InputEvent::MotionEvent(MotionEvent { ptr }) => *ptr,
            InputEvent::KeyEvent(KeyEvent { ptr }) => *ptr,
        }
    }

    /// Creates a Java [`android.view.InputEvent`] object that is a copy of this native
    /// [`InputEvent`].
    ///
    /// [`android.view.KeyEvent`]: https://developer.android.com/reference/android/view/KeyEvent
    #[cfg(feature = "api-level-35")]
    #[doc(alias = "AInputEvent_toJava")]
    pub unsafe fn to_java(&self, env: *mut JNIEnv) -> jobject {
        ffi::AInputEvent_toJava(env, self.ptr().as_ptr())
    }

    /// Get the source of the event.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#ainputevent_getsource)
    #[inline]
    #[doc(alias = "AInputEvent_getSource")]
    pub fn source(&self) -> Source {
        let source = unsafe { ffi::AInputEvent_getSource(self.ptr().as_ptr()) };
        source.into()
    }

    /// Get the device id associated with the event.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#ainputevent_getdeviceid)
    #[inline]
    #[doc(alias = "AInputEvent_getDeviceId")]
    pub fn device_id(&self) -> i32 {
        unsafe { ffi::AInputEvent_getDeviceId(self.ptr().as_ptr()) }
    }
}

/// A bitfield representing the state of modifier keys during an event.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MetaState(pub u32);

impl MetaState {
    #[inline]
    #[doc(alias = "AMETA_ALT_ON")]
    pub fn alt_on(self) -> bool {
        self.0 & ffi::AMETA_ALT_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_ALT_LEFT_ON")]
    pub fn alt_left_on(self) -> bool {
        self.0 & ffi::AMETA_ALT_LEFT_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_ALT_RIGHT_ON")]
    pub fn alt_right_on(self) -> bool {
        self.0 & ffi::AMETA_ALT_RIGHT_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_SHIFT_ON")]
    pub fn shift_on(self) -> bool {
        self.0 & ffi::AMETA_SHIFT_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_SHIFT_LEFT_ON")]
    pub fn shift_left_on(self) -> bool {
        self.0 & ffi::AMETA_SHIFT_LEFT_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_SHIFT_RIGHT_ON")]
    pub fn shift_right_on(self) -> bool {
        self.0 & ffi::AMETA_SHIFT_RIGHT_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_SYM_ON")]
    pub fn sym_on(self) -> bool {
        self.0 & ffi::AMETA_SYM_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_FUNCTION_ON")]
    pub fn function_on(self) -> bool {
        self.0 & ffi::AMETA_FUNCTION_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_CTRL_ON")]
    pub fn ctrl_on(self) -> bool {
        self.0 & ffi::AMETA_CTRL_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_CTRL_LEFT_ON")]
    pub fn ctrl_left_on(self) -> bool {
        self.0 & ffi::AMETA_CTRL_LEFT_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_CTRL_RIGHT_ON")]
    pub fn ctrl_right_on(self) -> bool {
        self.0 & ffi::AMETA_CTRL_RIGHT_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_META_ON")]
    pub fn meta_on(self) -> bool {
        self.0 & ffi::AMETA_META_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_META_LEFT_ON")]
    pub fn meta_left_on(self) -> bool {
        self.0 & ffi::AMETA_META_LEFT_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_META_RIGHT_ON")]
    pub fn meta_right_on(self) -> bool {
        self.0 & ffi::AMETA_META_RIGHT_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_CAPS_LOCK_ON")]
    pub fn caps_lock_on(self) -> bool {
        self.0 & ffi::AMETA_CAPS_LOCK_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_NUM_LOCK_ON")]
    pub fn num_lock_on(self) -> bool {
        self.0 & ffi::AMETA_NUM_LOCK_ON != 0
    }
    #[inline]
    #[doc(alias = "AMETA_SCROLL_LOCK_ON")]
    pub fn scroll_lock_on(self) -> bool {
        self.0 & ffi::AMETA_SCROLL_LOCK_ON != 0
    }
}

/// A motion event
///
/// Wraps an [`AInputEvent *`] of the [`ffi::AINPUT_EVENT_TYPE_MOTION`] type.
///
/// For general discussion of motion events in Android, see [the relevant
/// javadoc](https://developer.android.com/reference/android/view/MotionEvent).
///
/// [`AInputEvent *`]: https://developer.android.com/ndk/reference/group/input#ainputevent
#[derive(Clone, Debug)]
#[doc(alias = "AINPUT_EVENT_TYPE_MOTION")]
pub struct MotionEvent {
    ptr: NonNull<ffi::AInputEvent>,
}

// TODO: thread safety?

/// A motion action.
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(i32)]
#[non_exhaustive]
pub enum MotionAction {
    #[doc(alias = "AMOTION_EVENT_ACTION_DOWN")]
    Down = ffi::AMOTION_EVENT_ACTION_DOWN as i32,
    #[doc(alias = "AMOTION_EVENT_ACTION_UP")]
    Up = ffi::AMOTION_EVENT_ACTION_UP as i32,
    #[doc(alias = "AMOTION_EVENT_ACTION_MOVE")]
    Move = ffi::AMOTION_EVENT_ACTION_MOVE as i32,
    #[doc(alias = "AMOTION_EVENT_ACTION_CANCEL")]
    Cancel = ffi::AMOTION_EVENT_ACTION_CANCEL as i32,
    #[doc(alias = "AMOTION_EVENT_ACTION_OUTSIDE")]
    Outside = ffi::AMOTION_EVENT_ACTION_OUTSIDE as i32,
    #[doc(alias = "AMOTION_EVENT_ACTION_POINTER_DOWN")]
    PointerDown = ffi::AMOTION_EVENT_ACTION_POINTER_DOWN as i32,
    #[doc(alias = "AMOTION_EVENT_ACTION_POINTER_UP")]
    PointerUp = ffi::AMOTION_EVENT_ACTION_POINTER_UP as i32,
    #[doc(alias = "AMOTION_EVENT_ACTION_HOVER_MOVE")]
    HoverMove = ffi::AMOTION_EVENT_ACTION_HOVER_MOVE as i32,
    #[doc(alias = "AMOTION_EVENT_ACTION_SCROLL")]
    Scroll = ffi::AMOTION_EVENT_ACTION_SCROLL as i32,
    #[doc(alias = "AMOTION_EVENT_ACTION_HOVER_ENTER")]
    HoverEnter = ffi::AMOTION_EVENT_ACTION_HOVER_ENTER as i32,
    #[doc(alias = "AMOTION_EVENT_ACTION_HOVER_EXIT")]
    HoverExit = ffi::AMOTION_EVENT_ACTION_HOVER_EXIT as i32,
    #[doc(alias = "AMOTION_EVENT_ACTION_BUTTON_PRESS")]
    ButtonPress = ffi::AMOTION_EVENT_ACTION_BUTTON_PRESS as i32,
    #[doc(alias = "AMOTION_EVENT_ACTION_BUTTON_RELEASE")]
    ButtonRelease = ffi::AMOTION_EVENT_ACTION_BUTTON_RELEASE as i32,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(i32),
}

/// An axis of a motion event.
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(i32)]
#[non_exhaustive]
#[doc(alias = "AMOTION_EVENT_AXIS")]
pub enum Axis {
    #[doc(alias = "AMOTION_EVENT_AXIS_X")]
    X = ffi::AMOTION_EVENT_AXIS_X as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_Y")]
    Y = ffi::AMOTION_EVENT_AXIS_Y as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_PRESSURE")]
    Pressure = ffi::AMOTION_EVENT_AXIS_PRESSURE as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_SIZE")]
    Size = ffi::AMOTION_EVENT_AXIS_SIZE as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_TOUCH_MAJOR")]
    TouchMajor = ffi::AMOTION_EVENT_AXIS_TOUCH_MAJOR as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_TOUCH_MINOR")]
    TouchMinor = ffi::AMOTION_EVENT_AXIS_TOUCH_MINOR as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_TOOL_MAJOR")]
    ToolMajor = ffi::AMOTION_EVENT_AXIS_TOOL_MAJOR as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_TOOL_MINOR")]
    ToolMinor = ffi::AMOTION_EVENT_AXIS_TOOL_MINOR as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_ORIENTATION")]
    Orientation = ffi::AMOTION_EVENT_AXIS_ORIENTATION as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_VSCROLL")]
    Vscroll = ffi::AMOTION_EVENT_AXIS_VSCROLL as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_HSCROLL")]
    Hscroll = ffi::AMOTION_EVENT_AXIS_HSCROLL as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_Z")]
    Z = ffi::AMOTION_EVENT_AXIS_Z as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_RX")]
    Rx = ffi::AMOTION_EVENT_AXIS_RX as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_RY")]
    Ry = ffi::AMOTION_EVENT_AXIS_RY as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_RZ")]
    Rz = ffi::AMOTION_EVENT_AXIS_RZ as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_HAT_X")]
    HatX = ffi::AMOTION_EVENT_AXIS_HAT_X as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_HAT_Y")]
    HatY = ffi::AMOTION_EVENT_AXIS_HAT_Y as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_LTRIGGER")]
    Ltrigger = ffi::AMOTION_EVENT_AXIS_LTRIGGER as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_RTRIGGER")]
    Rtrigger = ffi::AMOTION_EVENT_AXIS_RTRIGGER as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_THROTTLE")]
    Throttle = ffi::AMOTION_EVENT_AXIS_THROTTLE as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_RUDDER")]
    Rudder = ffi::AMOTION_EVENT_AXIS_RUDDER as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_WHEEL")]
    Wheel = ffi::AMOTION_EVENT_AXIS_WHEEL as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GAS")]
    Gas = ffi::AMOTION_EVENT_AXIS_GAS as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_BRAKE")]
    Brake = ffi::AMOTION_EVENT_AXIS_BRAKE as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_DISTANCE")]
    Distance = ffi::AMOTION_EVENT_AXIS_DISTANCE as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_TILT")]
    Tilt = ffi::AMOTION_EVENT_AXIS_TILT as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_SCROLL")]
    Scroll = ffi::AMOTION_EVENT_AXIS_SCROLL as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_RELATIVE_X")]
    RelativeX = ffi::AMOTION_EVENT_AXIS_RELATIVE_X as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_RELATIVE_Y")]
    RelativeY = ffi::AMOTION_EVENT_AXIS_RELATIVE_Y as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_1")]
    Generic1 = ffi::AMOTION_EVENT_AXIS_GENERIC_1 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_2")]
    Generic2 = ffi::AMOTION_EVENT_AXIS_GENERIC_2 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_3")]
    Generic3 = ffi::AMOTION_EVENT_AXIS_GENERIC_3 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_4")]
    Generic4 = ffi::AMOTION_EVENT_AXIS_GENERIC_4 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_5")]
    Generic5 = ffi::AMOTION_EVENT_AXIS_GENERIC_5 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_6")]
    Generic6 = ffi::AMOTION_EVENT_AXIS_GENERIC_6 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_7")]
    Generic7 = ffi::AMOTION_EVENT_AXIS_GENERIC_7 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_8")]
    Generic8 = ffi::AMOTION_EVENT_AXIS_GENERIC_8 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_9")]
    Generic9 = ffi::AMOTION_EVENT_AXIS_GENERIC_9 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_10")]
    Generic10 = ffi::AMOTION_EVENT_AXIS_GENERIC_10 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_11")]
    Generic11 = ffi::AMOTION_EVENT_AXIS_GENERIC_11 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_12")]
    Generic12 = ffi::AMOTION_EVENT_AXIS_GENERIC_12 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_13")]
    Generic13 = ffi::AMOTION_EVENT_AXIS_GENERIC_13 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_14")]
    Generic14 = ffi::AMOTION_EVENT_AXIS_GENERIC_14 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_15")]
    Generic15 = ffi::AMOTION_EVENT_AXIS_GENERIC_15 as i32,
    #[doc(alias = "AMOTION_EVENT_AXIS_GENERIC_16")]
    Generic16 = ffi::AMOTION_EVENT_AXIS_GENERIC_16 as i32,

    /// Axis constant: X gesture offset axis of a motion event.
    ///
    /// - For a touch pad, reports the distance that a swipe gesture has moved in the X axis, as a
    ///   proportion of the touch pad's size. For example, if a touch pad is `1000` units wide, and
    ///   a swipe gesture starts at `X = 500` then moves to `X = 400`, this axis would have a value
    ///   of `-0.1`.
    ///
    /// These values are relative to the state from the last event, not accumulated, so developers
    /// should make sure to process this axis value for all batched historical events.
    ///
    /// This axis is only set on the first pointer in a motion event.
    #[doc(alias = "AMOTION_EVENT_AXIS_GESTURE_X_OFFSET")]
    GestureXOffset = ffi::AMOTION_EVENT_AXIS_GESTURE_X_OFFSET as i32,
    /// Axis constant: Y gesture offset axis of a motion event.
    ///
    /// The same as [`Axis::GestureXOffset`], but for the Y axis.
    #[doc(alias = "AMOTION_EVENT_AXIS_GESTURE_Y_OFFSET")]
    GestureYOffset = ffi::AMOTION_EVENT_AXIS_GESTURE_Y_OFFSET as i32,
    /// Axis constant: X scroll distance axis of a motion event.
    ///
    /// - For a touch pad, reports the distance that should be scrolled in the X axis as a result of
    ///   the user's two-finger scroll gesture, in display pixels.
    ///
    /// These values are relative to the state from the last event, not accumulated, so developers
    /// should make sure to process this axis value for all batched historical events.
    ///
    /// This axis is only set on the first pointer in a motion event.
    #[doc(alias = "AMOTION_EVENT_AXIS_GESTURE_SCROLL_X_DISTANCE")]
    GestureScrollXDistance = ffi::AMOTION_EVENT_AXIS_GESTURE_SCROLL_X_DISTANCE as i32,
    /// Axis constant: Y scroll distance axis of a motion event.
    ///
    /// The same as [`Axis::GestureScrollXDistance`], but for the Y axis.
    #[doc(alias = "AMOTION_EVENT_AXIS_GESTURE_SCROLL_Y_DISTANCE")]
    GestureScrollYDistance = ffi::AMOTION_EVENT_AXIS_GESTURE_SCROLL_Y_DISTANCE as i32,
    /// Axis constant: pinch scale factor of a motion event.
    ///
    /// - For a touch pad, reports the change in distance between the fingers when the user is
    ///   making a pinch gesture, as a proportion of that distance when the gesture was last
    ///   reported. For example, if the fingers were `50` units apart and are now `52` units apart,
    ///   the scale factor would be `1.04`.
    ///
    /// These values are relative to the state from the last event, not accumulated, so developers
    /// should make sure to process this axis value for all batched historical events.
    ///
    /// This axis is only set on the first pointer in a motion event.
    #[doc(alias = "AMOTION_EVENT_AXIS_GESTURE_PINCH_SCALE_FACTOR")]
    GesturePinchScaleFactor = ffi::AMOTION_EVENT_AXIS_GESTURE_PINCH_SCALE_FACTOR as i32,
    /// Axis constant: the number of fingers being used in a multi-finger swipe gesture.
    ///
    /// - For a touch pad, reports the number of fingers being used in a multi-finger swipe gesture
    ///   (with [`MotionClassification::MultiFingerSwipe`]).
    ///
    /// Since [`MotionClassification::MultiFingerSwipe`] is a hidden API, so is this axis. It is
    /// only set on the first pointer in a motion event.
    #[doc(alias = "AMOTION_EVENT_AXIS_GESTURE_SWIPE_FINGER_COUNT")]
    GestureSwipeFingerCount = ffi::AMOTION_EVENT_AXIS_GESTURE_SWIPE_FINGER_COUNT as i32,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(i32),
}

/// The tool type of a pointer.
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(i32)]
#[non_exhaustive]
pub enum ToolType {
    #[doc(alias = "AMOTION_EVENT_TOOL_TYPE_UNKNOWN")]
    Unknown = ffi::AMOTION_EVENT_TOOL_TYPE_UNKNOWN as i32,
    #[doc(alias = "AMOTION_EVENT_TOOL_TYPE_FINGER")]
    Finger = ffi::AMOTION_EVENT_TOOL_TYPE_FINGER as i32,
    #[doc(alias = "AMOTION_EVENT_TOOL_TYPE_STYLUS")]
    Stylus = ffi::AMOTION_EVENT_TOOL_TYPE_STYLUS as i32,
    #[doc(alias = "AMOTION_EVENT_TOOL_TYPE_MOUSE")]
    Mouse = ffi::AMOTION_EVENT_TOOL_TYPE_MOUSE as i32,
    #[doc(alias = "AMOTION_EVENT_TOOL_TYPE_ERASER")]
    Eraser = ffi::AMOTION_EVENT_TOOL_TYPE_ERASER as i32,
    #[doc(alias = "AMOTION_EVENT_TOOL_TYPE_PALM")]
    Palm = ffi::AMOTION_EVENT_TOOL_TYPE_PALM as i32,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(i32),
}

/// Constants that identify different gesture classification types.
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
#[non_exhaustive]
#[doc(alias = "AMotionClassification")]
pub enum MotionClassification {
    /// No additional information is available about the current motion event stream.
    #[doc(alias = "AMOTION_EVENT_CLASSIFICATION_NONE")]
    None = 0,
    /// The user's intent with respect to the current event stream is not yet determined. Events
    /// starting in #AMOTION_EVENT_CLASSIFICATION_AMBIGUOUS_GESTURE will eventually resolve into
    /// either [`MotionClassification::DeepPress`] or [`MotionClassification::None`].
    /// Gestural actions, such as scrolling, should be inhibited until the classification resolves
    /// to another value or the event stream ends.
    #[doc(alias = "AMOTION_EVENT_CLASSIFICATION_AMBIGUOUS_GESTURE")]
    AmbiguousGesture = 1,
    /// The current event stream represents the user intentionally pressing harder on the screen.
    /// This classification type should be used to accelerate the long press behaviour.
    #[doc(alias = "AMOTION_EVENT_CLASSIFICATION_DEEP_PRESS")]
    DeepPress = 2,
    /// The current event stream represents the user swiping with two fingers on a touchpad.
    #[doc(alias = "AMOTION_EVENT_CLASSIFICATION_TWO_FINGER_SWIPE")]
    TwoFingerSwipe = 3,
    /// The current event stream represents the user swiping with three or more fingers on a
    /// touchpad. Unlike two-finger swipes, these are only to be handled by the system UI, which is
    /// why they have a separate constant from two-finger swipes.
    #[doc(alias = "AMOTION_EVENT_CLASSIFICATION_MULTI_FINGER_SWIPE")]
    MultiFingerSwipe = 4,
    /// The current event stream represents the user pinching with two fingers on a touchpad. The
    /// gesture is centered around the current cursor position.
    #[doc(alias = "AMOTION_EVENT_CLASSIFICATION_PINCH")]
    Pinch = 5,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(u32),
}

/// A bitfield representing the state of buttons during a motion event.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ButtonState(pub u32);

impl ButtonState {
    #[inline]
    #[doc(alias = "AMOTION_EVENT_BUTTON_PRIMARY")]
    pub fn primary(self) -> bool {
        self.0 & ffi::AMOTION_EVENT_BUTTON_PRIMARY != 0
    }
    #[inline]
    #[doc(alias = "AMOTION_EVENT_BUTTON_SECONDARY")]
    pub fn secondary(self) -> bool {
        self.0 & ffi::AMOTION_EVENT_BUTTON_SECONDARY != 0
    }
    #[inline]
    #[doc(alias = "AMOTION_EVENT_BUTTON_TERTIARY")]
    pub fn teriary(self) -> bool {
        self.0 & ffi::AMOTION_EVENT_BUTTON_TERTIARY != 0
    }
    #[inline]
    #[doc(alias = "AMOTION_EVENT_BUTTON_BACK")]
    pub fn back(self) -> bool {
        self.0 & ffi::AMOTION_EVENT_BUTTON_BACK != 0
    }
    #[inline]
    #[doc(alias = "AMOTION_EVENT_BUTTON_FORWARD")]
    pub fn forward(self) -> bool {
        self.0 & ffi::AMOTION_EVENT_BUTTON_FORWARD != 0
    }
    #[inline]
    #[doc(alias = "AMOTION_EVENT_BUTTON_STYLUS_PRIMARY")]
    pub fn stylus_primary(self) -> bool {
        self.0 & ffi::AMOTION_EVENT_BUTTON_STYLUS_PRIMARY != 0
    }
    #[inline]
    #[doc(alias = "AMOTION_EVENT_BUTTON_STYLUS_SECONDARY")]
    pub fn stylus_secondary(self) -> bool {
        self.0 & ffi::AMOTION_EVENT_BUTTON_STYLUS_SECONDARY != 0
    }
}

/// A bitfield representing which edges were touched by a motion event.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EdgeFlags(pub u32);

impl EdgeFlags {
    #[inline]
    #[doc(alias = "AMOTION_EVENT_EDGE_FLAG_TOP")]
    pub fn top(self) -> bool {
        self.0 & ffi::AMOTION_EVENT_EDGE_FLAG_TOP != 0
    }
    #[inline]
    #[doc(alias = "AMOTION_EVENT_EDGE_FLAG_BOTTOM")]
    pub fn bottom(self) -> bool {
        self.0 & ffi::AMOTION_EVENT_EDGE_FLAG_BOTTOM != 0
    }
    #[inline]
    #[doc(alias = "AMOTION_EVENT_EDGE_FLAG_LEFT")]
    pub fn left(self) -> bool {
        self.0 & ffi::AMOTION_EVENT_EDGE_FLAG_LEFT != 0
    }
    #[inline]
    #[doc(alias = "AMOTION_EVENT_EDGE_FLAG_RIGHT")]
    pub fn right(self) -> bool {
        self.0 & ffi::AMOTION_EVENT_EDGE_FLAG_RIGHT != 0
    }
}

/// Flags associated with this [`MotionEvent`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MotionEventFlags(pub u32);

impl MotionEventFlags {
    #[inline]
    #[doc(alias = "AMOTION_EVENT_FLAG_WINDOW_IS_OBSCURED")]
    pub fn window_is_obscured(self) -> bool {
        self.0 & ffi::AMOTION_EVENT_FLAG_WINDOW_IS_OBSCURED != 0
    }
}

impl MotionEvent {
    /// Constructs a MotionEvent from a pointer to a native [`ffi::AInputEvent`]
    ///
    /// # Safety
    /// By calling this method, you assert that the pointer is a valid, non-null pointer to a
    /// native [`ffi::AInputEvent`] and that [`ffi::AInputEvent`]
    /// is an `AMotionEvent`.
    #[inline]
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AInputEvent>) -> Self {
        Self { ptr }
    }

    /// Creates a native [`InputEvent`] object that is a copy of the specified
    /// Java [`android.view.MotionEvent`]. The result may be used with generic and
    /// [`MotionEvent`]-specific functions.
    ///
    /// # Safety
    ///
    /// This function should be called with a healthy JVM pointer and with a non-null
    /// [`android.view.MotionEvent`].
    ///
    /// [`android.view.MotionEvent`]: https://developer.android.com/reference/android/view/MotionEvent
    #[cfg(feature = "api-level-31")]
    #[doc(alias = "AMotionEvent_fromJava")]
    pub unsafe fn from_java(env: *mut JNIEnv, key_event: jobject) -> Option<InputEventJava> {
        let ptr = unsafe { ffi::AMotionEvent_fromJava(env, key_event) };
        Some(InputEventJava(InputEvent::MotionEvent(Self::from_ptr(
            NonNull::new(ptr.cast_mut())?,
        ))))
    }

    /// Returns a pointer to the native [`ffi::AInputEvent`].
    #[inline]
    pub fn ptr(&self) -> NonNull<ffi::AInputEvent> {
        self.ptr
    }

    /// Get the source of the event.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#ainputevent_getsource)
    #[inline]
    #[doc(alias = "AInputEvent_getSource")]
    pub fn source(&self) -> Source {
        let source = unsafe { ffi::AInputEvent_getSource(self.ptr.as_ptr()) };
        source.into()
    }

    /// Get the device id associated with the event.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#ainputevent_getdeviceid)
    #[inline]
    #[doc(alias = "AInputEvent_getDeviceId")]
    pub fn device_id(&self) -> i32 {
        unsafe { ffi::AInputEvent_getDeviceId(self.ptr.as_ptr()) }
    }

    /// Returns the motion action associated with the event.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_getaction)
    #[inline]
    #[doc(alias = "AMotionEvent_getAction")]
    #[doc(alias = "AMOTION_EVENT_ACTION_MASK")]
    pub fn action(&self) -> MotionAction {
        let action = unsafe { ffi::AMotionEvent_getAction(self.ptr.as_ptr()) }
            & ffi::AMOTION_EVENT_ACTION_MASK as i32;
        action.into()
    }

    /// Returns the pointer index of an `Up` or `Down` event.
    ///
    /// Pointer indices can change per motion event.  For an identifier that stays the same, see
    /// [`Pointer::pointer_id()`].
    ///
    /// This only has a meaning when the [action][Self::action] is one of [`Up`][MotionAction::Up],
    /// [`Down`][MotionAction::Down], [`PointerUp`][MotionAction::PointerUp],
    /// or [`PointerDown`][MotionAction::PointerDown].
    #[inline]
    #[doc(alias = "AMotionEvent_getAction")]
    #[doc(alias = "AMOTION_EVENT_ACTION_POINTER_INDEX_MASK")]
    #[doc(alias = "AMOTION_EVENT_ACTION_POINTER_INDEX_SHIFT")]
    pub fn pointer_index(&self) -> usize {
        let action = unsafe { ffi::AMotionEvent_getAction(self.ptr.as_ptr()) as u32 };
        let index = (action & ffi::AMOTION_EVENT_ACTION_POINTER_INDEX_MASK)
            >> ffi::AMOTION_EVENT_ACTION_POINTER_INDEX_SHIFT;
        index as usize
    }

    /*
    /// Returns the pointer id associated with the given pointer index.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_getpointerid)
    // TODO: look at output with out-of-range pointer index
    // Probably -1 though
    #[doc(alias = "AMotionEvent_getPointerId")]
    pub fn pointer_id_for(&self, pointer_index: usize) -> i32 {
        unsafe { ffi::AMotionEvent_getPointerId(self.ptr.as_ptr(), pointer_index) }
    }
    */

    /// Returns the number of pointers in this event
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_getpointercount)
    #[inline]
    #[doc(alias = "AMotionEvent_getPointerCount")]
    pub fn pointer_count(&self) -> usize {
        unsafe { ffi::AMotionEvent_getPointerCount(self.ptr.as_ptr()) }
    }

    /// An iterator over the pointers in this motion event
    #[inline]
    #[doc(alias = "AMotionEvent_getPointerCount")]
    pub fn pointers(&self) -> PointersIter<'_> {
        PointersIter {
            event: self.ptr,
            next_index: 0,
            count: self.pointer_count(),
            _marker: std::marker::PhantomData,
        }
    }

    /// The pointer at a given pointer index. Panics if the pointer index is out of bounds.
    ///
    /// If you need to loop over all the pointers, prefer the [`pointers()`][Self::pointers] method.
    #[inline]
    pub fn pointer_at_index(&self, index: usize) -> Pointer<'_> {
        if index >= self.pointer_count() {
            panic!("Pointer index {} is out of bounds", index);
        }
        Pointer {
            event: self.ptr,
            index,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the size of the history contained in this event.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_gethistorysize)
    #[inline]
    #[doc(alias = "AMotionEvent_getHistorySize")]
    pub fn history_size(&self) -> usize {
        unsafe { ffi::AMotionEvent_getHistorySize(self.ptr.as_ptr()) }
    }

    /// An iterator over the historical events contained in this event.
    #[inline]
    #[doc(alias = "AMotionEvent_getHistorySize")]
    pub fn history(&self) -> HistoricalMotionEventsIter<'_> {
        HistoricalMotionEventsIter {
            event: self.ptr,
            next_history_index: 0,
            history_size: self.history_size(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the state of any modifier keys that were pressed during the event.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_getmetastate)
    #[inline]
    #[doc(alias = "AMotionEvent_getMetaState")]
    pub fn meta_state(&self) -> MetaState {
        MetaState(unsafe { ffi::AMotionEvent_getMetaState(self.ptr.as_ptr()) } as u32)
    }

    /// Returns the button state during this event, as a bitfield.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_getbuttonstate)
    #[inline]
    #[doc(alias = "AMotionEvent_getButtonState")]
    pub fn button_state(&self) -> ButtonState {
        ButtonState(unsafe { ffi::AMotionEvent_getButtonState(self.ptr.as_ptr()) } as u32)
    }

    /// Returns the time of the start of this gesture, in the `java.lang.System.nanoTime()` time
    /// base
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_getdowntime)
    #[inline]
    #[doc(alias = "AMotionEvent_getDownTime")]
    pub fn down_time(&self) -> i64 {
        unsafe { ffi::AMotionEvent_getDownTime(self.ptr.as_ptr()) }
    }

    /// Returns a bitfield indicating which edges were touched by this event.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_getedgeflags)
    #[inline]
    #[doc(alias = "AMotionEvent_getEdgeFlags")]
    pub fn edge_flags(&self) -> EdgeFlags {
        EdgeFlags(unsafe { ffi::AMotionEvent_getEdgeFlags(self.ptr.as_ptr()) } as u32)
    }

    /// Returns the time of this event, in the `java.lang.System.nanoTime()` time base
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_geteventtime)
    #[inline]
    #[doc(alias = "AMotionEvent_getEventTime")]
    pub fn event_time(&self) -> i64 {
        unsafe { ffi::AMotionEvent_getEventTime(self.ptr.as_ptr()) }
    }

    /// The flags associated with a motion event.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_getflags)
    #[inline]
    #[doc(alias = "AMotionEvent_getFlags")]
    pub fn flags(&self) -> MotionEventFlags {
        MotionEventFlags(unsafe { ffi::AMotionEvent_getFlags(self.ptr.as_ptr()) } as u32)
    }

    /// Returns the offset in the x direction between the coordinates and the raw coordinates
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_getxoffset)
    #[inline]
    #[doc(alias = "AMotionEvent_getXOffset")]
    pub fn x_offset(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getXOffset(self.ptr.as_ptr()) }
    }

    /// Returns the offset in the y direction between the coordinates and the raw coordinates
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_getyoffset)
    #[inline]
    #[doc(alias = "AMotionEvent_getYOffset")]
    pub fn y_offset(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getYOffset(self.ptr.as_ptr()) }
    }

    /// Returns the precision of the x value of the coordinates
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_getxprecision)
    #[inline]
    #[doc(alias = "AMotionEvent_getXPrecision")]
    pub fn x_precision(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getXPrecision(self.ptr.as_ptr()) }
    }

    /// Returns the precision of the y value of the coordinates
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_getyprecision)
    #[inline]
    #[doc(alias = "AMotionEvent_getYPrecision")]
    pub fn y_precision(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getYPrecision(self.ptr.as_ptr()) }
    }

    /// Get the action button for the motion event. Returns a valid action button when the event is
    /// associated with a button press or button release action. For other actions the return value
    /// is undefined.
    #[cfg(feature = "api-level-33")]
    #[doc(alias = "AMotionEvent_getActionButton")]
    pub fn action_button(&self) -> ButtonState {
        ButtonState(unsafe { ffi::AMotionEvent_getActionButton(self.ptr.as_ptr()) } as u32)
    }

    /// Returns the classification for the current gesture. The classification may change as more
    /// events become available for the same gesture.
    #[cfg(feature = "api-level-33")]
    #[doc(alias = "AMotionEvent_getClassification")]
    pub fn classification(&self) -> MotionClassification {
        u32::try_from(unsafe { ffi::AMotionEvent_getClassification(self.ptr.as_ptr()) })
            .unwrap()
            .into()
    }
}

/// A view into the data of a specific pointer in a motion event.
#[derive(Debug)]
pub struct Pointer<'a> {
    event: NonNull<ffi::AInputEvent>,
    index: usize,
    _marker: std::marker::PhantomData<&'a MotionEvent>,
}

// TODO: thread safety?

impl Pointer<'_> {
    #[inline]
    pub fn pointer_index(&self) -> usize {
        self.index
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getPointerId")]
    pub fn pointer_id(&self) -> i32 {
        unsafe { ffi::AMotionEvent_getPointerId(self.event.as_ptr(), self.index) }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getAxisValue")]
    pub fn axis_value(&self, axis: Axis) -> f32 {
        unsafe { ffi::AMotionEvent_getAxisValue(self.event.as_ptr(), axis.into(), self.index) }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getOrientation")]
    pub fn orientation(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getOrientation(self.event.as_ptr(), self.index) }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getPressure")]
    pub fn pressure(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getPressure(self.event.as_ptr(), self.index) }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getRawX")]
    pub fn raw_x(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getRawX(self.event.as_ptr(), self.index) }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getRawY")]
    pub fn raw_y(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getRawY(self.event.as_ptr(), self.index) }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getX")]
    pub fn x(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getX(self.event.as_ptr(), self.index) }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getY")]
    pub fn y(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getY(self.event.as_ptr(), self.index) }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getSize")]
    pub fn size(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getSize(self.event.as_ptr(), self.index) }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getToolMajor")]
    pub fn tool_major(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getToolMajor(self.event.as_ptr(), self.index) }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getToolMinor")]
    pub fn tool_minor(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getToolMinor(self.event.as_ptr(), self.index) }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getTouchMajor")]
    pub fn touch_major(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getTouchMajor(self.event.as_ptr(), self.index) }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getTouchMinor")]
    pub fn touch_minor(&self) -> f32 {
        unsafe { ffi::AMotionEvent_getTouchMinor(self.event.as_ptr(), self.index) }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getToolType")]
    pub fn tool_type(&self) -> ToolType {
        let tool_type = unsafe { ffi::AMotionEvent_getToolType(self.event.as_ptr(), self.index) };
        tool_type.into()
    }
}

/// An iterator over the pointers in a [`MotionEvent`].
#[derive(Debug)]
pub struct PointersIter<'a> {
    event: NonNull<ffi::AInputEvent>,
    next_index: usize,
    count: usize,
    _marker: std::marker::PhantomData<&'a MotionEvent>,
}

// TODO: thread safety?

impl<'a> Iterator for PointersIter<'a> {
    type Item = Pointer<'a>;
    fn next(&mut self) -> Option<Pointer<'a>> {
        if self.next_index < self.count {
            let ptr = Pointer {
                event: self.event,
                index: self.next_index,
                _marker: std::marker::PhantomData,
            };
            self.next_index += 1;
            Some(ptr)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.count - self.next_index;
        (size, Some(size))
    }
}
impl ExactSizeIterator for PointersIter<'_> {
    fn len(&self) -> usize {
        self.count - self.next_index
    }
}

/// Represents a view into a past moment of a motion event
#[derive(Debug)]
pub struct HistoricalMotionEvent<'a> {
    event: NonNull<ffi::AInputEvent>,
    history_index: usize,
    _marker: std::marker::PhantomData<&'a MotionEvent>,
}

// TODO: thread safety?

impl<'a> HistoricalMotionEvent<'a> {
    /// Returns the "history index" associated with this historical event.  Older events have smaller indices.
    #[inline]
    pub fn history_index(&self) -> usize {
        self.history_index
    }

    /// Returns the time of the historical event, in the `java.lang.System.nanoTime()` time base
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#amotionevent_gethistoricaleventtime)
    #[inline]
    #[doc(alias = "AMotionEvent_getHistoricalEventTime")]
    pub fn event_time(&self) -> i64 {
        unsafe { ffi::AMotionEvent_getHistoricalEventTime(self.event.as_ptr(), self.history_index) }
    }

    /// An iterator over the pointers of this historical motion event
    #[inline]
    #[doc(alias = "MotionEvent_getPointerCount")]
    pub fn pointers(&self) -> HistoricalPointersIter<'a> {
        HistoricalPointersIter {
            event: self.event,
            history_index: self.history_index,
            next_pointer_index: 0,
            pointer_count: unsafe { ffi::AMotionEvent_getPointerCount(self.event.as_ptr()) },
            _marker: std::marker::PhantomData,
        }
    }
}

/// An iterator over all the historical moments in a [`MotionEvent`].
///
/// It iterates from oldest to newest.
#[derive(Debug)]
pub struct HistoricalMotionEventsIter<'a> {
    event: NonNull<ffi::AInputEvent>,
    next_history_index: usize,
    history_size: usize,
    _marker: std::marker::PhantomData<&'a MotionEvent>,
}

// TODO: thread safety?

impl<'a> Iterator for HistoricalMotionEventsIter<'a> {
    type Item = HistoricalMotionEvent<'a>;

    fn next(&mut self) -> Option<HistoricalMotionEvent<'a>> {
        if self.next_history_index < self.history_size {
            let res = HistoricalMotionEvent {
                event: self.event,
                history_index: self.next_history_index,
                _marker: std::marker::PhantomData,
            };
            self.next_history_index += 1;
            Some(res)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.history_size - self.next_history_index;
        (size, Some(size))
    }
}
impl ExactSizeIterator for HistoricalMotionEventsIter<'_> {
    fn len(&self) -> usize {
        self.history_size - self.next_history_index
    }
}
impl<'a> DoubleEndedIterator for HistoricalMotionEventsIter<'a> {
    fn next_back(&mut self) -> Option<HistoricalMotionEvent<'a>> {
        if self.next_history_index < self.history_size {
            self.history_size -= 1;
            Some(HistoricalMotionEvent {
                event: self.event,
                history_index: self.history_size,
                _marker: std::marker::PhantomData,
            })
        } else {
            None
        }
    }
}

/// A view into a pointer at a historical moment
#[derive(Debug)]
pub struct HistoricalPointer<'a> {
    event: NonNull<ffi::AInputEvent>,
    pointer_index: usize,
    history_index: usize,
    _marker: std::marker::PhantomData<&'a MotionEvent>,
}

// TODO: thread safety?

impl HistoricalPointer<'_> {
    #[inline]
    pub fn pointer_index(&self) -> usize {
        self.pointer_index
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getPointerId")]
    pub fn pointer_id(&self) -> i32 {
        unsafe { ffi::AMotionEvent_getPointerId(self.event.as_ptr(), self.pointer_index) }
    }

    #[inline]
    pub fn history_index(&self) -> usize {
        self.history_index
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getHistoricalAxisValue")]
    pub fn axis_value(&self, axis: Axis) -> f32 {
        unsafe {
            ffi::AMotionEvent_getHistoricalAxisValue(
                self.event.as_ptr(),
                axis.into(),
                self.pointer_index,
                self.history_index,
            )
        }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getHistoricalOrientation")]
    pub fn orientation(&self) -> f32 {
        unsafe {
            ffi::AMotionEvent_getHistoricalOrientation(
                self.event.as_ptr(),
                self.pointer_index,
                self.history_index,
            )
        }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getHistoricalPressure")]
    pub fn pressure(&self) -> f32 {
        unsafe {
            ffi::AMotionEvent_getHistoricalPressure(
                self.event.as_ptr(),
                self.pointer_index,
                self.history_index,
            )
        }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getHistoricalRawX")]
    pub fn raw_x(&self) -> f32 {
        unsafe {
            ffi::AMotionEvent_getHistoricalRawX(
                self.event.as_ptr(),
                self.pointer_index,
                self.history_index,
            )
        }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getHistoricalRawY")]
    pub fn raw_y(&self) -> f32 {
        unsafe {
            ffi::AMotionEvent_getHistoricalRawY(
                self.event.as_ptr(),
                self.pointer_index,
                self.history_index,
            )
        }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getHistoricalX")]
    pub fn x(&self) -> f32 {
        unsafe {
            ffi::AMotionEvent_getHistoricalX(
                self.event.as_ptr(),
                self.pointer_index,
                self.history_index,
            )
        }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getHistoricalY")]
    pub fn y(&self) -> f32 {
        unsafe {
            ffi::AMotionEvent_getHistoricalY(
                self.event.as_ptr(),
                self.pointer_index,
                self.history_index,
            )
        }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getHistoricalSize")]
    pub fn size(&self) -> f32 {
        unsafe {
            ffi::AMotionEvent_getHistoricalSize(
                self.event.as_ptr(),
                self.pointer_index,
                self.history_index,
            )
        }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getHistoricalToolMajor")]
    pub fn tool_major(&self) -> f32 {
        unsafe {
            ffi::AMotionEvent_getHistoricalToolMajor(
                self.event.as_ptr(),
                self.pointer_index,
                self.history_index,
            )
        }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getHistoricalToolMinor")]
    pub fn tool_minor(&self) -> f32 {
        unsafe {
            ffi::AMotionEvent_getHistoricalToolMinor(
                self.event.as_ptr(),
                self.pointer_index,
                self.history_index,
            )
        }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getHistoricalTouchMajor")]
    pub fn touch_major(&self) -> f32 {
        unsafe {
            ffi::AMotionEvent_getHistoricalTouchMajor(
                self.event.as_ptr(),
                self.pointer_index,
                self.history_index,
            )
        }
    }

    #[inline]
    #[doc(alias = "AMotionEvent_getHistoricalTouchMinor")]
    pub fn touch_minor(&self) -> f32 {
        unsafe {
            ffi::AMotionEvent_getHistoricalTouchMinor(
                self.event.as_ptr(),
                self.pointer_index,
                self.history_index,
            )
        }
    }
}

/// An iterator over the pointers in a historical motion event
#[derive(Debug)]
pub struct HistoricalPointersIter<'a> {
    event: NonNull<ffi::AInputEvent>,
    history_index: usize,
    next_pointer_index: usize,
    pointer_count: usize,
    _marker: std::marker::PhantomData<&'a MotionEvent>,
}

// TODO: thread safety?

impl<'a> Iterator for HistoricalPointersIter<'a> {
    type Item = HistoricalPointer<'a>;

    fn next(&mut self) -> Option<HistoricalPointer<'a>> {
        if self.next_pointer_index < self.pointer_count {
            let ptr = HistoricalPointer {
                event: self.event,
                history_index: self.history_index,
                pointer_index: self.next_pointer_index,
                _marker: std::marker::PhantomData,
            };
            self.next_pointer_index += 1;
            Some(ptr)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.pointer_count - self.next_pointer_index;
        (size, Some(size))
    }
}
impl ExactSizeIterator for HistoricalPointersIter<'_> {
    fn len(&self) -> usize {
        self.pointer_count - self.next_pointer_index
    }
}

/// A key event
///
/// Wraps an [`AInputEvent *`] of the [`ffi::AINPUT_EVENT_TYPE_KEY`] type.
///
/// For general discussion of key events in Android, see [the relevant
/// javadoc](https://developer.android.com/reference/android/view/KeyEvent).
///
/// [`AInputEvent *`]: https://developer.android.com/ndk/reference/group/input#ainputevent
#[derive(Debug)]
#[doc(alias = "AINPUT_EVENT_TYPE_KEY")]
pub struct KeyEvent {
    ptr: NonNull<ffi::AInputEvent>,
}

// TODO: thread safety?

/// Key actions.
/// See [the NDK docs](https://developer.android.com/ndk/reference/group/input#anonymous-enum-27)
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(i32)]
#[non_exhaustive]
pub enum KeyAction {
    Down = ffi::AKEY_EVENT_ACTION_DOWN as i32,
    Up = ffi::AKEY_EVENT_ACTION_UP as i32,
    Multiple = ffi::AKEY_EVENT_ACTION_MULTIPLE as i32,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(i32),
}

/// Key codes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(i32)]
#[non_exhaustive]
pub enum Keycode {
    #[doc(alias = "AKEYCODE_UNKNOWN")]
    Unknown = ffi::AKEYCODE_UNKNOWN as i32,
    #[doc(alias = "AKEYCODE_SOFT_LEFT")]
    SoftLeft = ffi::AKEYCODE_SOFT_LEFT as i32,
    #[doc(alias = "AKEYCODE_SOFT_RIGHT")]
    SoftRight = ffi::AKEYCODE_SOFT_RIGHT as i32,
    #[doc(alias = "AKEYCODE_HOME")]
    Home = ffi::AKEYCODE_HOME as i32,
    #[doc(alias = "AKEYCODE_BACK")]
    Back = ffi::AKEYCODE_BACK as i32,
    #[doc(alias = "AKEYCODE_CALL")]
    Call = ffi::AKEYCODE_CALL as i32,
    #[doc(alias = "AKEYCODE_ENDCALL")]
    Endcall = ffi::AKEYCODE_ENDCALL as i32,
    #[doc(alias = "AKEYCODE_0")]
    Keycode0 = ffi::AKEYCODE_0 as i32,
    #[doc(alias = "AKEYCODE_1")]
    Keycode1 = ffi::AKEYCODE_1 as i32,
    #[doc(alias = "AKEYCODE_2")]
    Keycode2 = ffi::AKEYCODE_2 as i32,
    #[doc(alias = "AKEYCODE_3")]
    Keycode3 = ffi::AKEYCODE_3 as i32,
    #[doc(alias = "AKEYCODE_4")]
    Keycode4 = ffi::AKEYCODE_4 as i32,
    #[doc(alias = "AKEYCODE_5")]
    Keycode5 = ffi::AKEYCODE_5 as i32,
    #[doc(alias = "AKEYCODE_6")]
    Keycode6 = ffi::AKEYCODE_6 as i32,
    #[doc(alias = "AKEYCODE_7")]
    Keycode7 = ffi::AKEYCODE_7 as i32,
    #[doc(alias = "AKEYCODE_8")]
    Keycode8 = ffi::AKEYCODE_8 as i32,
    #[doc(alias = "AKEYCODE_9")]
    Keycode9 = ffi::AKEYCODE_9 as i32,
    #[doc(alias = "AKEYCODE_STAR")]
    Star = ffi::AKEYCODE_STAR as i32,
    #[doc(alias = "AKEYCODE_POUND")]
    Pound = ffi::AKEYCODE_POUND as i32,
    #[doc(alias = "AKEYCODE_DPAD_UP")]
    DpadUp = ffi::AKEYCODE_DPAD_UP as i32,
    #[doc(alias = "AKEYCODE_DPAD_DOWN")]
    DpadDown = ffi::AKEYCODE_DPAD_DOWN as i32,
    #[doc(alias = "AKEYCODE_DPAD_LEFT")]
    DpadLeft = ffi::AKEYCODE_DPAD_LEFT as i32,
    #[doc(alias = "AKEYCODE_DPAD_RIGHT")]
    DpadRight = ffi::AKEYCODE_DPAD_RIGHT as i32,
    #[doc(alias = "AKEYCODE_DPAD_CENTER")]
    DpadCenter = ffi::AKEYCODE_DPAD_CENTER as i32,
    #[doc(alias = "AKEYCODE_VOLUME_UP")]
    VolumeUp = ffi::AKEYCODE_VOLUME_UP as i32,
    #[doc(alias = "AKEYCODE_VOLUME_DOWN")]
    VolumeDown = ffi::AKEYCODE_VOLUME_DOWN as i32,
    #[doc(alias = "AKEYCODE_POWER")]
    Power = ffi::AKEYCODE_POWER as i32,
    #[doc(alias = "AKEYCODE_CAMERA")]
    Camera = ffi::AKEYCODE_CAMERA as i32,
    #[doc(alias = "AKEYCODE_CLEAR")]
    Clear = ffi::AKEYCODE_CLEAR as i32,
    #[doc(alias = "AKEYCODE_A")]
    A = ffi::AKEYCODE_A as i32,
    #[doc(alias = "AKEYCODE_B")]
    B = ffi::AKEYCODE_B as i32,
    #[doc(alias = "AKEYCODE_C")]
    C = ffi::AKEYCODE_C as i32,
    #[doc(alias = "AKEYCODE_D")]
    D = ffi::AKEYCODE_D as i32,
    #[doc(alias = "AKEYCODE_E")]
    E = ffi::AKEYCODE_E as i32,
    #[doc(alias = "AKEYCODE_F")]
    F = ffi::AKEYCODE_F as i32,
    #[doc(alias = "AKEYCODE_G")]
    G = ffi::AKEYCODE_G as i32,
    #[doc(alias = "AKEYCODE_H")]
    H = ffi::AKEYCODE_H as i32,
    #[doc(alias = "AKEYCODE_I")]
    I = ffi::AKEYCODE_I as i32,
    #[doc(alias = "AKEYCODE_J")]
    J = ffi::AKEYCODE_J as i32,
    #[doc(alias = "AKEYCODE_K")]
    K = ffi::AKEYCODE_K as i32,
    #[doc(alias = "AKEYCODE_L")]
    L = ffi::AKEYCODE_L as i32,
    #[doc(alias = "AKEYCODE_M")]
    M = ffi::AKEYCODE_M as i32,
    #[doc(alias = "AKEYCODE_N")]
    N = ffi::AKEYCODE_N as i32,
    #[doc(alias = "AKEYCODE_O")]
    O = ffi::AKEYCODE_O as i32,
    #[doc(alias = "AKEYCODE_P")]
    P = ffi::AKEYCODE_P as i32,
    #[doc(alias = "AKEYCODE_Q")]
    Q = ffi::AKEYCODE_Q as i32,
    #[doc(alias = "AKEYCODE_R")]
    R = ffi::AKEYCODE_R as i32,
    #[doc(alias = "AKEYCODE_S")]
    S = ffi::AKEYCODE_S as i32,
    #[doc(alias = "AKEYCODE_T")]
    T = ffi::AKEYCODE_T as i32,
    #[doc(alias = "AKEYCODE_U")]
    U = ffi::AKEYCODE_U as i32,
    #[doc(alias = "AKEYCODE_V")]
    V = ffi::AKEYCODE_V as i32,
    #[doc(alias = "AKEYCODE_W")]
    W = ffi::AKEYCODE_W as i32,
    #[doc(alias = "AKEYCODE_X")]
    X = ffi::AKEYCODE_X as i32,
    #[doc(alias = "AKEYCODE_Y")]
    Y = ffi::AKEYCODE_Y as i32,
    #[doc(alias = "AKEYCODE_Z")]
    Z = ffi::AKEYCODE_Z as i32,
    #[doc(alias = "AKEYCODE_COMMA")]
    Comma = ffi::AKEYCODE_COMMA as i32,
    #[doc(alias = "AKEYCODE_PERIOD")]
    Period = ffi::AKEYCODE_PERIOD as i32,
    #[doc(alias = "AKEYCODE_ALT_LEFT")]
    AltLeft = ffi::AKEYCODE_ALT_LEFT as i32,
    #[doc(alias = "AKEYCODE_ALT_RIGHT")]
    AltRight = ffi::AKEYCODE_ALT_RIGHT as i32,
    #[doc(alias = "AKEYCODE_SHIFT_LEFT")]
    ShiftLeft = ffi::AKEYCODE_SHIFT_LEFT as i32,
    #[doc(alias = "AKEYCODE_SHIFT_RIGHT")]
    ShiftRight = ffi::AKEYCODE_SHIFT_RIGHT as i32,
    #[doc(alias = "AKEYCODE_TAB")]
    Tab = ffi::AKEYCODE_TAB as i32,
    #[doc(alias = "AKEYCODE_SPACE")]
    Space = ffi::AKEYCODE_SPACE as i32,
    #[doc(alias = "AKEYCODE_SYM")]
    Sym = ffi::AKEYCODE_SYM as i32,
    #[doc(alias = "AKEYCODE_EXPLORER")]
    Explorer = ffi::AKEYCODE_EXPLORER as i32,
    #[doc(alias = "AKEYCODE_ENVELOPE")]
    Envelope = ffi::AKEYCODE_ENVELOPE as i32,
    #[doc(alias = "AKEYCODE_ENTER")]
    Enter = ffi::AKEYCODE_ENTER as i32,
    #[doc(alias = "AKEYCODE_DEL")]
    Del = ffi::AKEYCODE_DEL as i32,
    #[doc(alias = "AKEYCODE_GRAVE")]
    Grave = ffi::AKEYCODE_GRAVE as i32,
    #[doc(alias = "AKEYCODE_MINUS")]
    Minus = ffi::AKEYCODE_MINUS as i32,
    #[doc(alias = "AKEYCODE_EQUALS")]
    Equals = ffi::AKEYCODE_EQUALS as i32,
    #[doc(alias = "AKEYCODE_LEFT_BRACKET")]
    LeftBracket = ffi::AKEYCODE_LEFT_BRACKET as i32,
    #[doc(alias = "AKEYCODE_RIGHT_BRACKET")]
    RightBracket = ffi::AKEYCODE_RIGHT_BRACKET as i32,
    #[doc(alias = "AKEYCODE_BACKSLASH")]
    Backslash = ffi::AKEYCODE_BACKSLASH as i32,
    #[doc(alias = "AKEYCODE_SEMICOLON")]
    Semicolon = ffi::AKEYCODE_SEMICOLON as i32,
    #[doc(alias = "AKEYCODE_APOSTROPHE")]
    Apostrophe = ffi::AKEYCODE_APOSTROPHE as i32,
    #[doc(alias = "AKEYCODE_SLASH")]
    Slash = ffi::AKEYCODE_SLASH as i32,
    #[doc(alias = "AKEYCODE_AT")]
    At = ffi::AKEYCODE_AT as i32,
    #[doc(alias = "AKEYCODE_NUM")]
    Num = ffi::AKEYCODE_NUM as i32,
    #[doc(alias = "AKEYCODE_HEADSETHOOK")]
    Headsethook = ffi::AKEYCODE_HEADSETHOOK as i32,
    #[doc(alias = "AKEYCODE_FOCUS")]
    Focus = ffi::AKEYCODE_FOCUS as i32,
    #[doc(alias = "AKEYCODE_PLUS")]
    Plus = ffi::AKEYCODE_PLUS as i32,
    #[doc(alias = "AKEYCODE_MENU")]
    Menu = ffi::AKEYCODE_MENU as i32,
    #[doc(alias = "AKEYCODE_NOTIFICATION")]
    Notification = ffi::AKEYCODE_NOTIFICATION as i32,
    #[doc(alias = "AKEYCODE_SEARCH")]
    Search = ffi::AKEYCODE_SEARCH as i32,
    #[doc(alias = "AKEYCODE_MEDIA_PLAY_PAUSE")]
    MediaPlayPause = ffi::AKEYCODE_MEDIA_PLAY_PAUSE as i32,
    #[doc(alias = "AKEYCODE_MEDIA_STOP")]
    MediaStop = ffi::AKEYCODE_MEDIA_STOP as i32,
    #[doc(alias = "AKEYCODE_MEDIA_NEXT")]
    MediaNext = ffi::AKEYCODE_MEDIA_NEXT as i32,
    #[doc(alias = "AKEYCODE_MEDIA_PREVIOUS")]
    MediaPrevious = ffi::AKEYCODE_MEDIA_PREVIOUS as i32,
    #[doc(alias = "AKEYCODE_MEDIA_REWIND")]
    MediaRewind = ffi::AKEYCODE_MEDIA_REWIND as i32,
    #[doc(alias = "AKEYCODE_MEDIA_FAST_FORWARD")]
    MediaFastForward = ffi::AKEYCODE_MEDIA_FAST_FORWARD as i32,
    #[doc(alias = "AKEYCODE_MUTE")]
    Mute = ffi::AKEYCODE_MUTE as i32,
    #[doc(alias = "AKEYCODE_PAGE_UP")]
    PageUp = ffi::AKEYCODE_PAGE_UP as i32,
    #[doc(alias = "AKEYCODE_PAGE_DOWN")]
    PageDown = ffi::AKEYCODE_PAGE_DOWN as i32,
    #[doc(alias = "AKEYCODE_PICTSYMBOLS")]
    Pictsymbols = ffi::AKEYCODE_PICTSYMBOLS as i32,
    #[doc(alias = "AKEYCODE_SWITCH_CHARSET")]
    SwitchCharset = ffi::AKEYCODE_SWITCH_CHARSET as i32,
    #[doc(alias = "AKEYCODE_BUTTON_A")]
    ButtonA = ffi::AKEYCODE_BUTTON_A as i32,
    #[doc(alias = "AKEYCODE_BUTTON_B")]
    ButtonB = ffi::AKEYCODE_BUTTON_B as i32,
    #[doc(alias = "AKEYCODE_BUTTON_C")]
    ButtonC = ffi::AKEYCODE_BUTTON_C as i32,
    #[doc(alias = "AKEYCODE_BUTTON_X")]
    ButtonX = ffi::AKEYCODE_BUTTON_X as i32,
    #[doc(alias = "AKEYCODE_BUTTON_Y")]
    ButtonY = ffi::AKEYCODE_BUTTON_Y as i32,
    #[doc(alias = "AKEYCODE_BUTTON_Z")]
    ButtonZ = ffi::AKEYCODE_BUTTON_Z as i32,
    #[doc(alias = "AKEYCODE_BUTTON_L1")]
    ButtonL1 = ffi::AKEYCODE_BUTTON_L1 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_R1")]
    ButtonR1 = ffi::AKEYCODE_BUTTON_R1 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_L2")]
    ButtonL2 = ffi::AKEYCODE_BUTTON_L2 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_R2")]
    ButtonR2 = ffi::AKEYCODE_BUTTON_R2 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_THUMBL")]
    ButtonThumbl = ffi::AKEYCODE_BUTTON_THUMBL as i32,
    #[doc(alias = "AKEYCODE_BUTTON_THUMBR")]
    ButtonThumbr = ffi::AKEYCODE_BUTTON_THUMBR as i32,
    #[doc(alias = "AKEYCODE_BUTTON_START")]
    ButtonStart = ffi::AKEYCODE_BUTTON_START as i32,
    #[doc(alias = "AKEYCODE_BUTTON_SELECT")]
    ButtonSelect = ffi::AKEYCODE_BUTTON_SELECT as i32,
    #[doc(alias = "AKEYCODE_BUTTON_MODE")]
    ButtonMode = ffi::AKEYCODE_BUTTON_MODE as i32,
    #[doc(alias = "AKEYCODE_ESCAPE")]
    Escape = ffi::AKEYCODE_ESCAPE as i32,
    #[doc(alias = "AKEYCODE_FORWARD_DEL")]
    ForwardDel = ffi::AKEYCODE_FORWARD_DEL as i32,
    #[doc(alias = "AKEYCODE_CTRL_LEFT")]
    CtrlLeft = ffi::AKEYCODE_CTRL_LEFT as i32,
    #[doc(alias = "AKEYCODE_CTRL_RIGHT")]
    CtrlRight = ffi::AKEYCODE_CTRL_RIGHT as i32,
    #[doc(alias = "AKEYCODE_CAPS_LOCK")]
    CapsLock = ffi::AKEYCODE_CAPS_LOCK as i32,
    #[doc(alias = "AKEYCODE_SCROLL_LOCK")]
    ScrollLock = ffi::AKEYCODE_SCROLL_LOCK as i32,
    #[doc(alias = "AKEYCODE_META_LEFT")]
    MetaLeft = ffi::AKEYCODE_META_LEFT as i32,
    #[doc(alias = "AKEYCODE_META_RIGHT")]
    MetaRight = ffi::AKEYCODE_META_RIGHT as i32,
    #[doc(alias = "AKEYCODE_FUNCTION")]
    Function = ffi::AKEYCODE_FUNCTION as i32,
    #[doc(alias = "AKEYCODE_SYSRQ")]
    Sysrq = ffi::AKEYCODE_SYSRQ as i32,
    #[doc(alias = "AKEYCODE_BREAK")]
    Break = ffi::AKEYCODE_BREAK as i32,
    #[doc(alias = "AKEYCODE_MOVE_HOME")]
    MoveHome = ffi::AKEYCODE_MOVE_HOME as i32,
    #[doc(alias = "AKEYCODE_MOVE_END")]
    MoveEnd = ffi::AKEYCODE_MOVE_END as i32,
    #[doc(alias = "AKEYCODE_INSERT")]
    Insert = ffi::AKEYCODE_INSERT as i32,
    #[doc(alias = "AKEYCODE_FORWARD")]
    Forward = ffi::AKEYCODE_FORWARD as i32,
    #[doc(alias = "AKEYCODE_MEDIA_PLAY")]
    MediaPlay = ffi::AKEYCODE_MEDIA_PLAY as i32,
    #[doc(alias = "AKEYCODE_MEDIA_PAUSE")]
    MediaPause = ffi::AKEYCODE_MEDIA_PAUSE as i32,
    #[doc(alias = "AKEYCODE_MEDIA_CLOSE")]
    MediaClose = ffi::AKEYCODE_MEDIA_CLOSE as i32,
    #[doc(alias = "AKEYCODE_MEDIA_EJECT")]
    MediaEject = ffi::AKEYCODE_MEDIA_EJECT as i32,
    #[doc(alias = "AKEYCODE_MEDIA_RECORD")]
    MediaRecord = ffi::AKEYCODE_MEDIA_RECORD as i32,
    #[doc(alias = "AKEYCODE_F1")]
    F1 = ffi::AKEYCODE_F1 as i32,
    #[doc(alias = "AKEYCODE_F2")]
    F2 = ffi::AKEYCODE_F2 as i32,
    #[doc(alias = "AKEYCODE_F3")]
    F3 = ffi::AKEYCODE_F3 as i32,
    #[doc(alias = "AKEYCODE_F4")]
    F4 = ffi::AKEYCODE_F4 as i32,
    #[doc(alias = "AKEYCODE_F5")]
    F5 = ffi::AKEYCODE_F5 as i32,
    #[doc(alias = "AKEYCODE_F6")]
    F6 = ffi::AKEYCODE_F6 as i32,
    #[doc(alias = "AKEYCODE_F7")]
    F7 = ffi::AKEYCODE_F7 as i32,
    #[doc(alias = "AKEYCODE_F8")]
    F8 = ffi::AKEYCODE_F8 as i32,
    #[doc(alias = "AKEYCODE_F9")]
    F9 = ffi::AKEYCODE_F9 as i32,
    #[doc(alias = "AKEYCODE_F10")]
    F10 = ffi::AKEYCODE_F10 as i32,
    #[doc(alias = "AKEYCODE_F11")]
    F11 = ffi::AKEYCODE_F11 as i32,
    #[doc(alias = "AKEYCODE_F12")]
    F12 = ffi::AKEYCODE_F12 as i32,
    #[doc(alias = "AKEYCODE_NUM_LOCK")]
    NumLock = ffi::AKEYCODE_NUM_LOCK as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_0")]
    Numpad0 = ffi::AKEYCODE_NUMPAD_0 as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_1")]
    Numpad1 = ffi::AKEYCODE_NUMPAD_1 as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_2")]
    Numpad2 = ffi::AKEYCODE_NUMPAD_2 as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_3")]
    Numpad3 = ffi::AKEYCODE_NUMPAD_3 as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_4")]
    Numpad4 = ffi::AKEYCODE_NUMPAD_4 as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_5")]
    Numpad5 = ffi::AKEYCODE_NUMPAD_5 as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_6")]
    Numpad6 = ffi::AKEYCODE_NUMPAD_6 as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_7")]
    Numpad7 = ffi::AKEYCODE_NUMPAD_7 as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_8")]
    Numpad8 = ffi::AKEYCODE_NUMPAD_8 as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_9")]
    Numpad9 = ffi::AKEYCODE_NUMPAD_9 as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_DIVIDE")]
    NumpadDivide = ffi::AKEYCODE_NUMPAD_DIVIDE as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_MULTIPLY")]
    NumpadMultiply = ffi::AKEYCODE_NUMPAD_MULTIPLY as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_SUBTRACT")]
    NumpadSubtract = ffi::AKEYCODE_NUMPAD_SUBTRACT as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_ADD")]
    NumpadAdd = ffi::AKEYCODE_NUMPAD_ADD as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_DOT")]
    NumpadDot = ffi::AKEYCODE_NUMPAD_DOT as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_COMMA")]
    NumpadComma = ffi::AKEYCODE_NUMPAD_COMMA as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_ENTER")]
    NumpadEnter = ffi::AKEYCODE_NUMPAD_ENTER as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_EQUALS")]
    NumpadEquals = ffi::AKEYCODE_NUMPAD_EQUALS as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_LEFT_PAREN")]
    NumpadLeftParen = ffi::AKEYCODE_NUMPAD_LEFT_PAREN as i32,
    #[doc(alias = "AKEYCODE_NUMPAD_RIGHT_PAREN")]
    NumpadRightParen = ffi::AKEYCODE_NUMPAD_RIGHT_PAREN as i32,
    #[doc(alias = "AKEYCODE_VOLUME_MUTE")]
    VolumeMute = ffi::AKEYCODE_VOLUME_MUTE as i32,
    #[doc(alias = "AKEYCODE_INFO")]
    Info = ffi::AKEYCODE_INFO as i32,
    #[doc(alias = "AKEYCODE_CHANNEL_UP")]
    ChannelUp = ffi::AKEYCODE_CHANNEL_UP as i32,
    #[doc(alias = "AKEYCODE_CHANNEL_DOWN")]
    ChannelDown = ffi::AKEYCODE_CHANNEL_DOWN as i32,
    #[doc(alias = "AKEYCODE_ZOOM_IN")]
    ZoomIn = ffi::AKEYCODE_ZOOM_IN as i32,
    #[doc(alias = "AKEYCODE_ZOOM_OUT")]
    ZoomOut = ffi::AKEYCODE_ZOOM_OUT as i32,
    #[doc(alias = "AKEYCODE_TV")]
    Tv = ffi::AKEYCODE_TV as i32,
    #[doc(alias = "AKEYCODE_WINDOW")]
    Window = ffi::AKEYCODE_WINDOW as i32,
    #[doc(alias = "AKEYCODE_GUIDE")]
    Guide = ffi::AKEYCODE_GUIDE as i32,
    #[doc(alias = "AKEYCODE_DVR")]
    Dvr = ffi::AKEYCODE_DVR as i32,
    #[doc(alias = "AKEYCODE_BOOKMARK")]
    Bookmark = ffi::AKEYCODE_BOOKMARK as i32,
    #[doc(alias = "AKEYCODE_CAPTIONS")]
    Captions = ffi::AKEYCODE_CAPTIONS as i32,
    #[doc(alias = "AKEYCODE_SETTINGS")]
    Settings = ffi::AKEYCODE_SETTINGS as i32,
    #[doc(alias = "AKEYCODE_TV_POWER")]
    TvPower = ffi::AKEYCODE_TV_POWER as i32,
    #[doc(alias = "AKEYCODE_TV_INPUT")]
    TvInput = ffi::AKEYCODE_TV_INPUT as i32,
    #[doc(alias = "AKEYCODE_STB_POWER")]
    StbPower = ffi::AKEYCODE_STB_POWER as i32,
    #[doc(alias = "AKEYCODE_STB_INPUT")]
    StbInput = ffi::AKEYCODE_STB_INPUT as i32,
    #[doc(alias = "AKEYCODE_AVR_POWER")]
    AvrPower = ffi::AKEYCODE_AVR_POWER as i32,
    #[doc(alias = "AKEYCODE_AVR_INPUT")]
    AvrInput = ffi::AKEYCODE_AVR_INPUT as i32,
    #[doc(alias = "AKEYCODE_PROG_RED")]
    ProgRed = ffi::AKEYCODE_PROG_RED as i32,
    #[doc(alias = "AKEYCODE_PROG_GREEN")]
    ProgGreen = ffi::AKEYCODE_PROG_GREEN as i32,
    #[doc(alias = "AKEYCODE_PROG_YELLOW")]
    ProgYellow = ffi::AKEYCODE_PROG_YELLOW as i32,
    #[doc(alias = "AKEYCODE_PROG_BLUE")]
    ProgBlue = ffi::AKEYCODE_PROG_BLUE as i32,
    #[doc(alias = "AKEYCODE_APP_SWITCH")]
    AppSwitch = ffi::AKEYCODE_APP_SWITCH as i32,
    #[doc(alias = "AKEYCODE_BUTTON_1")]
    Button1 = ffi::AKEYCODE_BUTTON_1 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_2")]
    Button2 = ffi::AKEYCODE_BUTTON_2 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_3")]
    Button3 = ffi::AKEYCODE_BUTTON_3 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_4")]
    Button4 = ffi::AKEYCODE_BUTTON_4 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_5")]
    Button5 = ffi::AKEYCODE_BUTTON_5 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_6")]
    Button6 = ffi::AKEYCODE_BUTTON_6 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_7")]
    Button7 = ffi::AKEYCODE_BUTTON_7 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_8")]
    Button8 = ffi::AKEYCODE_BUTTON_8 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_9")]
    Button9 = ffi::AKEYCODE_BUTTON_9 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_10")]
    Button10 = ffi::AKEYCODE_BUTTON_10 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_11")]
    Button11 = ffi::AKEYCODE_BUTTON_11 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_12")]
    Button12 = ffi::AKEYCODE_BUTTON_12 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_13")]
    Button13 = ffi::AKEYCODE_BUTTON_13 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_14")]
    Button14 = ffi::AKEYCODE_BUTTON_14 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_15")]
    Button15 = ffi::AKEYCODE_BUTTON_15 as i32,
    #[doc(alias = "AKEYCODE_BUTTON_16")]
    Button16 = ffi::AKEYCODE_BUTTON_16 as i32,
    #[doc(alias = "AKEYCODE_LANGUAGE_SWITCH")]
    LanguageSwitch = ffi::AKEYCODE_LANGUAGE_SWITCH as i32,
    #[doc(alias = "AKEYCODE_MANNER_MODE")]
    MannerMode = ffi::AKEYCODE_MANNER_MODE as i32,
    #[doc(alias = "AKEYCODE_3D_MODE")]
    Keycode3dMode = ffi::AKEYCODE_3D_MODE as i32,
    #[doc(alias = "AKEYCODE_CONTACTS")]
    Contacts = ffi::AKEYCODE_CONTACTS as i32,
    #[doc(alias = "AKEYCODE_CALENDAR")]
    Calendar = ffi::AKEYCODE_CALENDAR as i32,
    #[doc(alias = "AKEYCODE_MUSIC")]
    Music = ffi::AKEYCODE_MUSIC as i32,
    #[doc(alias = "AKEYCODE_CALCULATOR")]
    Calculator = ffi::AKEYCODE_CALCULATOR as i32,
    #[doc(alias = "AKEYCODE_ZENKAKU_HANKAKU")]
    ZenkakuHankaku = ffi::AKEYCODE_ZENKAKU_HANKAKU as i32,
    #[doc(alias = "AKEYCODE_EISU")]
    Eisu = ffi::AKEYCODE_EISU as i32,
    #[doc(alias = "AKEYCODE_MUHENKAN")]
    Muhenkan = ffi::AKEYCODE_MUHENKAN as i32,
    #[doc(alias = "AKEYCODE_HENKAN")]
    Henkan = ffi::AKEYCODE_HENKAN as i32,
    #[doc(alias = "AKEYCODE_KATAKANA_HIRAGANA")]
    KatakanaHiragana = ffi::AKEYCODE_KATAKANA_HIRAGANA as i32,
    #[doc(alias = "AKEYCODE_YEN")]
    Yen = ffi::AKEYCODE_YEN as i32,
    #[doc(alias = "AKEYCODE_RO")]
    Ro = ffi::AKEYCODE_RO as i32,
    #[doc(alias = "AKEYCODE_KANA")]
    Kana = ffi::AKEYCODE_KANA as i32,
    #[doc(alias = "AKEYCODE_ASSIST")]
    Assist = ffi::AKEYCODE_ASSIST as i32,
    #[doc(alias = "AKEYCODE_BRIGHTNESS_DOWN")]
    BrightnessDown = ffi::AKEYCODE_BRIGHTNESS_DOWN as i32,
    #[doc(alias = "AKEYCODE_BRIGHTNESS_UP")]
    BrightnessUp = ffi::AKEYCODE_BRIGHTNESS_UP as i32,
    #[doc(alias = "AKEYCODE_MEDIA_AUDIO_TRACK")]
    MediaAudioTrack = ffi::AKEYCODE_MEDIA_AUDIO_TRACK as i32,
    #[doc(alias = "AKEYCODE_SLEEP")]
    Sleep = ffi::AKEYCODE_SLEEP as i32,
    #[doc(alias = "AKEYCODE_WAKEUP")]
    Wakeup = ffi::AKEYCODE_WAKEUP as i32,
    #[doc(alias = "AKEYCODE_PAIRING")]
    Pairing = ffi::AKEYCODE_PAIRING as i32,
    #[doc(alias = "AKEYCODE_MEDIA_TOP_MENU")]
    MediaTopMenu = ffi::AKEYCODE_MEDIA_TOP_MENU as i32,
    #[doc(alias = "AKEYCODE_11")]
    Keycode11 = ffi::AKEYCODE_11 as i32,
    #[doc(alias = "AKEYCODE_12")]
    Keycode12 = ffi::AKEYCODE_12 as i32,
    #[doc(alias = "AKEYCODE_LAST_CHANNEL")]
    LastChannel = ffi::AKEYCODE_LAST_CHANNEL as i32,
    #[doc(alias = "AKEYCODE_TV_DATA_SERVICE")]
    TvDataService = ffi::AKEYCODE_TV_DATA_SERVICE as i32,
    #[doc(alias = "AKEYCODE_VOICE_ASSIST")]
    VoiceAssist = ffi::AKEYCODE_VOICE_ASSIST as i32,
    #[doc(alias = "AKEYCODE_TV_RADIO_SERVICE")]
    TvRadioService = ffi::AKEYCODE_TV_RADIO_SERVICE as i32,
    #[doc(alias = "AKEYCODE_TV_TELETEXT")]
    TvTeletext = ffi::AKEYCODE_TV_TELETEXT as i32,
    #[doc(alias = "AKEYCODE_TV_NUMBER_ENTRY")]
    TvNumberEntry = ffi::AKEYCODE_TV_NUMBER_ENTRY as i32,
    #[doc(alias = "AKEYCODE_TV_TERRESTRIAL_ANALOG")]
    TvTerrestrialAnalog = ffi::AKEYCODE_TV_TERRESTRIAL_ANALOG as i32,
    #[doc(alias = "AKEYCODE_TV_TERRESTRIAL_DIGITAL")]
    TvTerrestrialDigital = ffi::AKEYCODE_TV_TERRESTRIAL_DIGITAL as i32,
    #[doc(alias = "AKEYCODE_TV_SATELLITE")]
    TvSatellite = ffi::AKEYCODE_TV_SATELLITE as i32,
    #[doc(alias = "AKEYCODE_TV_SATELLITE_BS")]
    TvSatelliteBs = ffi::AKEYCODE_TV_SATELLITE_BS as i32,
    #[doc(alias = "AKEYCODE_TV_SATELLITE_CS")]
    TvSatelliteCs = ffi::AKEYCODE_TV_SATELLITE_CS as i32,
    #[doc(alias = "AKEYCODE_TV_SATELLITE_SERVICE")]
    TvSatelliteService = ffi::AKEYCODE_TV_SATELLITE_SERVICE as i32,
    #[doc(alias = "AKEYCODE_TV_NETWORK")]
    TvNetwork = ffi::AKEYCODE_TV_NETWORK as i32,
    #[doc(alias = "AKEYCODE_TV_ANTENNA_CABLE")]
    TvAntennaCable = ffi::AKEYCODE_TV_ANTENNA_CABLE as i32,
    #[doc(alias = "AKEYCODE_TV_INPUT_HDMI_1")]
    TvInputHdmi1 = ffi::AKEYCODE_TV_INPUT_HDMI_1 as i32,
    #[doc(alias = "AKEYCODE_TV_INPUT_HDMI_2")]
    TvInputHdmi2 = ffi::AKEYCODE_TV_INPUT_HDMI_2 as i32,
    #[doc(alias = "AKEYCODE_TV_INPUT_HDMI_3")]
    TvInputHdmi3 = ffi::AKEYCODE_TV_INPUT_HDMI_3 as i32,
    #[doc(alias = "AKEYCODE_TV_INPUT_HDMI_4")]
    TvInputHdmi4 = ffi::AKEYCODE_TV_INPUT_HDMI_4 as i32,
    #[doc(alias = "AKEYCODE_TV_INPUT_COMPOSITE_1")]
    TvInputComposite1 = ffi::AKEYCODE_TV_INPUT_COMPOSITE_1 as i32,
    #[doc(alias = "AKEYCODE_TV_INPUT_COMPOSITE_2")]
    TvInputComposite2 = ffi::AKEYCODE_TV_INPUT_COMPOSITE_2 as i32,
    #[doc(alias = "AKEYCODE_TV_INPUT_COMPONENT_1")]
    TvInputComponent1 = ffi::AKEYCODE_TV_INPUT_COMPONENT_1 as i32,
    #[doc(alias = "AKEYCODE_TV_INPUT_COMPONENT_2")]
    TvInputComponent2 = ffi::AKEYCODE_TV_INPUT_COMPONENT_2 as i32,
    #[doc(alias = "AKEYCODE_TV_INPUT_VGA_1")]
    TvInputVga1 = ffi::AKEYCODE_TV_INPUT_VGA_1 as i32,
    #[doc(alias = "AKEYCODE_TV_AUDIO_DESCRIPTION")]
    TvAudioDescription = ffi::AKEYCODE_TV_AUDIO_DESCRIPTION as i32,
    #[doc(alias = "AKEYCODE_TV_AUDIO_DESCRIPTION_MIX_UP")]
    TvAudioDescriptionMixUp = ffi::AKEYCODE_TV_AUDIO_DESCRIPTION_MIX_UP as i32,
    #[doc(alias = "AKEYCODE_TV_AUDIO_DESCRIPTION_MIX_DOWN")]
    TvAudioDescriptionMixDown = ffi::AKEYCODE_TV_AUDIO_DESCRIPTION_MIX_DOWN as i32,
    #[doc(alias = "AKEYCODE_TV_ZOOM_MODE")]
    TvZoomMode = ffi::AKEYCODE_TV_ZOOM_MODE as i32,
    #[doc(alias = "AKEYCODE_TV_CONTENTS_MENU")]
    TvContentsMenu = ffi::AKEYCODE_TV_CONTENTS_MENU as i32,
    #[doc(alias = "AKEYCODE_TV_MEDIA_CONTEXT_MENU")]
    TvMediaContextMenu = ffi::AKEYCODE_TV_MEDIA_CONTEXT_MENU as i32,
    #[doc(alias = "AKEYCODE_TV_TIMER_PROGRAMMING")]
    TvTimerProgramming = ffi::AKEYCODE_TV_TIMER_PROGRAMMING as i32,
    #[doc(alias = "AKEYCODE_HELP")]
    Help = ffi::AKEYCODE_HELP as i32,
    #[doc(alias = "AKEYCODE_NAVIGATE_PREVIOUS")]
    NavigatePrevious = ffi::AKEYCODE_NAVIGATE_PREVIOUS as i32,
    #[doc(alias = "AKEYCODE_NAVIGATE_NEXT")]
    NavigateNext = ffi::AKEYCODE_NAVIGATE_NEXT as i32,
    #[doc(alias = "AKEYCODE_NAVIGATE_IN")]
    NavigateIn = ffi::AKEYCODE_NAVIGATE_IN as i32,
    #[doc(alias = "AKEYCODE_NAVIGATE_OUT")]
    NavigateOut = ffi::AKEYCODE_NAVIGATE_OUT as i32,
    #[doc(alias = "AKEYCODE_STEM_PRIMARY")]
    StemPrimary = ffi::AKEYCODE_STEM_PRIMARY as i32,
    #[doc(alias = "AKEYCODE_STEM_1")]
    Stem1 = ffi::AKEYCODE_STEM_1 as i32,
    #[doc(alias = "AKEYCODE_STEM_2")]
    Stem2 = ffi::AKEYCODE_STEM_2 as i32,
    #[doc(alias = "AKEYCODE_STEM_3")]
    Stem3 = ffi::AKEYCODE_STEM_3 as i32,
    #[doc(alias = "AKEYCODE_DPAD_UP_LEFT")]
    DpadUpLeft = ffi::AKEYCODE_DPAD_UP_LEFT as i32,
    #[doc(alias = "AKEYCODE_DPAD_DOWN_LEFT")]
    DpadDownLeft = ffi::AKEYCODE_DPAD_DOWN_LEFT as i32,
    #[doc(alias = "AKEYCODE_DPAD_UP_RIGHT")]
    DpadUpRight = ffi::AKEYCODE_DPAD_UP_RIGHT as i32,
    #[doc(alias = "AKEYCODE_DPAD_DOWN_RIGHT")]
    DpadDownRight = ffi::AKEYCODE_DPAD_DOWN_RIGHT as i32,
    #[doc(alias = "AKEYCODE_MEDIA_SKIP_FORWARD")]
    MediaSkipForward = ffi::AKEYCODE_MEDIA_SKIP_FORWARD as i32,
    #[doc(alias = "AKEYCODE_MEDIA_SKIP_BACKWARD")]
    MediaSkipBackward = ffi::AKEYCODE_MEDIA_SKIP_BACKWARD as i32,
    #[doc(alias = "AKEYCODE_MEDIA_STEP_FORWARD")]
    MediaStepForward = ffi::AKEYCODE_MEDIA_STEP_FORWARD as i32,
    #[doc(alias = "AKEYCODE_MEDIA_STEP_BACKWARD")]
    MediaStepBackward = ffi::AKEYCODE_MEDIA_STEP_BACKWARD as i32,
    #[doc(alias = "AKEYCODE_SOFT_SLEEP")]
    SoftSleep = ffi::AKEYCODE_SOFT_SLEEP as i32,
    #[doc(alias = "AKEYCODE_CUT")]
    Cut = ffi::AKEYCODE_CUT as i32,
    #[doc(alias = "AKEYCODE_COPY")]
    Copy = ffi::AKEYCODE_COPY as i32,
    #[doc(alias = "AKEYCODE_PASTE")]
    Paste = ffi::AKEYCODE_PASTE as i32,
    #[doc(alias = "AKEYCODE_SYSTEM_NAVIGATION_UP")]
    SystemNavigationUp = ffi::AKEYCODE_SYSTEM_NAVIGATION_UP as i32,
    #[doc(alias = "AKEYCODE_SYSTEM_NAVIGATION_DOWN")]
    SystemNavigationDown = ffi::AKEYCODE_SYSTEM_NAVIGATION_DOWN as i32,
    #[doc(alias = "AKEYCODE_SYSTEM_NAVIGATION_LEFT")]
    SystemNavigationLeft = ffi::AKEYCODE_SYSTEM_NAVIGATION_LEFT as i32,
    #[doc(alias = "AKEYCODE_SYSTEM_NAVIGATION_RIGHT")]
    SystemNavigationRight = ffi::AKEYCODE_SYSTEM_NAVIGATION_RIGHT as i32,
    #[doc(alias = "AKEYCODE_ALL_APPS")]
    AllApps = ffi::AKEYCODE_ALL_APPS as i32,
    #[doc(alias = "AKEYCODE_REFRESH")]
    Refresh = ffi::AKEYCODE_REFRESH as i32,
    #[doc(alias = "AKEYCODE_THUMBS_UP")]
    ThumbsUp = ffi::AKEYCODE_THUMBS_UP as i32,
    #[doc(alias = "AKEYCODE_THUMBS_DOWN")]
    ThumbsDown = ffi::AKEYCODE_THUMBS_DOWN as i32,
    #[doc(alias = "AKEYCODE_PROFILE_SWITCH")]
    ProfileSwitch = ffi::AKEYCODE_PROFILE_SWITCH as i32,
    // TODO: Still missing a bunch here
    /// Open Emoji picker
    EmojiPicker = ffi::AKEYCODE_EMOJI_PICKER as i32,
    /// Take Screenshot
    Screenshot = ffi::AKEYCODE_SCREENSHOT as i32,

    #[doc(hidden)]
    #[num_enum(catch_all)]
    __Unknown(i32),
}

impl KeyEvent {
    /// Constructs a KeyEvent from a pointer to a native [`ffi::AInputEvent`]
    ///
    /// # Safety
    /// By calling this method, you assert that the pointer is a valid, non-null pointer to an
    /// [`ffi::AInputEvent`], and that [`ffi::AInputEvent`] is an `AKeyEvent`.
    #[inline]
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AInputEvent>) -> Self {
        Self { ptr }
    }

    /// Creates a native [`InputEvent`] object that is a copy of the specified Java
    /// [`android.view.KeyEvent`]. The result may be used with generic and [`KeyEvent`]-specific
    /// functions.
    ///
    /// # Safety
    ///
    /// This function should be called with a healthy JVM pointer and with a non-null
    /// [`android.view.KeyEvent`].
    ///
    /// [`android.view.KeyEvent`]: https://developer.android.com/reference/android/view/KeyEvent
    #[cfg(feature = "api-level-31")]
    #[doc(alias = "AKeyEvent_fromJava")]
    pub unsafe fn from_java(env: *mut JNIEnv, key_event: jobject) -> Option<InputEventJava> {
        let ptr = unsafe { ffi::AKeyEvent_fromJava(env, key_event) };
        Some(InputEventJava(InputEvent::KeyEvent(Self::from_ptr(
            NonNull::new(ptr.cast_mut())?,
        ))))
    }

    /// Returns a pointer to the native [`ffi::AInputEvent`].
    #[inline]
    pub fn ptr(&self) -> NonNull<ffi::AInputEvent> {
        self.ptr
    }

    /// Returns the key action represented by this event.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#akeyevent_getaction)
    #[inline]
    #[doc(alias = "AKeyEvent_getAction")]
    pub fn action(&self) -> KeyAction {
        let action = unsafe { ffi::AKeyEvent_getAction(self.ptr.as_ptr()) };
        action.into()
    }

    /// Get the source of the event.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#ainputevent_getsource)
    #[inline]
    #[doc(alias = "AInputEvent_getSource")]
    pub fn source(&self) -> Source {
        let source = unsafe { ffi::AInputEvent_getSource(self.ptr.as_ptr()) };
        source.into()
    }

    /// Get the device id associated with the event.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#ainputevent_getdeviceid)
    #[inline]
    #[doc(alias = "AInputEvent_getDeviceId")]
    pub fn device_id(&self) -> i32 {
        unsafe { ffi::AInputEvent_getDeviceId(self.ptr.as_ptr()) }
    }

    /// Returns the last time the key was pressed.  This is on the scale of
    /// `java.lang.System.nanoTime()`, which has nanosecond precision, but no defined start time.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#akeyevent_getdowntime)
    #[inline]
    #[doc(alias = "AKeyEvent_getDownTime")]
    pub fn down_time(&self) -> i64 {
        unsafe { ffi::AKeyEvent_getDownTime(self.ptr.as_ptr()) }
    }

    /// Returns the time this event occured.  This is on the scale of
    /// `java.lang.System.nanoTime()`, which has nanosecond precision, but no defined start time.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#akeyevent_geteventtime)
    #[inline]
    #[doc(alias = "AKeyEvent_getEventTime")]
    pub fn event_time(&self) -> i64 {
        unsafe { ffi::AKeyEvent_getEventTime(self.ptr.as_ptr()) }
    }

    /// Returns the keycode associated with this key event
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#akeyevent_getkeycode)
    #[inline]
    #[doc(alias = "AKeyEvent_getKeyCode")]
    pub fn key_code(&self) -> Keycode {
        let keycode = unsafe { ffi::AKeyEvent_getKeyCode(self.ptr.as_ptr()) };
        keycode.into()
    }

    /// Returns the number of repeats of a key.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#akeyevent_getrepeatcount)
    #[inline]
    #[doc(alias = "AKeyEvent_getRepeatCount")]
    pub fn repeat_count(&self) -> i32 {
        unsafe { ffi::AKeyEvent_getRepeatCount(self.ptr.as_ptr()) }
    }

    /// Returns the hardware keycode of a key.  This varies from device to device.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#akeyevent_getscancode)
    #[inline]
    #[doc(alias = "AKeyEvent_getScanCode")]
    pub fn scan_code(&self) -> i32 {
        unsafe { ffi::AKeyEvent_getScanCode(self.ptr.as_ptr()) }
    }

    /// Flags associated with this [`KeyEvent`].
    ///
    /// See [the NDK docs](https://developer.android.com/ndk/reference/group/input#akeyevent_getflags)
    #[inline]
    #[doc(alias = "AKeyEvent_getFlags")]
    pub fn flags(&self) -> KeyEventFlags {
        unsafe { KeyEventFlags(ffi::AKeyEvent_getFlags(self.ptr.as_ptr()) as u32) }
    }

    /// Returns the state of the modifiers during this key event, represented by a bitmask.
    ///
    /// See [the NDK
    /// docs](https://developer.android.com/ndk/reference/group/input#akeyevent_getmetastate)
    #[inline]
    #[doc(alias = "AKeyEvent_getMetaState")]
    pub fn meta_state(&self) -> MetaState {
        unsafe { MetaState(ffi::AKeyEvent_getMetaState(self.ptr.as_ptr()) as u32) }
    }
}

/// Flags associated with [`KeyEvent`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct KeyEventFlags(pub u32);

impl KeyEventFlags {
    #[inline]
    #[doc(alias = "AKEY_EVENT_FLAG_WOKE_HERE")]
    pub fn woke_here(self) -> bool {
        self.0 & ffi::AKEY_EVENT_FLAG_WOKE_HERE != 0
    }
    #[inline]
    #[doc(alias = "AKEY_EVENT_FLAG_SOFT_KEYBOARD")]
    pub fn soft_keyboard(self) -> bool {
        self.0 & ffi::AKEY_EVENT_FLAG_SOFT_KEYBOARD != 0
    }
    #[inline]
    #[doc(alias = "AKEY_EVENT_FLAG_KEEP_TOUCH_MODE")]
    pub fn keep_touch_mode(self) -> bool {
        self.0 & ffi::AKEY_EVENT_FLAG_KEEP_TOUCH_MODE != 0
    }
    #[inline]
    #[doc(alias = "AKEY_EVENT_FLAG_FROM_SYSTEM")]
    pub fn from_system(self) -> bool {
        self.0 & ffi::AKEY_EVENT_FLAG_FROM_SYSTEM != 0
    }
    #[inline]
    #[doc(alias = "AKEY_EVENT_FLAG_EDITOR_ACTION")]
    pub fn editor_action(self) -> bool {
        self.0 & ffi::AKEY_EVENT_FLAG_EDITOR_ACTION != 0
    }
    #[inline]
    #[doc(alias = "AKEY_EVENT_FLAG_CANCELED")]
    pub fn canceled(self) -> bool {
        self.0 & ffi::AKEY_EVENT_FLAG_CANCELED != 0
    }
    #[inline]
    #[doc(alias = "AKEY_EVENT_FLAG_VIRTUAL_HARD_KEY")]
    pub fn virtual_hard_key(self) -> bool {
        self.0 & ffi::AKEY_EVENT_FLAG_VIRTUAL_HARD_KEY != 0
    }
    #[inline]
    #[doc(alias = "AKEY_EVENT_FLAG_LONG_PRESS")]
    pub fn long_press(self) -> bool {
        self.0 & ffi::AKEY_EVENT_FLAG_LONG_PRESS != 0
    }
    #[inline]
    #[doc(alias = "AKEY_EVENT_FLAG_CANCELED_LONG_PRESS")]
    pub fn canceled_long_press(self) -> bool {
        self.0 & ffi::AKEY_EVENT_FLAG_CANCELED_LONG_PRESS != 0
    }
    #[inline]
    #[doc(alias = "AKEY_EVENT_FLAG_TRACKING")]
    pub fn tracking(self) -> bool {
        self.0 & ffi::AKEY_EVENT_FLAG_TRACKING != 0
    }
    #[inline]
    #[doc(alias = "AKEY_EVENT_FLAG_FALLBACK")]
    pub fn fallback(self) -> bool {
        self.0 & ffi::AKEY_EVENT_FLAG_FALLBACK != 0
    }
}
