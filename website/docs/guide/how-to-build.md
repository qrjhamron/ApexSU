# Build guide

This page covers building ApexSU for supported Android GKI workflows.

## Scope

- ApexSU supports only supported GKI devices.
- Non-GKI installation is unsupported and blocked.

## Build references

- Android kernel build docs: https://source.android.com/docs/setup/build/building-kernels
- ApexSU repository: https://github.com/qrjhamron/ApexSU

## Typical flow

1. Sync a matching Android GKI kernel source.
2. Build the kernel and required modules.
3. Use a matching boot image for your exact firmware/build.
4. Install through ApexSU Manager on a supported GKI device.
