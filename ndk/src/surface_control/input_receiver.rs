#![cfg(feature = "api-level-35")]
//! Bindings for [`AInputReceiver`]

use std::{ffi::c_void, ptr::NonNull};

use jni_sys::{jobject, JNIEnv};

use crate::{
    choreographer::Choreographer,
    event::{KeyEvent, KeyEventJava, MotionEvent, MotionEventJava},
    looper::ForeignLooper,
    utils::abort_on_panic,
};

use super::SurfaceControl;

/// This callback is invoked when the registered input channel receives a motion event.
///
/// TODO: Document return
#[doc(alias = "AInputReceiver_onMotionEvent")]
type OnMotionEvent = Box<dyn FnMut(MotionEventJava) -> bool>;
/// This callback is invoked when the registered input channel receives a key event.
///
/// TODO: Document return
#[doc(alias = "AInputReceiver_onKeyEvent")]
type OnKeyEvent = Box<dyn FnMut(KeyEventJava) -> bool>;

/// The [`InputReceiver`] that holds the reference to the registered input channel.
#[derive(Debug)]
#[doc(alias = "AInputReceiver")]
pub struct InputReceiver {
    ptr: NonNull<ffi::AInputReceiver>,
}

impl Drop for InputReceiver {
    // TODO: Drop thread comment and enforce by "lack of" Send/Sync (unless this is not true for the other functions?)
    /// Unregisters the input channel and deletes the [`InputReceiver`]. This must be called on the
    /// same looper thread it was created with.
    #[doc(alias = "AInputReceiver_release")]
    fn drop(&mut self) {
        unsafe { ffi::AInputReceiver_release(self.ptr.as_ptr()) }
    }
}

impl InputReceiver {
    /// Registers an input receiver for an ASurfaceControl that will receive batched input event. For
    /// those events that are batched, the invocation will happen once per AChoreographer frame, and
    /// other input events will be delivered immediately.
    ///
    /// This is different from AInputReceiver_createUnbatchedInputReceiver in that the input events are
    /// received batched. The caller must invoke AInputReceiver_release to clean up the resources when
    /// no longer needing to use the input receiver.
    ///
    /// \param aChoreographer         The AChoreographer used for batching. This should match the
    ///                               rendering AChoreographer.
    /// \param hostInputTransferToken The host token to link the embedded. This is used to handle
    ///                               transferring touch gesture from host to embedded and for ANRs
    ///                               to ensure the host receives the ANR if any issues with
    ///                               touch on the embedded. This can be retrieved for the host window
    ///                               by calling AttachedSurfaceControl#getInputTransferToken()
    /// \param aSurfaceControl        The ASurfaceControl to register the InputChannel for
    /// \param aInputReceiverCallbacks The SurfaceControlInputReceiver that will receive the input events
    ///
    /// Returns the reference to AInputReceiver to clean up resources when done.
    #[doc(alias = "AInputReceiver_createBatchedInputReceiver")]
    pub fn create_batched_input_receiver(
        choreographer: &Choreographer,
        host_input_transfer_token: &InputTransferToken,
        surface_control: &SurfaceControl,
        input_receiver_callbacks: &InputReceiverCallbacks,
    ) -> Self {
        let ptr = NonNull::new(unsafe {
            ffi::AInputReceiver_createBatchedInputReceiver(
                choreographer.ptr().as_ptr(),
                host_input_transfer_token.ptr.as_ptr(),
                surface_control.ptr().as_ptr(),
                input_receiver_callbacks.ptr.as_ptr(),
            )
        })
        .unwrap();

        Self { ptr }
    }

