#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn ANativeActivity_onCreate(
    activity: *mut std::os::raw::c_void,
    saved_state: *mut std::os::raw::c_void,
    saved_state_size: usize,
) {
    std::env::set_var("RUST_BACKTRACE", "1");
    ndk_glue::init(
        activity as _,
        saved_state as _,
        saved_state_size as _,
        app_main,
    );
}

pub fn app_main() {
    println!("hello world");
}
