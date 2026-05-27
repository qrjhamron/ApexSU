<p align="center">
  <img src="manager/app/src/main/assets/logo/apexsu_logo.svg" alt="ApexSU Logo" width="140" />
</p>

<h1 align="center">ApexSU</h1>

<p align="center">
  <a href="https://github.com/qrjhamron/ApexSU/releases">
    <img src="https://img.shields.io/github/v/release/qrjhamron/ApexSU?display_name=tag&sort=semver" alt="GitHub Release" />
  </a>
  <a href="https://github.com/qrjhamron/ApexSU/stargazers">
    <img src="https://img.shields.io/github/stars/qrjhamron/ApexSU?style=flat" alt="GitHub Stars" />
  </a>
  <a href="https://github.com/qrjhamron/ApexSU/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/qrjhamron/ApexSU/ci.yml?branch=main&label=CI" alt="CI Status" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/License-GPL--2.0-blue.svg" alt="License: GPL-2.0" />
  </a>
</p>

ApexSU is a security-focused Android root management project based on KernelSU, with a Rust-first userspace, stricter kernel ABI checks, safer module handling, and audit-focused release practices.

## Project Overview

ApexSU targets transparent, maintainable, device-owner root management. The project prioritizes kernel safety, explicit authorization checks, and verifiable build/test evidence over stealth claims.

## Releases

- Latest builds and assets: [GitHub Releases](https://github.com/qrjhamron/ApexSU/releases)
- Release artifacts include APK and per-KMI `kernelsu.ko` variants.

## What Is ApexSU?

- Kernel-integrated root management for Android.
- A KernelSU-derived codebase with ApexSU-specific hardening and tooling.
- A mixed C (kernel) + Rust (userspace tooling/daemon) + Kotlin (manager UI) architecture.

## What ApexSU Is Not

- Not a banking/anti-cheat/DRM bypass project.
- Not a stealth/evasion framework.
- Not a malware or unauthorized-access tool.

## Current Status

- Not release-ready.
- Kernel hardening is in progress.
- Android/GKI target build evidence is still required.
- KUnit and runtime stress verification are not yet complete.
- Closed tester rollout is not open by default.

## Architecture

- `kernel/`: privileged kernel logic, hooks, policy enforcement, ioctl handling.
- `userspace/ksud`: Rust daemon and policy/module processing.
- `userspace/ksuinit`: boot handoff/init path helpers.
- `manager/`: Android manager app and JNI bridge.

## Security Model

- Fail-closed checks for manager identity and privileged operations.
- Strict input validation for ABI-facing structures and module metadata.
- Security-sensitive behavior is documented and tested where feasible.

## Kernel/Userspace ABI

ABI contracts are defined in kernel headers and mirrored in Rust:

- Kernel ioctl structures and command IDs must remain layout-compatible.
- `kernel/rust/abi_checker` provides host-side ABI parity tests.

## Manager App

The manager app is the user-facing control plane for root policy decisions, diagnostics, and operational visibility. Kernel enforcement remains in kernel/userspace backends.

## Modules

Module handling is supported with explicit validation. Invalid ZIP entries (for example symlink/special-file abuse paths) are rejected during validation.

## Diagnostics

ApexSU includes diagnostic paths in userspace and manager flows to aid reproducible bug reporting and recovery-oriented testing.

## Compatibility Status

Compatibility claims are conservative:

- No claim of WSA/ChromeOS/container support without build + smoke evidence.
- No claim of real-device support without verified target-device testing.

## Build Verification

See `kernel/docs/BUILD_VERIFICATION.md`. Host builds are weak evidence; Android/GKI target builds are required for strong kernel verification.

## Testing Status

Current matrix includes Rust unit tests and ABI checker tests. Remaining blockers include target-kernel module builds and runtime stress/KUnit execution evidence.

## Roadmap

- Complete kernel hardening review cycle.
- Expand target-kernel validation and stress testing.
- Improve CI/release gating and documentation accuracy.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Keep changes scoped, include verification commands, and avoid unrelated refactors in security patches.

## Security Disclosure

See [SECURITY.md](SECURITY.md). Do not report vulnerabilities in public issues.

## Credits and Upstream Relationship

ApexSU builds on the KernelSU project. We retain attribution to upstream KernelSU where applicable while developing ApexSU-specific hardening, tooling, and documentation.

## License

GPL-2.0. See [LICENSE](LICENSE).