    /// Registers an input receiver for an ASurfaceControl that will receive every input event.
    /// This is different from AInputReceiver_createBatchedInputReceiver in that the input events are
    /// received unbatched. The caller must invoke AInputReceiver_release to clean up the resources when
    /// no longer needing to use the input receiver.
    ///
    /// \param aLooper                The looper to use when invoking callbacks.
    /// \param hostInputTransferToken The host token to link the embedded. This is used to handle
    ///                               transferring touch gesture from host to embedded and for ANRs
    ///                               to ensure the host receives the ANR if any issues with
    ///                               touch on the embedded. This can be retrieved for the host window
    ///                               by calling AttachedSurfaceControl#getInputTransferToken()
    /// \param aSurfaceControl        The ASurfaceControl to register the InputChannel for
    /// \param aInputReceiverCallbacks The SurfaceControlInputReceiver that will receive the input events
    ///
    /// Returns the reference to AInputReceiver to clean up resources when done.
    #[doc(alias = "AInputReceiver_createUnbatchedInputReceiver")]
    pub fn create_unbatched_input_receiver(
        // TODO: Foreign or local?
        looper: &ForeignLooper,
        host_input_transfer_token: &InputTransferToken,
        surface_control: &SurfaceControl,
        input_receiver_callbacks: &InputReceiverCallbacks,
    ) -> Self {
        let ptr = NonNull::new(unsafe {
            ffi::AInputReceiver_createUnbatchedInputReceiver(
                looper.ptr().as_ptr(),
                host_input_transfer_token.ptr.as_ptr(),
                surface_control.ptr().as_ptr(),
                input_receiver_callbacks.ptr.as_ptr(),
            )
        })
        .unwrap();

        Self { ptr }
    }

    /// Returns the AInputTransferToken that can be used to transfer touch gesture to or from other
    /// windows. This InputTransferToken is associated with the SurfaceControl that registered an input
    /// receiver and can be used with the host token for things like transfer touch gesture via
    /// WindowManager#transferTouchGesture().
    ///
    /// This must be released with AInputTransferToken_release.
    ///
    /// \param aInputReceiver The inputReceiver object to retrieve the AInputTransferToken for.
    #[doc(alias = "AInputReceiver_getInputTransferToken")]
    pub fn input_transfer_token(&self) -> InputTransferToken {
        let ptr = NonNull::new(
            unsafe { ffi::AInputReceiver_getInputTransferToken(self.ptr.as_ptr()) }.cast_mut(),
        )
        .unwrap();

        InputTransferToken { ptr }
    }
}

type Functions = (Option<OnMotionEvent>, Option<OnKeyEvent>);

// #[derive(Debug)]
#[doc(alias = "AInputReceiverCallbacks")]
pub struct InputReceiverCallbacks {
    ptr: NonNull<ffi::AInputReceiverCallbacks>,
    context: Box<Functions>,
}

impl Drop for InputReceiverCallbacks {
    /// Releases the AInputReceiverCallbacks. This must be called on the same
    /// looper thread the AInputReceiver was created with. The receiver will not invoke any callbacks
    /// once it's been released.
    #[doc(alias = "AInputReceiverCallbacks_release")]
    fn drop(&mut self) {
        unsafe { ffi::AInputReceiverCallbacks_release(self.ptr.as_ptr()) }
    }
}

impl InputReceiverCallbacks {
    pub fn ptr(&self) -> NonNull<ffi::AInputReceiverCallbacks> {
        self.ptr
    }

    /// Creates a AInputReceiverCallbacks object that is used when registering for an AInputReceiver.
    /// This must be released using AInputReceiverCallbacks_release
    #[doc(alias = "AInputReceiverCallbacks_create")]
    pub fn new() -> Self {
        let mut context = Box::new((None, None));
        let ptr = NonNull::new(unsafe {
            ffi::AInputReceiverCallbacks_create(<*mut _>::cast(&mut context))
        })
        .unwrap();

        // TODO: As usual, we must track:
        // - The boxes for the user functions that we'll set here
        // - set SendSync correctly, if any of this gets invoked on external threads?

        Self { ptr, context }
    }

