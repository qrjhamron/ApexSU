# Building the Rust JNI Bridge for KernelSU Manager

This Rust crate (`kernelsu-jni`) replaces the C++ JNI bridge (`ksu.cc` + `jni.cc`).
It produces `libkernelsu.so` loaded by `Natives.kt` via `System.loadLibrary("kernelsu")`.

## Prerequisites

- Rust toolchain (rustup)
- Android NDK (set `ANDROID_NDK_HOME`)
- `cargo-ndk`: `cargo install cargo-ndk`
- Android target: `rustup target add aarch64-linux-android`

## Build

```bash
cd manager/app/src/main/rust/kernelsu-jni

# Release build for arm64
cargo ndk -t arm64-v8a build --release

# Copy the shared library into the Manager's jniLibs
mkdir -p ../../jniLibs/arm64-v8a
cp target/aarch64-linux-android/release/libkernelsu.so ../../jniLibs/arm64-v8a/
```

## Verification

After copying, build the Manager app normally:

```bash
cd manager
./gradlew assembleRelease
```

## Migration from C++

Once the Rust library is verified on-device:

1. Remove `manager/app/src/main/cpp/` (CMakeLists.txt, ksu.cc, jni.cc, ksu.h)
2. Remove the `externalNativeBuild` block from `app/build.gradle.kts` if present
3. The Kotlin `Natives.kt` file requires **no changes** — the JNI function names
   and library name are identical.
