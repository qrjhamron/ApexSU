<div align="center">

# ApexSU

**Next-gen Android root. Rust-powered. Stealth-hardened. Zero compromise.**

[![CI](https://img.shields.io/github/actions/workflow/status/qrjhamron/ApexSU/ci.yml?branch=main&style=for-the-badge&logo=github&label=CI)](https://github.com/qrjhamron/ApexSU/actions/workflows/ci.yml)
[![Clippy](https://img.shields.io/github/actions/workflow/status/qrjhamron/ApexSU/clippy.yml?branch=main&style=for-the-badge&logo=rust&label=Clippy)](https://github.com/qrjhamron/ApexSU/actions/workflows/clippy.yml)
[![Manager Build](https://img.shields.io/github/actions/workflow/status/qrjhamron/ApexSU/build-manager.yml?branch=main&style=for-the-badge&logo=android&label=Manager)](https://github.com/qrjhamron/ApexSU/actions/workflows/build-manager.yml)

[![Version](https://img.shields.io/badge/version-0.1.5--beta-blue?style=for-the-badge&logo=semver)](https://github.com/qrjhamron/ApexSU/releases)
[![License](https://img.shields.io/badge/license-GPL--2.0-green?style=for-the-badge&logo=gnu)](LICENSE)
[![Android](https://img.shields.io/badge/Android-12%2B-34A853?style=for-the-badge&logo=android)](https://developer.android.com)
[![Kernel](https://img.shields.io/badge/Kernel-5.10%2B-orange?style=for-the-badge&logo=linux)](https://www.kernel.org)

---

**[Documentation](https://github.com/qrjhamron/ApexSU/wiki)** | **[Releases](https://github.com/qrjhamron/ApexSU/releases)** | **[Issues](https://github.com/qrjhamron/ApexSU/issues)** | **[Discussions](https://github.com/qrjhamron/ApexSU/discussions)**

</div>

---

## What is ApexSU?

ApexSU is a hardened fork of [KernelSU](https://kernelsu.org) that rewrites the userspace in **Rust** for memory safety, tighter security, and better stealth. It hooks directly into the kernel via syscall interception — no `/proc`, no `/sys`, no `/dev` footprints. Just clean, silent root.

> **TL;DR:** KernelSU, but with less C, more Rust, and way harder to detect.

---

## Why ApexSU?

| Feature | KernelSU | ApexSU |
|:--------|:--------:|:------:|
| JNI Bridge | C++ | **Rust** |
| Anon Inode Name | `[ksu_driver]` | `[io_uring]` |
| Module Validation | Basic | **Path traversal + size + field checks** |
| Built-in Diagnostics | No | **Yes (`ksud diagnose`)** |
| Dead Code | Tolerated | **Zero tolerance** |
| Clippy Policy | Not enforced | **`clippy::all` + `clippy::pedantic`** |

---

## Architecture

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

Communication happens over **IOCTLs on anonymous inodes** — invisible to standard detection tools.

---

## Codebase Stats

> Generated with `cloc` — real numbers, no fluff.

| Language | Files | Lines of Code |
|:---------|------:|--------------:|
| Kotlin | 73 | 14,441 |
| Rust | 28 | 6,559 |
| C | 21 | 5,595 |
| C/C++ Header | 22 | 670 |
| C++ | 2 | 425 |
| **Total** | **403** | **52,526** |

Rust makes up **~24%** of the core codebase and growing.

---

## Features

- **Rust-First Daemon** — `ksud` is written in Rust. Memory-safe, fast, reliable.
- **Stealth Mode** — Hides from root detection by mimicking kernel subsystems.
- **Zero Clippy Warnings** — Enforced across all Rust code. No exceptions.
- **Systemless Modules** — Full Magisk-compatible module support.
- **Health Diagnostics** — Run `ksud diagnose` to check system integrity.

---

## Contributing

We love PRs! Before you dive in:

1. Read [`CONTRIBUTING.md`](CONTRIBUTING.md) for code standards.
2. Security bugs? Open a **private Security Advisory** — not a public issue.
3. Keep Clippy happy: `clippy::all` + `clippy::pedantic` = zero warnings.

---

## License

**GPL-2.0** — Same as KernelSU and the Linux kernel.

See [`LICENSE`](LICENSE) for details.

---

<div align="center">

**Built with Rust. Hardened for Android. Made for power users.**

[![Stars](https://img.shields.io/github/stars/qrjhamron/ApexSU?style=social)](https://github.com/qrjhamron/ApexSU)
[![Forks](https://img.shields.io/github/forks/qrjhamron/ApexSU?style=social)](https://github.com/qrjhamron/ApexSU/fork)
[![Watchers](https://img.shields.io/github/watchers/qrjhamron/ApexSU?style=social)](https://github.com/qrjhamron/ApexSU)

</div>

## Star History

<a href="https://www.star-history.com/?repos=qrjhamron%2FApexSU&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/image?repos=qrjhamron/ApexSU&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/image?repos=qrjhamron/ApexSU&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/image?repos=qrjhamron/ApexSU&type=date&legend=top-left" />
 </picture>
</a>
