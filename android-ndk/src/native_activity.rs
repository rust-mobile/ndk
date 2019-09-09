//! Bindings for `ANativeActivity`
//!
//! See also [the NDK
//! docs](https://developer.android.com/ndk/reference/struct/a-native-activity.html)

use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr::NonNull;

/// An `ANativeActivity *`
///
/// This is either provided in `ANativeActivity_onCreate`, or accessible in
/// `android_native_app_glue`'s android_app.
pub struct NativeActivity {
    ptr: NonNull<ffi::ANativeActivity>,
}

impl NativeActivity {
    /// Create a `NativeActivity` from a pointer
    ///
    /// By calling this function, you assert that it is a valid pointer to a native
    /// `ANativeActivity`.
    pub unsafe fn from_ptr(ptr: NonNull<ffi::ANativeActivity>) -> Self {
        Self { ptr }
    }

    /// The pointer to the native `ANativeActivity`
    pub fn ptr(&self) -> NonNull<ffi::ANativeActivity> {
        self.ptr
    }

    /// The platform's SDK version code
    pub fn sdk_version(&self) -> i32 {
        unsafe { self.ptr.as_ref().sdkVersion }
    }

    /// Path to this application's internal data directory
    pub fn internal_data_path(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.ptr.as_ref().internalDataPath) }
    }

    /// Path to this application's external (removable, mountable) data directory
    pub fn external_data_path(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.ptr.as_ref().externalDataPath) }
    }

    /// This app's asset manager, which can be used to access assets from the `.apk` file.
    pub fn asset_manager(&self) -> crate::asset::AssetManager {
        unsafe {
            crate::asset::AssetManager::from_ptr(
                NonNull::new(self.ptr.as_ref().assetManager).unwrap(),
            )
        }
    }

    /// Instance data associated with the activity
    pub fn instance(&self) -> *mut c_void {
        unsafe { self.ptr.as_ref().instance }
    }

    /// Instance data associated with the activity
    pub fn instance_mut(&mut self) -> &mut *mut c_void {
        unsafe { &mut self.ptr.as_mut().instance }
    }

    /// This processe's `JavaVM` object.
    ///
    /// ```no_run
    /// #let native_activity: NativeActivity = unimplemented!();
    /// let vm = native_activity.vm();
    /// let env = vm.attach_current_thread();
    /// // Do JNI with env ...
    /// ```
    pub fn vm(&self) -> jni::JavaVM {
        unsafe { jni::JavaVM::from_raw(self.ptr.as_ref().vm as *mut _).unwrap() }
    }

    /// The `android.app.NativeActivity` instance
    ///
    /// In the JNI, this is named `clazz`; however, as the docs say, "it should really be named
    /// 'activity' instead of 'clazz', since it's a reference to the NativeActivity instance.
    pub fn activity(&self) -> jni::objects::JObject<'_> {
        unsafe {
            jni::objects::JObject::from(&self.ptr.as_ref().clazz as *const _ as jni::sys::jobject)
        }
    }

    /// Path to the directory with the application's OBB files.
    ///
    /// Only available as of Honeycomb (Android 3.0+, API level 11+)
    pub unsafe fn obb_path(&self) -> &CStr {
        CStr::from_ptr(self.ptr.as_ref().obbPath)
    }
}
