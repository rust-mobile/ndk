//! Bindings for [`ffi::ASurfaceTexture`]
use crate::native_window::NativeWindow;
use jni_sys::jobject;
use jni_sys::JNIEnv;
use std::ptr::NonNull;

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SurfaceTexture {
    ptr: NonNull<ffi::ASurfaceTexture>,
}

unsafe impl Send for SurfaceTexture {}
unsafe impl Sync for SurfaceTexture {}

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

    pub fn ptr(&self) -> NonNull<ffi::ASurfaceTexture> {
        self.ptr
    }

    pub fn acquire_native_window(&mut self) -> Option<NativeWindow> {
        let native_window = unsafe { ffi::ASurfaceTexture_acquireANativeWindow(self.ptr.as_ptr()) };
        let n = NonNull::new(native_window)?;
        Some(unsafe { NativeWindow::from_ptr(n) })
    }

    pub fn attach_to_gl_context(&self, tex_name: u32) -> Result<(), i32> {
        let r = unsafe { ffi::ASurfaceTexture_attachToGLContext(self.ptr.as_ptr(), tex_name) };
        if r == 0 {
            Ok(())
        } else {
            Err(r)
        }
    }

    pub fn detach_from_gl_context(&self) -> Result<(), i32> {
        let r = unsafe { ffi::ASurfaceTexture_detachFromGLContext(self.ptr.as_ptr()) };
        if r == 0 {
            Ok(())
        } else {
            Err(r)
        }
    }

    pub fn get_transform_matrix(&self) -> [f32; 16] {
        let mut r = [0f32; 16];
        unsafe { ffi::ASurfaceTexture_getTransformMatrix(self.ptr.as_ptr(), r.as_mut_ptr()) };
        r
    }

    pub fn timestamp(&self) -> i64 {
        unsafe { ffi::ASurfaceTexture_getTimestamp(self.ptr.as_ptr()) }
    }

    pub fn update_tex_image(&self) -> Result<(), i32> {
        let r = unsafe { ffi::ASurfaceTexture_updateTexImage(self.ptr.as_ptr()) };
        if r == 0 {
            Ok(())
        } else {
            Err(r)
        }
    }

    pub fn from_surface_texture(env: *mut JNIEnv, surface_texture: jobject) -> Option<Self> {
        let a_surface_texture_ptr =
            unsafe { ffi::ASurfaceTexture_fromSurfaceTexture(env, surface_texture) };
        let s = NonNull::new(a_surface_texture_ptr)?;
        Some(unsafe { SurfaceTexture::from_ptr(s) })
    }
}
