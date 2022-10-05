//! Bindings for [`AMidiDevice`], [`AMidiInputPort`], and [`AMidiOutputPort`]
//!
//! See [the NDK guide](https://developer.android.com/ndk/guides/audio/midi) for
//! design and usage instructions, and [the NDK reference](https://developer.android.com/ndk/reference/group/midi)
//! for an API overview.
//!
//! [`AMidiDevice`]: https://developer.android.com/ndk/reference/group/midi#amididevice
//! [`AMidiInputPort`]: https://developer.android.com/ndk/reference/group/midi#amidiinputport
//! [`AMidiOutputPort`]: https://developer.android.com/ndk/reference/group/midi#amidioutputport
#![cfg(feature = "midi")]

pub use super::media::Result;
use super::media::{construct, NdkMediaError};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt;
use std::marker::PhantomData;
use std::os::raw::{c_int, c_uint};
use std::ptr::NonNull;

// There is no mention about thread-safety in the NDK reference, but the official Android C++ MIDI
// sample stores `AMidiDevice *` and `AMidi{Input,Output}Port *` in global variables and accesses the
// ports from separate threads.
// See https://github.com/android/ndk-samples/blob/7f6936ea044ee29c36b5c3ebd62bb3a64e1e6014/native-midi/app/src/main/cpp/AppMidiManager.cpp
unsafe impl Send for MidiDevice {}
unsafe impl<'a> Send for MidiInputPort<'a> {}
unsafe impl<'a> Send for MidiOutputPort<'a> {}

/// Result of [`MidiOutputPort::receive`].
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum MidiOpcode {
    /// No MIDI messages are available.
    NoMessage,
    /// Received a MIDI message with the given length.
    Data(usize),
    /// Instructed to discard all pending MIDI data.
    Flush,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum MidiDeviceType {
    Bluetooth = ffi::AMIDI_DEVICE_TYPE_BLUETOOTH,
    USB = ffi::AMIDI_DEVICE_TYPE_USB,
    Virtual = ffi::AMIDI_DEVICE_TYPE_VIRTUAL,
}

#[derive(Debug)]
pub struct MidiDevice {
    inner: NonNull<ffi::AMidiDevice>,
}

impl MidiDevice {
    /// Creates an `MidiDevice` from a pointer.
    ///
    /// # Safety
    /// By calling this function, you assert that the pointer is a valid pointer to a native
    /// `AMidiDevice`.
    pub unsafe fn from_ptr(inner: NonNull<ffi::AMidiDevice>) -> Self {
        Self { inner }
    }

    fn as_ptr(&self) -> *mut ffi::AMidiDevice {
        self.inner.as_ptr()
    }

    /// Connects a native Midi Device object to the associated Java MidiDevice object.
    ///
    /// Use the returned [`MidiDevice`] to access the rest of the native MIDI API.
    pub fn from_java(
        env: *mut jni_sys::JNIEnv,
        midi_device_obj: jni_sys::jobject,
    ) -> Result<MidiDevice> {
        unsafe {
            let ptr = construct(|res| ffi::AMidiDevice_fromJava(env, midi_device_obj, res))?;
            Ok(Self::from_ptr(NonNull::new_unchecked(ptr)))
        }
    }

    /// Gets the number of input (sending) ports available on the specified MIDI device.
    pub fn num_input_ports(&self) -> Result<usize> {
        let num_input_ports = unsafe { ffi::AMidiDevice_getNumInputPorts(self.as_ptr()) };
        if num_input_ports >= 0 {
            Ok(num_input_ports as usize)
        } else {
            NdkMediaError::from_status(ffi::media_status_t(num_input_ports as c_int)).map(|_| 0)
        }
    }

    /// Gets the number of output (receiving) ports available on the specified MIDI device.
    pub fn num_output_ports(&self) -> Result<usize> {
        let num_output_ports = unsafe { ffi::AMidiDevice_getNumOutputPorts(self.as_ptr()) };
        if num_output_ports >= 0 {
            Ok(num_output_ports as usize)
        } else {
            Err(
                NdkMediaError::from_status(ffi::media_status_t(num_output_ports as c_int))
                    .unwrap_err(),
            )
        }
    }

    /// Gets the MIDI device type.
    pub fn device_type(&self) -> Result<MidiDeviceType> {
        let device_type = unsafe { ffi::AMidiDevice_getType(self.as_ptr()) };
        if device_type >= 0 {
            let device_type = MidiDeviceType::try_from(device_type as u32).map_err(|e| {
                NdkMediaError::UnknownResult(ffi::media_status_t(e.number as c_int))
            })?;
            Ok(device_type)
        } else {
            Err(NdkMediaError::from_status(ffi::media_status_t(device_type)).unwrap_err())
        }
    }

    /// Opens the input port so that the client can send data to it.
    pub fn open_input_port(&self, port_number: i32) -> Result<MidiInputPort> {
        unsafe {
            let input_port =
                construct(|res| ffi::AMidiInputPort_open(self.as_ptr(), port_number, res))?;
            Ok(MidiInputPort::from_ptr(NonNull::new_unchecked(input_port)))
        }
    }

    /// Opens the output port so that the client can receive data from it.
    pub fn open_output_port(&self, port_number: i32) -> Result<MidiOutputPort> {
        unsafe {
            let output_port =
                construct(|res| ffi::AMidiOutputPort_open(self.as_ptr(), port_number, res))?;
            Ok(MidiOutputPort::from_ptr(NonNull::new_unchecked(
                output_port,
            )))
        }
    }
}

impl Drop for MidiDevice {
    fn drop(&mut self) {
        let status = unsafe { ffi::AMidiDevice_release(self.as_ptr()) };
        NdkMediaError::from_status(status).unwrap();
    }
}

pub struct MidiInputPort<'a> {
    inner: NonNull<ffi::AMidiInputPort>,
    _marker: PhantomData<&'a MidiDevice>,
}

