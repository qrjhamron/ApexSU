# ApexSU

A professional, advanced Android root solution based on KernelSU, designed with a Rust-first userspace. ApexSU prioritizes security hardening, precise control, and stealth to deliver a robust environment for Android developers and advanced users.

## About

ApexSU is a fork and evolution of KernelSU that integrates deeply with the Android kernel to provide root privileges. Unlike traditional userspace-only or hybrid root solutions, ApexSU operates at the kernel level. Its primary distinction is a rewritten, Rust-first userspace designed to minimize memory-related vulnerabilities and reduce the overall attack surface. This makes ApexSU fundamentally more secure, harder to detect, and significantly more resilient against unauthorized access.

## Features

ApexSU introduces several key advantages over standard KernelSU:

*   **Rust-First Userspace:** The core daemon (`ksud`) and JNI bridges are implemented in Rust, ensuring memory safety, preventing buffer overflows, and delivering exceptional performance.
*   **Security Hardening:** Enforces strict compile-time checks, zero dead-code tolerance, and comprehensive module validation (preventing path traversal and verifying payload integrity).
*   **Advanced Stealth Features:** Employs advanced techniques to hide root status from detection mechanisms. It utilizes anonymous inodes (e.g., `[io_uring]` instead of recognizable driver names) and avoids creating detectable entries in `/proc`, `/sys`, or `/dev`.
*   **Kernel-Level Execution:** Hooks syscalls directly in kernel-space to enforce a strict UID allowlist for root access, bypassing conventional userspace root detection.

## Architecture

The system operates through a streamlined communication channel using IOCTLs, linking the Android manager application, the Rust daemon, and the kernel module.

```text
┌─────────────────┐     ioctl      ┌──────────────────┐
│  Manager App    │ ────────────→  │  Kernel Module   │
│  (Kotlin + Rust)│                │  (C, kernel-space)│
└────────┬────────┘                └──────────────────┘
         │                                  ↑
         ↓                                  │
┌─────────────────┐     ioctl               │
│      ksud       │ ────────────────────────┘
│  (Rust daemon)  │
└─────────────────┘
```

## Requirements

To run ApexSU, your device must meet the following criteria:

*   Android 12 or higher.
*   Kernel version 5.10 or higher (GKI 2.0 compatible).
*   An unlocked bootloader.

## Installation

1.  Download the latest ApexSU Manager APK and the corresponding kernel image/module from the project's release page.
2.  Flash the provided boot image or kernel module to your device via `fastboot` or your preferred flashing tool.
3.  Install the ApexSU Manager APK.
4.  Open the Manager app to verify the installation and manage root access.

## Usage

*   **Granting Root Access:** Open the ApexSU Manager application. Navigate to the superuser list and explicitly grant root permissions to the desired applications.
*   **Managing Modules:** Use the Manager app to install, enable, disable, or remove systemless modules.
*   **Diagnostics:** The integrated Rust daemon includes built-in health checks that can be queried for troubleshooting and verification.

### Building from Source

To compile the userspace daemon and the manager APK, ensure you have the Rust stable toolchain (1.82+), Android NDK (r29), JDK 21, and the Android SDK installed.

**Compile the Userspace Daemon (`ksud`):**
```bash
cd userspace/ksud
cargo ndk -t arm64-v8a build --release
```

**Compile the Manager APK:**
```bash
cd manager
./gradlew assembleRelease
```

## Contributing

We welcome contributions from the Android development and security communities. Please review our `CONTRIBUTING.md` file for detailed coding standards, Rust clippy policies (we strictly enforce `clippy::all` and `clippy::pedantic`), and submission guidelines. For security-related bugs, please open a private GitHub Security Advisory instead of a public issue.

## License

ApexSU is licensed under the GPL-2.0 License, inheriting the open-source commitments of KernelSU and the Linux kernel. See the `LICENSE` file for full details.
