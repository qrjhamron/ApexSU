---
layout: home
title: Home

hero:
  name: ApexSU
  text: Modern root solution for supported Android GKI devices
  tagline: "ApexSU is intended for supported Android GKI kernels/devices."
  image:
    src: /logo.png
    alt: ApexSU
  actions:
    - theme: brand
      text: Get started
      link: /guide/what-is-kernelsu
    - theme: alt
      text: Download
      link: https://github.com/qrjhamron/ApexSU/releases
    - theme: alt
      text: View on GitHub
      link: https://github.com/qrjhamron/ApexSU

features:
  - title: GKI-only support
    details: ApexSU does not support non-GKI devices. If your kernel version does not contain the android keyword, installation should be blocked.
  - title: LKM support
    details: Install using Repository LKM (recommended/default for GKI) or Local LKM (advanced/manual option for GKI devices only).
  - title: Boot image workflow
    details: Requires a boot.img matching your firmware/build. Rooting/modifying boot images can bootloop devices; back up important data.
  - title: Modern Android manager
    details: Open ApexSU Manager to check kernel version. If it contains "android", it is treated as GKI.
---
