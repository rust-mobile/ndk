//! Safe bindings for [`AMidiDevice`], [`AMidiInputPort`], and [`AMidiOutputPort`]
//!
//! Provides implementation of `SafeMidiDeviceBox` that ensures the current thread is attached to
//! the Java VM when being dropped, which is required by [`AMidiDevice_release`]. All other types
//! of this module holds an [`Arc`] of `SafeMidiDeviceBox`.
//!
//! [`AMidiDevice`]: https://developer.android.com/ndk/reference/group/midi#amididevice
//! [`AMidiDevice_release`]: https://developer.android.com/ndk/reference/group/midi#amididevice_release
//! [`AMidiInputPort`]: https://developer.android.com/ndk/reference/group/midi#amidiinputport
//! [`AMidiOutputPort`]: https://developer.android.com/ndk/reference/group/midi#amidioutputport

use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::Deref;
use std::ptr::{self, NonNull};
use std::sync::Arc;
use std::{fmt, mem};

use crate::media_error::Result;

use super::*;

/// The owner of [`MidiDevice`]. [`SafeMidiDevice`], [`SafeMidiInputPort`], and
/// [`SafeMidiOutputPort`] holds an [`Arc<SafeMidiDeviceBox>`]. to ensure that the underlying
/// [`MidiDevice`] is not dropped while the safe wrappers are alive.
///
/// [`SafeMidiDeviceBox`] also holds a pointer to the current Java VM to attach the calling thread
/// of [`Drop::drop`] to the VM, which is required by [`ffi::AMidiDevice_release`].
struct SafeMidiDeviceBox {
    midi_device: ManuallyDrop<MidiDevice>,
    java_vm: NonNull<jni_sys::JavaVM>,
}

// SAFETY: [`SafeMidiDeviceBox::drop`] attaches the calling thread to the Java VM if required.
unsafe impl Send for SafeMidiDeviceBox {}

// SAFETY: [`SafeMidiDeviceBox::drop`] attaches the calling thread to the Java VM if required.
unsafe impl Sync for SafeMidiDeviceBox {}

impl Drop for SafeMidiDeviceBox {
    fn drop(&mut self) {
        unsafe {
            let java_vm_functions = self.java_vm.as_mut().as_ref().unwrap_unchecked();
            let java_vm = self.java_vm.as_ptr();
            let mut current_thread_was_attached = true;
            let mut env = MaybeUninit::uninit();

            // Try to get the current thread's JNIEnv
            if (java_vm_functions.GetEnv.unwrap_unchecked())(
                java_vm,
                env.as_mut_ptr(),
                jni_sys::JNI_VERSION_1_6,
            ) != jni_sys::JNI_OK
            {
                // Current thread is not attached to the Java VM. Try to attach.
                current_thread_was_attached = false;
                if (java_vm_functions
                    .AttachCurrentThreadAsDaemon
                    .unwrap_unchecked())(
                    java_vm, env.as_mut_ptr(), ptr::null_mut()
                ) != jni_sys::JNI_OK
                {
                    panic!("failed to attach the current thread to the Java VM");
                }
            }

            // Dropping MidiDevice requires the current thread to be attached to the Java VM.
            ManuallyDrop::drop(&mut self.midi_device);

            // Releasing MidiDevice is complete; if the current thread was not attached to the VM,
            // detach the current thread.
            if !current_thread_was_attached {
                (java_vm_functions.DetachCurrentThread.unwrap_unchecked())(java_vm);
            }
        }
    }
}

/// A thread-safe wrapper over [`MidiDevice`].
pub struct SafeMidiDevice {
    inner: Arc<SafeMidiDeviceBox>,
}

impl SafeMidiDevice {
    /// Assumes ownership of `ptr`
    ///
    /// # Safety
    /// `env` and `ptr` must be valid pointers to a [`jni_sys::JNIEnv`] instance and an Android
    /// [`ffi::AMidiDevice`].
    pub unsafe fn from_ptr(env: *mut jni_sys::JNIEnv, ptr: NonNull<ffi::AMidiDevice>) -> Self {
        Self::from_unsafe(env, MidiDevice::from_ptr(ptr))
    }

    /// Connects a native Midi Device object to the associated Java MidiDevice object.
    ///
    /// Use the returned [`SafeMidiDevice`] to access the rest of the native MIDI API.
    ///
    /// # Safety
    /// `env` and `midi_device_obj` must be valid pointers to a [`jni_sys::JNIEnv`] instance and a
    /// Java [`MidiDevice`](https://developer.android.com/reference/android/media/midi/MidiDevice)
    /// instance.
    pub unsafe fn from_java(
        env: *mut jni_sys::JNIEnv,
        midi_device_obj: jni_sys::jobject,
    ) -> Result<Self> {
        Ok(Self::from_unsafe(
            env,
            MidiDevice::from_java(env, midi_device_obj)?,
        ))
    }