    /// Sets a AInputReceiver_onMotionEvent callback for an AInputReceiverCallbacks
    ///
    /// \param callbacks The callback object to set the motion event on.
    /// \param onMotionEvent The motion event that will be invoked
    #[doc(alias = "AInputReceiverCallbacks_setMotionEventCallback")]
    pub fn set_motion_event_callback(&mut self, callback: OnMotionEvent) {
        self.context.0 = Some(callback);

        unsafe extern "C" fn on_motion_event(
            context: *mut c_void,
            motion_event: *mut ffi::AInputEvent,
        ) -> bool {
            abort_on_panic(|| {
                let funcs: *mut Functions = context.cast();
                let motion_event = MotionEvent::java_from_ptr(NonNull::new(motion_event).unwrap());

                ((*funcs).0.as_mut().unwrap())(motion_event)
            })
        }

        unsafe {
            ffi::AInputReceiverCallbacks_setMotionEventCallback(
                self.ptr.as_ptr(),
                Some(on_motion_event),
            )
        }
    }

    /// Sets a AInputReceiver_onKeyEvent callback for an AInputReceiverCallbacks
    ///
    /// \param callbacks The callback object to set the motion event on.
    /// \param onMotionEvent The key event that will be invoked
    #[doc(alias = "AInputReceiverCallbacks_setKeyEventCallback")]
    pub fn set_key_event_callback(&mut self, callback: OnKeyEvent) {
        self.context.1 = Some(callback);

        unsafe extern "C" fn on_key_event(
            context: *mut c_void,
            key_event: *mut ffi::AInputEvent,
        ) -> bool {
            abort_on_panic(|| {
                let funcs: *mut Functions = context.cast();
                let key_event = KeyEvent::java_from_ptr(NonNull::new(key_event).unwrap());

                ((*funcs).1.as_mut().unwrap())(key_event)
            })
        }

        unsafe {
            ffi::AInputReceiverCallbacks_setKeyEventCallback(self.ptr.as_ptr(), Some(on_key_event))
        }
    }
}

/// [`InputTransferToken`] can be used to request focus on or to transfer touch gesture to and from
/// an embedded [`SurfaceControl`].
#[derive(Debug)]
#[doc(alias = "AInputTransferToken")]
pub struct InputTransferToken {
    ptr: NonNull<ffi::AInputTransferToken>,
}

impl InputTransferToken {
    pub fn ptr(&self) -> NonNull<ffi::AInputTransferToken> {
        self.ptr
    }

    /// Return the [`InputTransferToken`] wrapped by a [Java `InputTransferToken`] object.
    ///
    /// # Safety
    ///
    /// This function should be called with a healthy JVM pointer and with a non-null
    /// [`android.window.InputTransferToken`].
    ///
    /// [Java `InputTransferToken`]: https://developer.android.com/reference/android/window/InputTransferToken
    /// [`android.window.InputTransferToken`]: https://developer.android.com/reference/android/window/InputTransferToken
    #[doc(alias = "AInputTransferToken_fromJava")]
    pub unsafe fn from_java(env: *mut JNIEnv, input_transfer_token: jobject) -> Option<Self> {
        let ptr =
            NonNull::new(unsafe { ffi::AInputTransferToken_fromJava(env, input_transfer_token) })?;
        Some(Self { ptr })
    }

    /// Return the [Java `InputTransferToken`] object that wraps [`InputTransferToken`].
    ///
    /// The returned value is an object of instance [`android.window.InputTransferToken`].
    ///
    /// # Safety
    ///
    /// This function should be called with a healthy JVM pointer.
    ///
    /// [Java `InputTransferToken`]: https://developer.android.com/reference/android/window/InputTransferToken
    /// [`android.window.InputTransferToken`]: https://developer.android.com/reference/android/window/InputTransferToken
    #[doc(alias = "AInputTransferToken_toJava")]
    pub unsafe fn to_java(&self, env: *mut JNIEnv) -> jobject {
        unsafe { ffi::AInputTransferToken_toJava(env, self.ptr.as_ptr()) }
    }
}

impl Drop for InputTransferToken {
    /// Removes a reference that was previously acquired in native.
    #[doc(alias = "AInputTransferToken_release")]
    fn drop(&mut self) {
        unsafe { ffi::AInputTransferToken_release(self.ptr.as_ptr()) }
    }
}
