#!/bin/bash

set -ex

rustup component add rustfmt

rustup target add arm-linux-androideabi
rustup target add armv7-linux-androideabi
rustup target add aarch64-linux-android
rustup target add i686-linux-android
rustup target add x86_64-linux-android

curl -LO https://dl.google.com/android/repository/android-ndk-r20-linux-x86_64.zip
unzip android-ndk-r20-linux-x86_64.zip -d $HOME
rm android-ndk-r20-linux-x86_64.zip
