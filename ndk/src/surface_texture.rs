//! Bindings for [`ffi::ASurfaceTexture`]
#![cfg(feature = "api-level-28")]

use super::posix::PosixError;
use crate::native_window::NativeWindow;
use jni_sys::{jobject, JNIEnv};
use std::ptr::NonNull;

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SurfaceTexture {
    ptr: NonNull<ffi::ASurfaceTexture>,
}

unsafe impl Send for SurfaceTexture {}

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
    /// Get a reference to the native ASurfaceTexture from the corresponding java object.
    //
    /// The caller must keep a reference to the Java `SurfaceTexture` during the lifetime of the returned [`SurfaceTexture`].
    /// Failing to do so could result in the [`SurfaceTexture`] to stop functioning properly once the Java object gets finalized.
    /// However, this will not result in program termination.
    pub unsafe fn from_surface_texture(env: *mut JNIEnv, surface_texture: jobject) -> Option<Self> {
        let a_surface_texture_ptr = ffi::ASurfaceTexture_fromSurfaceTexture(env, surface_texture);
        let s = NonNull::new(a_surface_texture_ptr)?;
        Some(SurfaceTexture::from_ptr(s))
    }

    /// Returns a pointer to the native [`ASurfaceTexture`](ffi::ASurfaceTexture).
    pub fn ptr(&self) -> NonNull<ffi::ASurfaceTexture> {
        self.ptr
    }

    /// Returns a reference to a [`NativeWindow`] (i.e. the Producer) for this [`SurfaceTexture`].
    ///
    /// This is equivalent to Java's:
    /// ```java
    /// Surface sur = new Surface(surfaceTexture);
    /// ```
    pub fn acquire_native_window(&mut self) -> Option<NativeWindow> {
        let native_window = unsafe { ffi::ASurfaceTexture_acquireANativeWindow(self.ptr.as_ptr()) };
        let n = NonNull::new(native_window)?;
        Some(unsafe { NativeWindow::from_ptr(n) })
    }

    /// Attach the [`SurfaceTexture`] to the OpenGL ES context that is current on the calling thread.
    /// A new OpenGL ES texture object is created and populated with the SurfaceTexture image frame
    /// that was current at the time of the last call to ASurfaceTexture_detachFromGLContext.
    /// This new texture is bound to the [`GL_TEXTURE_EXTERNAL_OES`] texture target.
    /// This can be used to access the [`SurfaceTexture`] image contents from multiple OpenGL ES contexts.
    /// Note, however, that the image contents are only accessible from one OpenGL ES context at a time.
    pub fn attach_to_gl_context(&self, tex_name: u32) -> Result<(), PosixError> {
        let r = unsafe { ffi::ASurfaceTexture_attachToGLContext(self.ptr.as_ptr(), tex_name) };
        if r == 0 {
            Ok(())
        } else {
            Err(PosixError(r))
        }
    }

    /// Detach the [`SurfaceTexture`] from the OpenGL ES context that owns the OpenGL ES texture object.
    /// This call must be made with the OpenGL ES context current on the calling thread. The OpenGL
    /// ES texture object will be deleted as a result of this call. After calling this method all
    /// calls to [`update_tex_image()`][Self::update_tex_image()] will fail until a successful call to
    /// [`attach_gl_context()`][Self::attach_gl_context()] is made.
    /// This can be used to access the [`SurfaceTexture`] image contents from multiple OpenGL ES contexts.
    /// Note, however, that the image contents are only accessible from one OpenGL ES context at a time.
    pub fn detach_from_gl_context(&self) -> Result<(), PosixError> {
        let r = unsafe { ffi::ASurfaceTexture_detachFromGLContext(self.ptr.as_ptr()) };
        if r == 0 {
            Ok(())
        } else {
            Err(PosixError(r))
        }
    }

    /// Retrieve the 4x4 texture coordinate transform matrix associated with the texture image set by the most recent call to [`update_tex_image()`][Self::update_tex_image()].
    pub fn transform_matrix(&self) -> [f32; 16] {
        let mut r = [0f32; 16];
        unsafe { ffi::ASurfaceTexture_getTransformMatrix(self.ptr.as_ptr(), r.as_mut_ptr()) };
        r
    }

    /// Retrieve the 4x4 texture coordinate transform matrix associated with the texture image set by the most recent call to updateTexImage.
    /// This transform matrix maps 2D homogeneous texture coordinates of the form (s, t, 0, 1) with s and t in the inclusive range [0, 1] to the texture coordinate
    /// that should be used to sample that location from the texture. Sampling the texture outside of the range of this transform is undefined.
    /// The matrix is stored in column-major order so that it may be passed directly to OpenGL ES via the glLoadMatrixf or glUniformMatrix4fv functions.
    /// This timestamp is in nanoseconds, and is normally monotonically increasing. The timestamp should be unaffected by time-of-day adjustments, and for
    /// a camera should be strictly monotonic but for a MediaPlayer may be reset when the position is set. The specific meaning and zero point of the timestamp depends
    /// on the source providing images to the SurfaceTexture. Unless otherwise specified by the image source, timestamps cannot generally be compared across SurfaceTexture
    /// instances, or across multiple program invocations. It is mostly useful for determining time offsets between subsequent frames
    /// For EGL/Vulkan producers, this timestamp is the desired present time set with the EGL_ANDROID_presentation_time or VK_GOOGLE_display_timing extensions
    pub fn timestamp(&self) -> i64 {
        unsafe { ffi::ASurfaceTexture_getTimestamp(self.ptr.as_ptr()) }
    }

    /// Update the texture image to the most recent frame from the image stream.
    /// This may only be called while the OpenGL ES context that owns the texture is
    /// current on the calling thread. It will implicitly bind its texture to the
    /// `GL_TEXTURE_EXTERNAL_OES` texture target.
    pub fn update_tex_image(&self) -> Result<(), PosixError> {
        let r = unsafe { ffi::ASurfaceTexture_updateTexImage(self.ptr.as_ptr()) };
        if r == 0 {
            Ok(())
        } else {
            Err(PosixError(r))
        }
    }
}
