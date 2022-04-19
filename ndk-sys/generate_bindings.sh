#!/bin/sh

while read ARCH && read TARGET ; do
    bindgen wrapper.h -o src/ffi_$ARCH.rs \
        --blocklist-item 'JNI\w+' \
        --blocklist-item 'C?_?JNIEnv' \
        --blocklist-item '_?JavaVM' \
        --blocklist-item '_?j\w+' \
        --newtype-enum '\w+_(result|status)_t' \
        --newtype-enum 'AndroidBitmapFormat' \
        --newtype-enum 'AHardwareBuffer_Format' \
        --newtype-enum 'AIMAGE_FORMATS' \
        -- \
        --sysroot="${ANDROID_NDK_ROOT}"/toolchains/llvm/prebuilt/linux-x86_64/sysroot/ --target=$TARGET
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
