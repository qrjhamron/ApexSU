# Kernel Build Verification

This module needs two different build checks before release decisions are made.

## Weak Host Build

Command:

```sh
make -C /lib/modules/$(uname -r)/build M=$PWD/kernel modules
```

This is a weak sanity check only. On a generic Linux host it may skip the
`CONFIG_KSU=m` object path, so a pass here does not prove that the ApexSU
module sources compile.

## Forced Host Module Build

Command:

```sh
make -C /lib/modules/$(uname -r)/build M=$PWD/kernel CONFIG_KSU=m modules
```

This is stronger than the weak host build because it forces the external module
object list. It can still fail on generic distro headers for Android/SELinux
internal APIs that are present only in a compatible Android/GKI kernel tree.

Known host-header limitation observed in this workspace:

```text
kernel/file_wrapper.c:17:10: fatal error: objsec.h: No such file or directory
```

Do not hide this failure by disabling `file_wrapper.o` or removing SELinux SID
handling. That would change security behavior to satisfy an unrepresentative
host build.

## Required Android/GKI Target Build

Before device testing or release, run the forced module build against the exact
Android/GKI target kernel source and configuration used for the target image.

Expected real verification command (example template):

```sh
make -C "$ANDROID_GKI_OUT" M="$PWD/kernel" CONFIG_KSU=m ARCH=arm64 LLVM=1 modules
```

`$ANDROID_GKI_OUT` must point to the prepared Android/GKI kernel build output
directory for the same kernel/config/device family you plan to ship.

Minimum evidence to record:

- target kernel source tree and branch/tag
- target architecture, for example `arm64` or `x86_64`
- exact `.config` or defconfig source
- full build command
- complete result for `CONFIG_KSU=m`
- whether `file_wrapper.c`, `pkg_observer.c`, `supercalls.c`, and
  `throne_tracker.c` compiled

Until that target build passes, host builds are not release proof.

Release decisions must also pass `scripts/release/check_release_readiness.sh`,
which enforces explicit Android/GKI/KUnit/runtime evidence in
`docs/release/RELEASE_EVIDENCE.md`.

## KUnit

KUnit tests in this repository are compile-gated with `CONFIG_KUNIT`. They are
not executed by the weak host build. Run them in a compatible kernel test
environment and record the suite names and dmesg/TAP output before treating
KUnit coverage as release evidence.

## Task Work Unload Stress (Manual Verification)

Task-work callbacks use function pointers in module text. Release validation
must include a stress pass proving no callback runs after unload begins.

Recommended target-kernel stress profile:

- enable memory/race diagnostics: `CONFIG_KASAN`, `CONFIG_DEBUG_LIST`,
  `CONFIG_PROVE_LOCKING`, and `CONFIG_KASAN_STACK`.
- repeatedly trigger callback producers:
  - reboot-magic fd install path
  - manager `setresuid` path
  - mount-namespace setup path
  - umount scheduling path
  - ksud zygote/post-fs-data path
- in parallel, perform module unload/reload loops (`rmmod`/`insmod`) on a
  disposable target device or test kernel.
- fail the run on any KASAN use-after-free, list corruption, lockdep warning,
  or callback into freed module text.
