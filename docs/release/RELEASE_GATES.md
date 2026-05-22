# ApexSU Release Gate Model

This repository separates release evidence into four gates. Host-only green is not
release proof.

## Gate 1: Host Smoke (weak evidence)

Purpose: detect obvious regressions on a generic Linux host.

Command:

```sh
just gate-host
```

This gate is weak evidence only. Passing does not prove Android/GKI behavior.

## Gate 2: Android-target `ksud` build/test (required)

Purpose: compile Android-only `cfg(target_os = "android")` code paths and test
objects with Android NDK toolchain.

Command:

```sh
just gate-android
```

Prerequisites:

- `ANDROID_NDK_HOME` or `ANDROID_NDK_ROOT`
- `aarch64-linux-android-clang`

## Gate 3: Android/GKI kernel module build (required)

Purpose: prove `CONFIG_KSU=m` objects build against Android/GKI headers, not
just generic host headers.

Command:

```sh
just gate-kernel
```

Prerequisites:

- `ANDROID_GKI_OUT` points to prepared target kernel output directory

## Gate 4: KUnit + runtime stress (required/manual evidence)

Purpose: verify high-risk kernel lifecycle behavior under target-kernel test
conditions.

Command template:

```sh
just gate-kunit
```

Runtime stress (manual) must include unload/reload plus callback producers, with
KASAN/lockdep/list-debug enabled, and no UAF/list corruption/lock warnings.

## Release Block Rule

Release is blocked unless all required gates are evidenced in
`docs/release/RELEASE_EVIDENCE.md` and pass
`scripts/release/check_release_readiness.sh`.

If a required gate cannot run in the current environment, release readiness is
**failed**, not inferred.
