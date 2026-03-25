# Frequently Asked Questions

Common questions about ApexSU answered.

---

## General

### What is ApexSU?

ApexSU is an advanced Android root solution based on KernelSU. It operates at the kernel level, hooking syscalls to provide root access. The key difference from KernelSU is the **Rust-first userspace** — the daemon (`ksud`) and JNI bridges are written in Rust for better memory safety and security.

### Is ApexSU a replacement for Magisk?

Not exactly. ApexSU is kernel-based (like KernelSU), while Magisk patches the boot image userspace. Both have pros and cons:

| | Magisk | ApexSU |
|:--|:------:|:------:|
| Root method | Userspace | Kernel |
| Detection resistance | Moderate | High |
| Module support | Yes | Yes (compatible) |
| Requires custom kernel | No | Yes |

### Is ApexSU compatible with Magisk modules?

**Yes!** ApexSU supports Magisk modules through systemless mounting. Most modules work out of the box.

### What Android versions are supported?

- **Minimum**: Android 12 (API 31)
- **Kernel**: 5.10 or higher (GKI 2.0)

---

## Installation

### My device bootlooped after installing. What do I do?

1. Boot to fastboot mode (usually `Power + Volume Down`)
2. Flash your stock boot image:
   ```bash
   fastboot flash boot stock_boot.img
   ```
3. Reboot and try again with correct boot image

### Manager says "Not Installed" but I flashed the kernel

Common causes:
- Wrong boot slot (A/B devices) — try flashing both slots
- Incompatible kernel version
- GKI not supported on your device

Check kernel logs:
```bash
adb shell dmesg | grep -i apexsu
```

### Can I use ApexSU with a locked bootloader?

**No.** An unlocked bootloader is required to flash custom boot images.

---

## Root Access

### App is not getting root access

1. Open ApexSU Manager
2. Go to **Superuser** tab
3. Find the app and ensure it's set to **Allow**
4. If app doesn't appear, it may not be requesting root properly

### How do I grant root to ADB shell?

```bash
adb shell
su
```

The `su` command is handled by ApexSU's kernel module.

### Root works but SafetyNet/Play Integrity fails

ApexSU has stealth features, but you may need additional modules:
- Use a Play Integrity Fix module
- Ensure no root-detection apps are running during attestation

---

## Modules

### Where are modules stored?

Modules are stored in `/data/adb/modules/`

### Module installed but not working

1. Check if module requires reboot
2. Verify module compatibility with your Android version
3. Check module logs in Manager
4. Some modules need Zygisk — ApexSU supports Zygisk modules

### How do I remove a module that causes bootloop?

Boot to recovery and delete the module folder:
```bash
# In recovery terminal or via ADB sideload
rm -rf /data/adb/modules/<module_name>
```

Or rename the `disable` flag:
```bash
touch /data/adb/modules/<module_name>/disable
```

---

## Security

### How does ApexSU avoid detection?

ApexSU uses several stealth techniques:
- Anonymous inode named `[io_uring]` instead of `[ksu_driver]`
- No entries in `/proc`, `/sys`, or `/dev`
- Kernel-level syscall hooking (invisible to userspace scanners)

See [Security & Stealth](Security-and-Stealth) for details.

### Is it safe to use ApexSU?

ApexSU is open source and auditable. The Rust codebase reduces memory vulnerabilities. However, **any root solution carries risks** — only install trusted modules and apps.

### How do I report a security vulnerability?

**Do NOT open a public issue.** Instead:
1. Go to [Security Advisories](https://github.com/qrjhamron/ApexSU/security/advisories)
2. Click **"New draft security advisory"**
3. Describe the vulnerability privately

---

## Development

### How do I build ApexSU from source?

See our build documentation:
- **ksud daemon**: `cd userspace/ksud && cargo ndk -t arm64-v8a build --release`
- **Manager APK**: `cd manager && ./gradlew assembleRelease`

Requirements: Rust 1.82+, Android NDK r29, JDK 21

### How do I contribute?

1. Fork the repository
2. Make your changes
3. Ensure `cargo clippy` passes with zero warnings
4. Submit a Pull Request

See [CONTRIBUTING.md](https://github.com/qrjhamron/ApexSU/blob/main/CONTRIBUTING.md)

---

## Troubleshooting

### How do I collect logs for bug reports?

```bash
# Kernel logs
adb shell dmesg > kernel.log

# ApexSU daemon logs
adb shell su -c "cat /data/adb/apexsu/log/*" > ksud.log

# Manager bugreport
# Use "Generate Bugreport" in Manager settings
```

### ksud diagnose shows errors

Run diagnostics:
```bash
adb shell su -c "ksud diagnose"
```

Common fixes:
- **SELinux denials**: Ensure proper context on ApexSU files
- **Module errors**: Disable problematic modules
- **Daemon not running**: Reboot or manually start ksud

---

Still have questions? [Open a discussion](https://github.com/qrjhamron/ApexSU/discussions) or [search existing issues](https://github.com/qrjhamron/ApexSU/issues).
