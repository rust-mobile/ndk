#!/bin/bash

set -ex

export NDK_HOME=$(pwd)/android-ndk-r20

# Run tests
cargo test --package android-ndk-sys --lib --target=x86_64-unknown-linux-gnu
cargo test --package android-ndk --target=x86_64-unknown-linux-gnu
