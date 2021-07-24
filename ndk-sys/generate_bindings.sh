#!/bin/sh

while read ARCH && read TARGET ; do
    bindgen wrapper.h -o src/ffi_$ARCH.rs -- --sysroot="${ANDROID_NDK_ROOT}"/toolchains/llvm/prebuilt/linux-x86_64/sysroot/ --target=$TARGET
done << EOF
arm
arm-linux-androideabi
aarch64
aarch64-linux-android
i686
i686-linux-android
x86_64
x86_64-linux-android
EOF
