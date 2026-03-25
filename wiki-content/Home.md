# Welcome to ApexSU Wiki

**ApexSU** is a next-generation Android root solution built on top of KernelSU. It features a Rust-first userspace, advanced stealth capabilities, and hardened security — all while maintaining full compatibility with Magisk modules.

---

## Quick Navigation

| Page | Description |
|:-----|:------------|
| [Installation Guide](Installation-Guide) | Step-by-step instructions to get ApexSU running |
| [FAQ](FAQ) | Common questions and troubleshooting tips |
| [Module Development](Module-Development) | Create your own systemless modules |
| [Security & Stealth](Security-and-Stealth) | How ApexSU stays hidden and secure |

---

## What Makes ApexSU Different?

| Feature | KernelSU | ApexSU |
|:--------|:--------:|:------:|
| Userspace Language | C++ | **Rust** |
| Anonymous Inode | `[ksu_driver]` | `[io_uring]` |
| Module Validation | Basic | **Comprehensive** |
| Clippy Enforcement | None | **Zero warnings** |
| Built-in Diagnostics | No | **`ksud diagnose`** |

---

## Architecture Overview

```text
+-------------------+       ioctl       +--------------------+
|    Manager App    | ----------------> |   Kernel Module    |
|  (Kotlin + Rust)  |                   |  (C, kernel-space) |
+---------+---------+                   +--------------------+
          |                                       ^
          v                                       |
+-------------------+       ioctl                 |
|       ksud        | ----------------------------+
|   (Rust daemon)   |
+-------------------+
```

- **Manager App**: Android app (Kotlin + Rust JNI) for managing root permissions
- **ksud**: Rust daemon handling module operations and root requests
- **Kernel Module**: Hooks syscalls to enforce UID-based root access

---

## Requirements

- Android 12 or higher
- Kernel 5.10+ (GKI 2.0 compatible)
- Unlocked bootloader

---

## Quick Links

- [GitHub Repository](https://github.com/qrjhamron/ApexSU)
- [Releases](https://github.com/qrjhamron/ApexSU/releases)
- [Report Issues](https://github.com/qrjhamron/ApexSU/issues)
- [Security Policy](https://github.com/qrjhamron/ApexSU/blob/main/SECURITY.md)

---

## License

ApexSU is licensed under **GPL-2.0**, same as KernelSU and the Linux kernel.
