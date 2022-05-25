#![cfg(feature = "api-level-28")]
//! Bindings for [`ffi::ASurfaceTexture`]
use crate::native_window::NativeWindow;
use jni_sys::{jobject, JNIEnv};
use std::ptr::NonNull;
use thiserror::Error;

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SurfaceTexture {
    ptr: NonNull<ffi::ASurfaceTexture>,
}

unsafe impl Send for SurfaceTexture {}

#[derive(Debug, Error)]
pub struct PosixError(pub i32);

impl std::fmt::Display for PosixError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Posix Error: {}", self.0)
    }
}

impl Drop for SurfaceTexture {
    fn drop(&mut self) {
        unsafe { ffi::ASurfaceTexture_release(self.ptr.as_ptr()) }
    }
}

impl SurfaceTexture {
    /// Assumes ownership of `ptr`
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an Android [`ffi::ASurfaceTexture`].
    pub unsafe fn from_ptr(ptr: NonNull<ffi::ASurfaceTexture>) -> Self {
        Self { ptr }
    }

    /// # Safety
    ///
    /// This function should be called with a healthy JVM pointer and with a non-null surface texture,
    /// which must be kept alive on the Java/Kotlin side.
    pub unsafe fn from_surface_texture(env: *mut JNIEnv, surface_texture: jobject) -> Option<Self> {
        let a_surface_texture_ptr = ffi::ASurfaceTexture_fromSurfaceTexture(env, surface_texture);
        let s = NonNull::new(a_surface_texture_ptr)?;
        Some(SurfaceTexture::from_ptr(s))
    }

    /// Returns native internal pointer
    pub fn ptr(&self) -> NonNull<ffi::ASurfaceTexture> {
        self.ptr
    }

    /// Returns a reference to an ANativeWindow (i.e. the Producer)
    /// for this SurfaceTexture. This is equivalent to Java's:
    /// Surface sur = new Surface(surfaceTexture);
    pub fn acquire_native_window(&mut self) -> Option<NativeWindow> {
        let native_window = unsafe { ffi::ASurfaceTexture_acquireANativeWindow(self.ptr.as_ptr()) };
        let n = NonNull::new(native_window)?;
        Some(unsafe { NativeWindow::from_ptr(n) })
    }

    /// Attach the SurfaceTexture to the OpenGL ES context that is current on the calling thread.
    pub fn attach_to_gl_context(&self, tex_name: u32) -> Result<(), PosixError> {
        let r = unsafe { ffi::ASurfaceTexture_attachToGLContext(self.ptr.as_ptr(), tex_name) };
        if r == 0 {
            Ok(())
        } else {
            Err(PosixError(r))
        }
    }

    /// Detach the SurfaceTexture from the OpenGL ES context that owns the OpenGL ES texture object.
    pub fn detach_from_gl_context(&self) -> Result<(), PosixError> {
        let r = unsafe { ffi::ASurfaceTexture_detachFromGLContext(self.ptr.as_ptr()) };
        if r == 0 {
            Ok(())
        } else {
            Err(PosixError(r))
        }
    }

    /// Retrieve the 4x4 texture coordinate transform matrix associated with the texture image set by the most recent call to updateTexImage.
    pub fn get_transform_matrix(&self) -> [f32; 16] {
        let mut r = [0f32; 16];
        unsafe { ffi::ASurfaceTexture_getTransformMatrix(self.ptr.as_ptr(), r.as_mut_ptr()) };
        r
    }

    /// Retrieve the timestamp associated with the texture image set by the most recent call to updateTexImage.
    pub fn timestamp(&self) -> i64 {
        unsafe { ffi::ASurfaceTexture_getTimestamp(self.ptr.as_ptr()) }
    }

    /// Update the texture image to the most recent frame from the image stream.
    pub fn update_tex_image(&self) -> Result<(), PosixError> {
        let r = unsafe { ffi::ASurfaceTexture_updateTexImage(self.ptr.as_ptr()) };
        if r == 0 {
            Ok(())
        } else {
            Err(PosixError(r))
        }
    }
}
