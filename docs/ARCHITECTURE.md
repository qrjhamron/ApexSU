# ApexSU Architecture

## System Overview

ApexSU (KernelSU fork) is a kernel-based root solution for Android.
It consists of three layers that communicate via IOCTLs on an anonymous inode.

```
┌───────────────────────────────────────────────────────────────┐
│                     ANDROID USERSPACE                        │
│                                                               │
│  ┌─────────────────────┐     ┌─────────────────────────────┐ │
│  │    Manager App       │     │     ksud Daemon              │ │
│  │   (Kotlin/Compose)   │     │       (Rust)                 │ │
│  │                      │     │                              │ │
│  │  Screens ──► VMs ──► JNI ──► IOCTL ◄── CLI               │ │
│  │                      │     │                              │ │
│  │  - SuperUser list    │     │  - Module install/update     │ │
│  │  - Module manager    │     │  - SEPolicy patching         │ │
│  │  - App profiles      │     │  - Boot image patching       │ │
│  │  - Settings          │     │  - Feature management        │ │
│  └─────────────────────┘     └──────────────┬──────────────┘ │
│                                              │                │
└──────────────────────────────────────────────┼────────────────┘
                                               │ IOCTL (anon inode FD)
┌──────────────────────────────────────────────┼────────────────┐
│                     KERNEL SPACE              │                │
│                                              ▼                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                    KSU Driver                             │ │
│  │                                                           │ │
│  │  supercalls.c ── 20 IOCTL handlers                        │ │
│  │       │                                                   │ │
│  │       ├── allowlist.c ── UID root grant bitmap            │ │
│  │       ├── app_profile.c ── per-app policy                 │ │
│  │       ├── feature.c ── feature flags                      │ │
│  │       └── selinux/ ── policy injection                    │ │
│  │                                                           │ │
│  │  Syscall Hooks:                                           │ │
│  │       ├── syscall_hook_manager.c ── hook registration     │ │
│  │       ├── sucompat.c ── su binary exec interception       │ │
│  │       ├── setuid_hook.c ── credential escalation          │ │
│  │       ├── ksud.c ── ksud communication                    │ │
│  │       └── throne_tracker.c ── app lifecycle tracking      │ │
│  │                                                           │ │
│  │  Filesystem:                                              │ │
│  │       ├── file_wrapper.c ── kernel file operations        │ │
│  │       ├── kernel_umount.c ── module unmounting            │ │
│  │       └── su_mount_ns.c ── mount namespace isolation      │ │
│  └──────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────┘
```

## Communication Channel

The kernel module and userspace communicate through a custom anonymous inode:

1. Kernel module creates an anonymous inode file descriptor (no `/proc`, `/sys`, or `/dev` entry)
2. FD is delivered to trusted processes via a reboot syscall kprobe:
   - Magic `0xDEADBEEF` → install driver FD
   - Magic `0xCAFEBABE` → scan for driver FD
3. All commands are sent as IOCTLs with magic byte `'K'` (0x4B)

This design minimizes exposed interfaces and keeps the control channel scoped to
trusted runtime paths.

## Root Grant Flow

```
App calls /system/bin/su
         │
         ▼
┌─────────────────────────┐
│ execve() syscall hook   │  (sucompat.c)
│ Intercepts su execution │
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│ Check UID in allowlist  │  (allowlist.c)
│ Bitmap + linked-list    │
└────────────┬────────────┘
             │
     ┌───────┴───────┐
     │               │
  Allowed         Denied
     │               │
     ▼               ▼
┌──────────┐   ┌──────────┐
│ Redirect │   │  Normal  │
│ to ksud  │   │  exec    │
│ su shell │   │ (denied) │
└────┬─────┘   └──────────┘
     │
     ▼
┌─────────────────────────┐
│ IOCTL: GRANT_ROOT       │  (supercalls.c)
│ Credential escalation   │
│ Disable seccomp         │
│ Set SELinux domain       │
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│ Root shell spawned      │  (su.rs)
│ with elevated creds     │
└─────────────────────────┘
```

## Module Installation Flow

