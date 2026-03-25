# Installation Guide

This guide walks you through installing ApexSU on your Android device.

---

## Prerequisites

Before you begin, make sure you have:

- **Android 12+** device
- **Kernel 5.10+** (GKI 2.0 compatible)
- **Unlocked bootloader**
- **ADB & Fastboot** installed on your PC
- **USB debugging** enabled on your device

---

## Method 1: Boot Image Patching (Recommended)

This method patches your existing boot image with the ApexSU kernel module.

### Step 1: Download Required Files

1. Download the latest **ApexSU Manager APK** from [Releases](https://github.com/qrjhamron/ApexSU/releases)
2. Download your device's **stock boot.img** (from firmware or use payload dumper)

### Step 2: Install Manager & Patch

1. Install the ApexSU Manager APK on your device
2. Open the Manager app
3. Tap **"Install"** and select **"Select and Patch a File"**
4. Choose your `boot.img` file
5. Wait for patching to complete — output will be `apexsu_patched_boot.img`

### Step 3: Flash Patched Boot Image

```bash
# Reboot to bootloader
adb reboot bootloader

# Flash the patched image
fastboot flash boot apexsu_patched_boot.img

# Reboot
fastboot reboot
```

### Step 4: Verify Installation

1. Open ApexSU Manager
2. You should see **"Working"** status with your kernel version
3. Done! You now have root access

---

## Method 2: GKI Kernel Replacement

For devices with Generic Kernel Image (GKI) support.

### Step 1: Download Pre-built Kernel

1. Go to [Releases](https://github.com/qrjhamron/ApexSU/releases)
2. Download the kernel zip matching your kernel version (e.g., `ApexSU_5.10.xxx_GKI.zip`)

### Step 2: Flash via Custom Recovery

1. Reboot to recovery mode
2. Select **"Install"** or **"Apply update"**
3. Choose the downloaded zip file
4. Reboot to system

### Step 3: Install Manager

Install the ApexSU Manager APK to manage root permissions.

---

## Method 3: Kernel Source Integration

For custom kernel developers who want to integrate ApexSU directly.

### Step 1: Clone ApexSU

```bash
git clone https://github.com/qrjhamron/ApexSU.git
cd ApexSU
```

### Step 2: Setup Kernel Source

```bash
# Add ApexSU to your kernel source
cd /path/to/your/kernel
curl -LSs "https://raw.githubusercontent.com/qrjhamron/ApexSU/main/kernel/setup.sh" | bash -
```

### Step 3: Configure & Build

```bash
# Enable ApexSU in kernel config
make menuconfig
# Navigate to: Kernel hacking -> ApexSU Support -> Enable

# Build kernel
make -j$(nproc)
```

---

## Post-Installation

### Grant Root Access

1. Open ApexSU Manager
2. Go to **Superuser** tab
3. Apps requesting root will appear here
4. Tap to **Allow** or **Deny**

### Install Modules

1. Go to **Module** tab
2. Tap the **+** button
3. Select a module zip file
4. Reboot to activate

---

## Troubleshooting

| Issue | Solution |
|:------|:---------|
| Manager shows "Not Installed" | Re-flash boot image, ensure correct slot |
| Bootloop after flash | Flash stock boot.img via fastboot |
| Apps can't get root | Check Superuser tab, ensure app is allowed |
| Module not working | Check module compatibility, see logs in Manager |

For more help, see the [FAQ](FAQ) or [open an issue](https://github.com/qrjhamron/ApexSU/issues).

---

## Updating ApexSU

1. Download new Manager APK and kernel from Releases
2. Install new Manager (update in place)
3. Use Manager to patch new boot image
4. Flash updated boot image

**Note:** Your module configurations are preserved during updates.
