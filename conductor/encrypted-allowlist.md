# Plan: Hardware-Backed Encrypted Allowlist (Rust)

## Objective
Implement an encrypted allowlist for ApexSU to enhance stealth and security. The `.allowlist` file will no longer be stored in plain text by the kernel. Instead, the kernel will delegate the persistence to `ksud` (user-space syncing), which will encrypt and decrypt the profiles.

## Key Files & Context
- `kernel/allowlist.c`: Currently handles reading and writing the `.allowlist` file directly.
- `kernel/ksud.c`: Triggers allowlist loading during `post-fs-data`.
- `userspace/ksud/src/cli.rs`: Command-line interface for `ksud`. Needs new commands for `profile load` and `profile sync`.
- `userspace/ksud/src/profile.rs`: Will handle reading from the kernel, encrypting/decrypting, and writing to `.allowlist.enc`.
- `userspace/ksud/src/init_event.rs`: Triggers allowlist loading during the boot process.

## Implementation Steps

### 1. Kernel Modification (`kernel/allowlist.c` & `kernel/ksud.c`)
- **Remove File I/O:** Delete `do_persistent_allow_list` and `ksu_load_allow_list` functions.
- **Usermodehelper for Sync:** Modify `ksu_persistent_allow_list` to use `call_usermodehelper` to execute `/data/adb/ksud profile sync` asynchronously whenever the allowlist is updated (e.g. from Manager app).
- **Remove Boot-time Load:** Remove the call to `ksu_load_allow_list()` inside `on_post_fs_data()`. `ksud` will handle pushing the list to the kernel.

### 2. Rust Setup: Key Derivation & Crypto
- **Dependencies:** Add `aes-gcm` or `chacha20poly1305` to `userspace/ksud/Cargo.toml` for encryption.
- **Key Derivation:** Implement a function in `ksud` to derive an encryption key using device-specific properties (e.g., `ro.boot.serialno`, `/data/vendor/mac_address` or a generated key file with `0600` root permissions).

### 3. Rust Modification (`userspace/ksud/src/profile.rs`)
- **Command `profile sync`:** 
  - Call kernel `KSU_IOCTL_NEW_GET_ALLOW_LIST` to retrieve all `app_profile` structs.
  - Serialize the structs to binary.
  - Encrypt the binary blob using the derived key.
  - Write to `/data/adb/ksu/.allowlist.enc` atomically.
- **Command `profile load`:**
  - Read `/data/adb/ksu/.allowlist.enc`.
  - Decrypt the file.
  - Deserialize the `app_profile` structs.
  - Call `KSU_IOCTL_SET_APP_PROFILE` for each profile to load them into the kernel.

### 4. Integration (`userspace/ksud/src/cli.rs` & `init_event.rs`)
- Add `sync` and `load` subcommands to the `Profile` enum in `cli.rs`.
- In `init_event.rs`, inside `on_post_data_fs()`, call `profile::load_allowlist()` before `assets::ensure_binaries()` or early in the boot sequence to ensure profiles are ready.

## Verification & Testing
- Compile kernel and `ksud`.
- Flash to device or emulator.
- Use Manager App to grant Root to an app.
- Verify that `/data/adb/ksu/.allowlist` no longer exists, and `/data/adb/ksu/.allowlist.enc` is created and contains encrypted ciphertext.
- Reboot device and ensure the app still retains root access (verifying that `ksud profile load` works).

## Rollback / Alternatives Considered
- **Kernel-space Crypto:** Considered using the kernel's Crypto API, but `ksud` user-space syncing is more flexible and aligns better with modern Android security practices where user-space handles crypto keys.