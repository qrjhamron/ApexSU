alias bk := build_ksud
alias bm := build_manager
alias gate-host := gate_host_smoke
alias gate-android := gate_android_ksud
alias gate-kernel := gate_kernel_gki
alias gate-kunit := gate_kunit
alias gate-release := gate_release_readiness

build_ksud:
    cross build --target aarch64-linux-android --release --manifest-path ./userspace/ksud/Cargo.toml

build_manager: build_ksud
    cp userspace/ksud/target/aarch64-linux-android/release/ksud manager/app/src/main/jniLibs/arm64-v8a/libksud.so
    cd manager && ./gradlew aDebug

clippy:
    cargo fmt --manifest-path ./userspace/ksud/Cargo.toml
    cross clippy --target x86_64-pc-windows-gnu --release --manifest-path ./userspace/ksud/Cargo.toml
    cross clippy --target aarch64-linux-android --release --manifest-path ./userspace/ksud/Cargo.toml

gate_host_smoke:
    cargo test --manifest-path userspace/ksud/Cargo.toml --locked
    cargo test --manifest-path userspace/ksuinit/Cargo.toml --locked
    cargo test --manifest-path kernel/rust/abi_checker/Cargo.toml --locked
    make -C /lib/modules/$(uname -r)/build M=$PWD/kernel modules

gate_android_ksud:
    scripts/release/check_toolchain_prereqs.sh --skip-gki-out
    cargo ndk -t arm64-v8a build --release --manifest-path userspace/ksud/Cargo.toml
    cargo ndk -t arm64-v8a test --no-run --manifest-path userspace/ksud/Cargo.toml

gate_kernel_gki:
    scripts/release/check_toolchain_prereqs.sh
    make -C "$ANDROID_GKI_OUT" M="$PWD/kernel" CONFIG_KSU=m ARCH=arm64 LLVM=1 modules

gate_kunit:
    scripts/release/check_toolchain_prereqs.sh
    make -C "$ANDROID_GKI_OUT" ARCH=arm64 LLVM=1 O="$ANDROID_GKI_OUT" kunit

gate_release_readiness:
    scripts/release/check_release_readiness.sh
