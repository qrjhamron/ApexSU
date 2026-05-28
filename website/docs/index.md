---
layout: home
title: Home

hero:
  name: ApexSU
  text: Modern root solution for supported Android GKI devices
  tagline: "ApexSU provides a focused root workflow for supported Android GKI kernels, with boot image installation, LKM support, and a modern Android manager."
  image:
    src: /logo.svg
    alt: ApexSU
  actions:
    - theme: brand
      text: Download
      link: https://github.com/qrjhamron/ApexSU/releases
    - theme: alt
      text: GitHub
      link: https://github.com/qrjhamron/ApexSU
    - theme: alt
      text: Guide
      link: /guide/installation

features:
  - title: GKI-only support
    details: Only supported Android GKI devices can install ApexSU. Non-GKI devices are blocked.
  - title: Boot image workflow
    details: Installation requires a boot.img that matches your exact firmware/build.
  - title: LKM support
    details: Repository LKM is default for supported GKI devices. Local LKM is advanced and GKI-only.
  - title: Modern Android manager
    details: ApexSU Manager provides install checks, module control, and clear safety states.
  - title: Open-source development
    details: Public source, issue tracking, and release artifacts are maintained on GitHub.
  - title: Clear safety warnings
    details: ApexSU highlights compatibility risk and blocks unsupported installation paths.
---

<div class="apex-warning">
  <h2>GKI-only support</h2>
  <p>ApexSU only supports supported Android GKI devices. Non-GKI devices are not supported and installation must be blocked.</p>
</div>

<div class="apex-quick-links">
  <h2>Quick links</h2>
  <ul>
    <li><a href="https://github.com/qrjhamron/ApexSU/releases">Download ApexSU</a></li>
    <li><a href="https://github.com/qrjhamron/ApexSU">GitHub repository</a></li>
    <li><a href="/guide/installation">Installation guide</a></li>
    <li><a href="https://t.me/smoothlady">@smoothlady</a></li>
  </ul>
</div>

<div class="apex-safety">
  <h2>Safety warning</h2>
  <p>Modifying boot images can bootloop devices. Always use a boot.img matching the exact firmware/build and back up important data before installation.</p>
</div>