    /// Wraps the given unsafe [`MidiDevice`] instance into a safe counterpart.
    ///
    /// # Safety
    ///
    /// `env` must be a valid pointer to a [`jni_sys::JNIEnv`] instance.
    pub unsafe fn from_unsafe(env: *mut jni_sys::JNIEnv, midi_device: MidiDevice) -> Self {
        let env_functions = env.as_mut().unwrap().as_ref().unwrap_unchecked();
        let mut java_vm = MaybeUninit::uninit();
        if (env_functions.GetJavaVM.unwrap_unchecked())(env, java_vm.as_mut_ptr())
            != jni_sys::JNI_OK
        {
            panic!("failed to get the current Java VM");
        }
        let java_vm = NonNull::new_unchecked(java_vm.assume_init());

        SafeMidiDevice {
            inner: Arc::new(SafeMidiDeviceBox {
                midi_device: ManuallyDrop::new(midi_device),
                java_vm,
            }),
        }
    }

    /// Opens the input port so that the client can send data to it.
    pub fn open_safe_input_port(&self, port_number: i32) -> Result<SafeMidiInputPort> {
        Ok(SafeMidiInputPort {
            // Convert the returned MidiInputPort<'_> into a MidiInputPort<'static>.
            //
            // SAFETY: the associated MIDI device of the input port will be alive during the
            // lifetime of the returned SafeMidiInputPort because it is hold by _device.
            // Since Rust calls the destructor of fields in declaration order, the MIDI device
            // will be alive even when the input port is being dropped.
            inner: unsafe { mem::transmute(self.open_input_port(port_number)?) },
            _device: Arc::clone(&self.inner),
        })
    }

    /// Opens the output port so that the client can receive data from it.
    pub fn open_safe_output_port(&self, port_number: i32) -> Result<SafeMidiOutputPort> {
        Ok(SafeMidiOutputPort {
            // Convert the returned MidiInputPort<'_> into a MidiInputPort<'static>.
            //
            // SAFETY: the associated MIDI device of the output port will be alive during the
            // lifetime of the returned SafeMidiOutputPort because it is hold by _device.
            // Since Rust calls the destructor of fields in declaration order, the MIDI device
            // will be alive even when the output port is being dropped.
            inner: unsafe { mem::transmute(self.open_output_port(port_number)?) },
            _device: Arc::clone(&self.inner),
        })
    }
}

impl Deref for SafeMidiDevice {
    type Target = MidiDevice;

    fn deref(&self) -> &Self::Target {
        &self.inner.midi_device
    }
}

impl fmt::Debug for SafeMidiDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SafeMidiDevice")
            .field("ptr", &self.inner.midi_device.ptr)
            .field("java_vm", &self.inner.java_vm)
            .finish()
    }
}

pub struct SafeMidiInputPort {
    inner: MidiInputPort<'static>,
    _device: Arc<SafeMidiDeviceBox>,
}

impl fmt::Debug for SafeMidiInputPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SafeMidiInputPort")
            .field("inner", &self.inner.ptr)
            .finish()
    }
}

impl Deref for SafeMidiInputPort {
    type Target = MidiInputPort<'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// SAFETY: a AMidi port is a mere holder of an atomic state, a pointer to the associated MIDI
// device, a binder, and a file descriptor, all of which are safe to be sent to another thread.
// https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/media/native/midi/amidi.cpp?q=symbol%3A%5CbAMIDI_Port%5Cb%20case%3Ayes
unsafe impl Send for SafeMidiInputPort {}

// SAFETY: AMidiInputPort contains a file descriptor to a socket opened with the SOCK_SEQPACKET
// mode, which preserves message boundaries so the receiver of a message always reads the whole
// message.
// https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/media/native/midi/amidi.cpp?q=symbol%3A%5CbAMIDI_PACKET_SIZE%5Cb%20case%3Ayes
unsafe impl Sync for SafeMidiInputPort {}

pub struct SafeMidiOutputPort {
    inner: MidiOutputPort<'static>,
    _device: Arc<SafeMidiDeviceBox>,
}

impl fmt::Debug for SafeMidiOutputPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SafeMidiOutputPort")
            .field("inner", &self.inner.ptr)
            .finish()
    }
}

impl Deref for SafeMidiOutputPort {
    type Target = MidiOutputPort<'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// SAFETY: a AMidi port is a mere holder of an atomic state, a pointer to the associated MIDI
// device, a binder, and a file descriptor, all of which are safe to be sent to another thread.
// https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/media/native/midi/amidi.cpp?q=symbol%3A%5CbAMIDI_Port%5Cb%20case%3Ayes
unsafe impl Send for SafeMidiOutputPort {}

// SAFETY: AMidiOutputPort is guarded by a atomic state ([`AMIDI_Port::state`]), which enables
// [`ffi::AMidiOutputPort_receive`] to detect accesses from multiple threads and return error.
// https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/media/native/midi/amidi.cpp?q=symbol%3A%5CbMidiReceiver%3A%3Areceive%5Cb%20case%3Ayes
unsafe impl Sync for SafeMidiOutputPort {}
