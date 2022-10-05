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

pub use super::media_error::Result;
use super::media_error::{construct, NdkMediaError};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;
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
pub enum MidiOpcode {
    /// No MIDI messages are available.
    NoMessage,
    /// Received a MIDI message with the given length and the timestamp.
    Data { length: usize, timestamp: i64 },
    /// Instructed to discard all pending MIDI data.
    Flush,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum MidiDeviceType {
    Bluetooth = ffi::AMIDI_DEVICE_TYPE_BLUETOOTH,
    USB = ffi::AMIDI_DEVICE_TYPE_USB,
    Virtual = ffi::AMIDI_DEVICE_TYPE_VIRTUAL,
}

#[derive(Debug)]
pub struct MidiDevice {
    ptr: NonNull<ffi::AMidiDevice>,
}

impl MidiDevice {
    /// Assumes ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::AMidiDevice`].
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AMidiDevice>) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> NonNull<ffi::AMidiDevice> {
        self.ptr
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

    /// Gets the number of input (sending) ports available on this device.
    pub fn num_input_ports(&self) -> Result<usize> {
        let num_input_ports = unsafe { ffi::AMidiDevice_getNumInputPorts(self.ptr.as_ptr()) };
        if num_input_ports >= 0 {
            Ok(num_input_ports as usize)
        } else {
            NdkMediaError::from_status(ffi::media_status_t(num_input_ports as c_int)).map(|_| 0)
        }
    }

    /// Gets the number of output (receiving) ports available on this device.
    pub fn num_output_ports(&self) -> Result<usize> {
        let num_output_ports = unsafe { ffi::AMidiDevice_getNumOutputPorts(self.ptr.as_ptr()) };
        if num_output_ports >= 0 {
            Ok(num_output_ports as usize)
        } else {
            Err(
                NdkMediaError::from_status(ffi::media_status_t(num_output_ports as c_int))
                    .unwrap_err(),
            )
        }
    }

    /// Gets the MIDI device type of this device.
    pub fn device_type(&self) -> Result<MidiDeviceType> {
        let device_type = unsafe { ffi::AMidiDevice_getType(self.ptr.as_ptr()) };
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
                construct(|res| ffi::AMidiInputPort_open(self.ptr.as_ptr(), port_number, res))?;
            Ok(MidiInputPort::from_ptr(NonNull::new_unchecked(input_port)))
        }
    }

    /// Opens the output port so that the client can receive data from it.
    pub fn open_output_port(&self, port_number: i32) -> Result<MidiOutputPort> {
        unsafe {
            let output_port =
                construct(|res| ffi::AMidiOutputPort_open(self.ptr.as_ptr(), port_number, res))?;
            Ok(MidiOutputPort::from_ptr(NonNull::new_unchecked(
                output_port,
            )))
        }
    }
}

impl Drop for MidiDevice {
    fn drop(&mut self) {
        let status = unsafe { ffi::AMidiDevice_release(self.ptr.as_ptr()) };
        NdkMediaError::from_status(status).unwrap();
    }
}

pub struct MidiInputPort<'a> {
    ptr: NonNull<ffi::AMidiInputPort>,
    _marker: PhantomData<&'a MidiDevice>,
}

impl<'a> fmt::Debug for MidiInputPort<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MidiInputPort")
            .field("inner", &self.ptr)
            .finish()
    }
}

impl<'a> MidiInputPort<'a> {
    /// Assumes ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::AMidiInputPort`].
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AMidiInputPort>) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    pub fn ptr(&self) -> NonNull<ffi::AMidiInputPort> {
        self.ptr
    }

    /// Sends data to this port.
    pub fn send(&self, buffer: &[u8]) -> Result<usize> {
        let num_bytes_sent = unsafe {
            ffi::AMidiInputPort_send(
                self.ptr.as_ptr(),
                buffer.as_ptr(),
                buffer.len() as ffi::size_t,
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

    /// Sends a message with a 'MIDI flush command code' to this port.
    ///
    /// This should cause a receiver to discard any pending MIDI data it may have accumulated and
    /// not processed.
    pub fn send_flush(&self) -> Result<()> {
        let result = unsafe { ffi::AMidiInputPort_sendFlush(self.ptr.as_ptr()) };
        NdkMediaError::from_status(result)
    }

    pub fn send_with_timestamp(&self, buffer: &[u8], timestamp: i64) -> Result<usize> {
        let num_bytes_sent = unsafe {
            ffi::AMidiInputPort_sendWithTimestamp(
                self.ptr.as_ptr(),
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
        unsafe { ffi::AMidiInputPort_close(self.ptr.as_ptr()) };
    }
}

pub struct MidiOutputPort<'a> {
    ptr: NonNull<ffi::AMidiOutputPort>,
    _marker: PhantomData<&'a MidiDevice>,
}

impl<'a> fmt::Debug for MidiOutputPort<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MidiOutputPort")
            .field("inner", &self.ptr)
            .finish()
    }
}

impl<'a> MidiOutputPort<'a> {
    /// Assumes ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::AMidiOutputPort`].
    pub unsafe fn from_ptr(ptr: NonNull<ffi::AMidiOutputPort>) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    pub fn ptr(&self) -> NonNull<ffi::AMidiOutputPort> {
        self.ptr
    }

    /// Receives the next pending MIDI message.
    ///
    /// To retrieve all pending messages, the client should repeatedly call this method until it
    /// returns [`Ok(MidiOpcode::NoMessage)`].
    ///
    /// Note that this is a non-blocking call. If there are no Midi messages are available, the
    /// function returns [`Ok(MidiOpcode::NoMessage)`] immediately (for 0 messages received).
    pub fn receive(&self, buffer: &mut [u8]) -> Result<MidiOpcode> {
        let mut opcode = 0i32;
        let mut timestamp = 0i64;
        let mut num_bytes_received: ffi::size_t = 0;
        let num_messages_received = unsafe {
            ffi::AMidiOutputPort_receive(
                self.ptr.as_ptr(),
                &mut opcode,
                buffer.as_mut_ptr(),
                buffer.len() as ffi::size_t,
                &mut num_bytes_received,
                &mut timestamp,
            )
        };

        match num_messages_received {
            r if r < 0 => {
                Err(NdkMediaError::from_status(ffi::media_status_t(r as c_int)).unwrap_err())
            }
            0 => Ok(MidiOpcode::NoMessage),
            1 => match opcode as c_uint {
                ffi::AMIDI_OPCODE_DATA => Ok(MidiOpcode::Data {
                    length: num_bytes_received as usize,
                    timestamp,
                }),
                ffi::AMIDI_OPCODE_FLUSH => Ok(MidiOpcode::Flush),
                _ => unreachable!("Unrecognized opcode {}", opcode),
            },
            r => unreachable!("Number of messages is positive integer {}", r),
        }
    }
}

impl<'a> Drop for MidiOutputPort<'a> {
    fn drop(&mut self) {
        unsafe { ffi::AMidiOutputPort_close(self.ptr.as_ptr()) };
    }
}
