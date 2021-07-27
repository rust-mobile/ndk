#!/bin/bash

set -e

rustup target install x86_64-linux-android
cargo install --path "$GITHUB_WORKSPACE/cargo-apk" --force
cd "$GITHUB_WORKSPACE/ndk-examples"
cargo apk run --example hello_world --target x86_64-linux-android

sleep 30s

adb logcat *:E hello-world:V -d > ~/logcat.log

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
