extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // FIXME: should it link -landroid -llog ?
    // With cargo-apk it's handled by ndk-build

    // TODO: generate for each SDK with -D__ANDROID_API__

    let ndk_dir = env::var("NDK_HOME").expect("Set NDK_HOME to the location of the NDK.");

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}/sysroot/usr/include", ndk_dir))
        // For some reason, some headers use `enum T : uint32_t { ... }`, a C++11 feature.
        // This is, however, supported in Clang 8.0 when compiling C, but not Clang 7 or GCC.
        .clang_arg("-xc++")
        .header("wrapper.h")
        .generate()
        .expect("Bindgen failed");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("ffi.rs"))
        .expect("Writing bindings failed");
}