```
User selects ZIP in Manager
         │
         ▼
┌─────────────────────────┐
│ module_validator.rs     │  Validate ZIP structure
│ - Path traversal check  │  - module.prop fields
│ - Size limits           │  - ID format
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│ module.rs install()     │  Extract to /data/adb/modules/<id>/
│ - Parse module.prop     │
│ - Run install.sh        │
│ - Set permissions       │
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│ init_event.rs           │  On next boot:
│ - post-fs-data stage    │  - Mount overlays
│ - service stage         │  - Run service.sh
│ - boot-completed stage  │  - Run boot-completed.sh
└─────────────────────────┘
```

## IOCTL Command Table

| # | Command | Direction | Purpose |
|---|---------|-----------|---------|
| 1 | `GRANT_ROOT` | W | Grant root to calling process |
| 2 | `GET_INFO` | R | Get kernel version and feature flags |
| 3 | `REPORT_EVENT` | W | Report boot stage events |
| 4 | `SET_SEPOLICY` | W | Apply SELinux policy rules |
| 5 | `CHECK_SAFEMODE` | R | Check if safe mode is active |
| 6 | `GET_ALLOW_LIST` | R | Get allowed UIDs (v2) |
| 7 | `GET_DENY_LIST` | R | Get denied UIDs (v2) |
| 8 | `UID_GRANTED_ROOT` | R | Check if specific UID has root |
| 9 | `UID_SHOULD_UMOUNT` | R | Check if UID needs module unmount |
| 10 | `GET_MANAGER_APPID` | R | Get manager application ID |
| 11 | `GET_APP_PROFILE` | R | Get per-app profile settings |
| 12 | `SET_APP_PROFILE` | W | Set per-app profile settings |
| 13 | `GET_FEATURE` | R | Get feature flag state |
| 14 | `SET_FEATURE` | W | Set feature flag state |
| 15 | `GET_WRAPPER_FD` | R | Get TTY wrapper for su |
| 16 | `MANAGE_MARK` | RW | Process mark management |
| 17 | `NUKE_EXT4_SYSFS` | W | Remove ext4 sysfs exposure |
| 18 | `ADD_TRY_UMOUNT` | W | Manage unmount list |

## ABI Compatibility Expectations

- Userspace and kernel IOCTL structs must remain byte-identical.
- `KSU_IOCTL_ABI_VERSION` in kernel headers tracks contract revisions.
- `KSU_IOCTL_GET_ALLOW_LIST` / `KSU_IOCTL_GET_DENY_LIST` are deprecated and kept only for compatibility.
- New userspace callers must prefer `KSU_IOCTL_NEW_GET_ALLOW_LIST` / `KSU_IOCTL_NEW_GET_DENY_LIST`.
- Any struct layout change requires:
  1. versioned migration handling in userspace
  2. explicit compatibility fallback path
  3. tests validating struct sizes/offsets

## Component Responsibilities

### Kernel Module (`kernel/`) — C

- Syscall hooking via kprobes (execve, setresuid, reboot)
- UID allowlist management (bitmap storage)
- Credential escalation (root grant)
- SELinux policy injection at runtime
- Module filesystem overlay management
- Anonymous inode IPC channel

### Userspace Daemon (`userspace/ksud/`) — Rust

- Module installation, update, and removal
- SELinux policy rule parsing (nom-based)
- Boot image patching (via magiskboot)
- CLI interface for all operations
- Feature flag management
- App profile persistence
- Module validation and diagnostics

### Manager App (`manager/`) — Kotlin

- Superuser access management UI
- Module browser and installer UI
- App profile configuration UI
- Settings and about screens
- WebUI hosting for module configuration

## Security Model

1. **Trust anchor**: Kernel module loaded at boot
2. **Identity verification**: Manager app verified by APK signature hash
3. **Access control**: UID-based allowlist in kernel memory
4. **Persistence**: Allowlist saved to `/data/adb/ksu/.allowlist` with atomic writes
5. **Isolation**: Per-app mount namespace separation
6. **Policy**: SELinux domain transition for root processes
7. **Auditability**: explicit validation and diagnostics for privileged flows

## Directory Layout

```
/data/adb/
├── ksu/
│   ├── .allowlist         # UID allowlist (binary format)
│   └── modules.img        # Module image
├── ksud                   # Userspace daemon binary
└── modules/               # Installed modules
    └── <module-id>/
        ├── module.prop    # Module metadata
        ├── system/        # Overlay files
        ├── post-fs-data.sh
        ├── service.sh
        └── uninstall.sh
```
