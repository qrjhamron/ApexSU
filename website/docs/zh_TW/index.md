---
layout: home
title: ApexSU

hero:
  name: ApexSU
  text: 面向受支援 Android GKI 裝置的現代 root 方案
  tagline: "ApexSU 為受支援的 Android GKI 核心提供聚焦的 root 流程，包含 boot image 安裝、LKM 支援與現代化管理器。"
  image:
    src: /logo.svg
    alt: ApexSU
  actions:
    - theme: brand
      text: 下載
      link: https://github.com/qrjhamron/ApexSU/releases
    - theme: alt
      text: GitHub
      link: https://github.com/qrjhamron/ApexSU
    - theme: alt
      text: 文件
      link: /zh_TW/guide/installation

features:
  - title: 僅支援 GKI
    details: ApexSU 僅支援受支援的 Android GKI 裝置。non-GKI 裝置不受支援，安裝必須被阻擋。
  - title: boot image 流程
    details: 安裝必須使用與目前韌體/版本完全相符的 boot.img。
  - title: LKM 支援
    details: 對受支援的 GKI 裝置，Repository LKM 是預設推薦方案。Local LKM 只適用於進階手動場景。
  - title: 清楚安全警告
    details: ApexSU 會明確提示 bootloop 風險，並阻擋不受支援的安裝路徑。
---

<div class="apex-warning">
  <h2>僅支援 GKI</h2>
  <p>non-GKI 裝置不受支援。ApexSU 在 non-GKI 裝置上的安裝必須始終被阻擋。</p>
</div>

<div class="apex-quick-links">
  <h2>快速連結</h2>
  <ul>
    <li><a href="https://github.com/qrjhamron/ApexSU/releases">下載 ApexSU</a></li>
    <li><a href="https://github.com/qrjhamron/ApexSU">GitHub 倉庫</a></li>
    <li><a href="/zh_TW/guide/installation">安裝指南</a></li>
    <li><a href="https://t.me/smoothlady">@smoothlady</a></li>
  </ul>
</div>

<div class="apex-safety">
  <h2>安全警告</h2>
  <p>修改 boot image 可能導致裝置 bootloop。請使用與目前韌體/版本完全相符的 boot.img，並先備份重要資料。</p>
</div>
