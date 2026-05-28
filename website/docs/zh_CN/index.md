---
layout: home
title: ApexSU

hero:
  name: ApexSU
  text: 面向受支持 Android GKI 设备的现代 root 方案
  tagline: "ApexSU 为受支持的 Android GKI 内核提供聚焦的 root 流程，包含 boot image 安装、LKM 支持和现代化管理器。"
  image:
    src: /logo.svg
    alt: ApexSU
  actions:
    - theme: brand
      text: 下载
      link: https://github.com/qrjhamron/ApexSU/releases
    - theme: alt
      text: GitHub
      link: https://github.com/qrjhamron/ApexSU
    - theme: alt
      text: 文档
      link: /zh_CN/guide/installation

features:
  - title: 仅支持 GKI
    details: ApexSU 仅支持受支持的 Android GKI 设备。non-GKI 设备不受支持，安装必须被阻止。
  - title: boot image 流程
    details: 安装必须使用与当前固件/版本完全匹配的 boot.img。
  - title: LKM 支持
    details: 对受支持的 GKI 设备，Repository LKM 是默认推荐方案。Local LKM 仅为高级手动选项。
  - title: 清晰安全提示
    details: ApexSU 明确提示 bootloop 风险，并阻止不受支持的安装路径。
---

<div class="apex-warning">
  <h2>仅支持 GKI</h2>
  <p>non-GKI 设备不受支持。ApexSU 在 non-GKI 设备上的安装必须始终被阻止。</p>
</div>

<div class="apex-quick-links">
  <h2>快速链接</h2>
  <ul>
    <li><a href="https://github.com/qrjhamron/ApexSU/releases">下载 ApexSU</a></li>
    <li><a href="https://github.com/qrjhamron/ApexSU">GitHub 仓库</a></li>
    <li><a href="/zh_CN/guide/installation">安装指南</a></li>
    <li><a href="https://t.me/smoothlady">@smoothlady</a></li>
  </ul>
</div>

<div class="apex-safety">
  <h2>安全警告</h2>
  <p>修改 boot image 可能导致设备 bootloop。请使用与当前固件/版本完全匹配的 boot.img，并提前备份重要数据。</p>
</div>