impl<'a> fmt::Debug for MidiInputPort<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MidiInputPort")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'a> MidiInputPort<'a> {
    /// Creates an `MidiInputPort` from a pointer.
    ///
    /// # Safety
    /// By calling this function, you assert that the pointer is a valid pointer to a native
    /// `AMidiInputPort`.
    pub unsafe fn from_ptr(inner: NonNull<ffi::AMidiInputPort>) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    fn as_ptr(&self) -> *mut ffi::AMidiInputPort {
        self.inner.as_ptr()
    }

    /// Sends data to the specified input port.
    pub fn send(&self, buffer: &[u8]) -> Result<usize> {
        let num_bytes_sent = unsafe {
            ffi::AMidiInputPort_send(self.as_ptr(), buffer.as_ptr(), buffer.len() as ffi::size_t)
        };
        if num_bytes_sent >= 0 {
            Ok(num_bytes_sent as usize)
        } else {
            Err(
                NdkMediaError::from_status(ffi::media_status_t(num_bytes_sent as c_int))
                    .unwrap_err(),
            )
        }
    }

    /// Sends a message with a 'MIDI flush command code' to the specified port.
    ///
    /// This should cause a receiver to discard any pending MIDI data it may have accumulated and
    /// not processed.
    pub fn send_flush(&self) -> Result<()> {
        let result = unsafe { ffi::AMidiInputPort_sendFlush(self.as_ptr()) };
        NdkMediaError::from_status(result)
    }

    pub fn send_with_timestamp(&self, buffer: &[u8], timestamp: i64) -> Result<usize> {
        let num_bytes_sent = unsafe {
            ffi::AMidiInputPort_sendWithTimestamp(
                self.as_ptr(),
                buffer.as_ptr(),
                buffer.len() as ffi::size_t,
                timestamp,
            )
        };
        if num_bytes_sent >= 0 {
            Ok(num_bytes_sent as usize)
        } else {
            Err(
                NdkMediaError::from_status(ffi::media_status_t(num_bytes_sent as c_int))
                    .unwrap_err(),
            )
        }
    }
}

impl<'a> Drop for MidiInputPort<'a> {
    fn drop(&mut self) {
        unsafe { ffi::AMidiInputPort_close(self.as_ptr()) };
    }
}

pub struct MidiOutputPort<'a> {
    inner: NonNull<ffi::AMidiOutputPort>,
    _marker: PhantomData<&'a MidiDevice>,
}

impl<'a> fmt::Debug for MidiOutputPort<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MidiOutputPort")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'a> MidiOutputPort<'a> {
    /// Creates an `MidiOutputPort` from a pointer.
    ///
    /// # Safety
    /// By calling this function, you assert that the pointer is a valid pointer to a native
    /// `AMidiOutputPort`.
    pub unsafe fn from_ptr(inner: NonNull<ffi::AMidiOutputPort>) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    fn as_ptr(&self) -> *mut ffi::AMidiOutputPort {
        self.inner.as_ptr()
    }

    /// Receives the next pending MIDI message.
    ///
    /// To retrieve all pending messages, the client should repeatedly call this method until it
    /// returns [`Ok(MidiOpcode::NoMessage)`].
    ///
    /// Note that this is a non-blocking call. If there are no Midi messages are available, the
    /// function returns [`Ok(MidiOpcode::NoMessage)`] immediately (for 0 messages received).
    pub fn receive(&self, buffer: &mut [u8]) -> Result<(MidiOpcode, i64)> {
        let mut opcode = 0i32;
        let mut timestamp = 0i64;
        let mut num_bytes_received: ffi::size_t = 0;
        let result = unsafe {
            ffi::AMidiOutputPort_receive(
                self.as_ptr(),
                &mut opcode,
                buffer.as_mut_ptr(),
                buffer.len() as ffi::size_t,
                &mut num_bytes_received,
                &mut timestamp,
            )
        };

        if result < 0 {
            Err(NdkMediaError::from_status(ffi::media_status_t(result as c_int)).unwrap_err())
        } else if result == 0 {
            Ok((MidiOpcode::NoMessage, timestamp))
        } else if opcode as c_uint == ffi::AMIDI_OPCODE_DATA {
            Ok((MidiOpcode::Data(num_bytes_received as usize), timestamp))
        } else {
            Ok((MidiOpcode::Flush, timestamp))
        }
    }
}

impl<'a> Drop for MidiOutputPort<'a> {
    fn drop(&mut self) {
        unsafe { ffi::AMidiOutputPort_close(self.as_ptr()) };
    }
}
