#!/bin/bash

set -ex

# Make sure the package is removed since it may end up in the AVD cache. This causes
# INSTALL_FAILED_UPDATE_INCOMPATIBLE errors when the debug keystore is regenerated,
# as it is not stored/cached on the CI:
# https://github.com/rust-windowing/android-ndk-rs/blob/240389f1e281f582b84a8049e2afaa8677d901c2/ndk-build/src/ndk.rs#L308-L332
adb uninstall rust.example.hello_world || true

if [ -z "$1" ];
then
    cargo apk run -p ndk-examples --target x86_64-linux-android --example hello_world
else
    adb install -r "$1/hello_world.apk"
    adb shell am start -a android.intent.action.MAIN -n "rust.example.hello_world/android.app.NativeActivity"
fi

sleep 30s

adb logcat *:E hello-world:V -d | tee ~/logcat.log

if grep 'hello world' ~/logcat.log;
then
    echo "App running"
else
    echo "::error::App not running"
    exit 1
fi

ERROR_MSG=$(grep -e 'thread.*panicked at' "$HOME"/logcat.log | true)
if [ -z "${ERROR_MSG}" ];
then
    exit 0
else
    echo "::error::${ERROR_MSG}"
    exit 1
fi
