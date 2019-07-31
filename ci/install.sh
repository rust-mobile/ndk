#!/bin/bash

set -ex

# Download NDK
wget https://dl.google.com/android/repository/android-ndk-r20-linux-x86_64.zip
unzip -q android-ndk-r20-linux-x86_64.zip android-ndk-r20/sysroot/*
