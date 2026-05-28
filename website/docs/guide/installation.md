# Installation

## Requirements

ApexSU is designed exclusively for supported Android GKI devices.

- **Supported Android GKI device**: Your kernel version must contain the keyword "android". If it does not, your device is treated as non-GKI and is unsupported.
- **Boot image**: You must have a boot image matching your current firmware/build.
- **ApexSU Manager**: To manage your installation and modules.
- **Internet connection**: Required if using Repository LKM.
- **Local .ko file**: Only required if using Local LKM on a supported GKI device.

## Unsupported devices

Non-GKI devices are **not supported**. 
- Installation must not continue on non-GKI devices.
- Local LKM is not a workaround for non-GKI compatibility.

## GKI check

To verify if your device is supported:
1. Open ApexSU Manager.
2. Check your kernel version on the home screen.
3. If the kernel contains "android" (e.g., `5.10.209-android12-9-00016-g7c6bbcca33e1`), it is treated as GKI and supported.
4. If it does not contain "android", the device is treated as unsupported and installation will be blocked.

## LKM options

ApexSU supports two installation methods for the loadable kernel module (LKM) on supported GKI devices:

- **Repository LKM**: The recommended and default option for GKI devices. It downloads the required module automatically based on your kernel version.
- **Local LKM**: An advanced, manual option for GKI devices only. It requires you to provide a compatible local `.ko` file.

*Note: For non-GKI devices, no LKM option should be used. The installation is unsupported.*

## Install button behavior

When you attempt to install ApexSU via the Manager:
- **On GKI**: You must provide a boot.img matching your firmware and select a valid Repository or Local LKM. The install button will proceed with patching.
- **On non-GKI**: The install button is disabled and installation is blocked.

## Backup stock boot.img

Before patching, it's essential that you back up your stock boot.img. Rooting or modifying boot images can bootloop devices. If you encounter a bootloop, you can restore the system by flashing the stock boot image via fastboot. Please back up all important data on your device.

## Security patch level

Newer Android devices may have anti-rollback mechanisms that prevent flashing a boot image with an old security patch level. Always use the boot image matching your exact firmware build to avoid bootloops.

## Manual patching via Manager

1. Open ApexSU Manager.
2. Ensure the manager reports your device as a supported GKI device.
3. Click the install button and choose to patch your stock boot.img.
4. Flash the patched boot image to your device using fastboot:
   ```sh
   fastboot flash boot patched_boot.img
   fastboot reboot
   ```

::: warning
Never attempt to force install or flash random modules on unsupported devices. Non-GKI devices are strictly unsupported.
:::
