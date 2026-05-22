# ApexSU kernel ABI checker

This standalone Rust crate mirrors the C ABI for selected profile and ioctl
payload structs from `kernel/app_profile.h` and `kernel/supercalls.h`.

It is intentionally not connected to kernel Kbuild or Android/GKI module
builds. Run it as host-side tooling:

```sh
cd kernel/rust/abi_checker
cargo test
```

The first migration wave stays away from kernel hot paths and only validates
layout constants, struct sizes, alignments, and app-profile input rules.
