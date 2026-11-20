#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
export IPHONEOS_DEPLOYMENT_TARGET=17.0
export CMAKE_POLICY_VERSION_MINIMUM=3.5
unset SDKROOT
platform="$1"
case "$platform" in
android)
    cargo ndk --platform 31 -t arm64-v8a -o mobile/android/app/build/jniLibs build -p adamo-play --release
    library=target/aarch64-linux-android/release/libadamo_play.so
    language=kotlin
    bindings=mobile/android/app/build/generated/uniffi
    ;;
iphoneos)
    cargo build -p adamo-play --release --target aarch64-apple-ios
    library=target/aarch64-apple-ios/release/libadamo_play.a
    language=swift
    bindings=mobile/ios/Generated
    ;;
iphonesimulator)
    cargo build -p adamo-play --release --target aarch64-apple-ios-sim
    library=target/aarch64-apple-ios-sim/release/libadamo_play.a
    language=swift
    bindings=mobile/ios/Generated
    ;;
esac
cargo run -p adamo-play --release --bin uniffi-bindgen -- generate --library "$library" --language "$language" --out-dir "$bindings"
