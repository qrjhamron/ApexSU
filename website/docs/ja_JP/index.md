---
layout: home
title: ApexSU

hero:
  name: ApexSU
  text: サポート対象の Android GKI デバイス向けモダン root ソリューション
  tagline: "ApexSU は、サポート対象の Android GKI カーネル向けに、boot image インストール、LKM サポート、モダンなマネージャーを提供します。"
  image:
    src: /apexsu_logo.svg
    alt: ApexSU
  actions:
    - theme: brand
      text: ダウンロード
      link: https://github.com/qrjhamron/ApexSU/releases
    - theme: alt
      text: GitHub
      link: https://github.com/qrjhamron/ApexSU
    - theme: alt
      text: ガイド
      link: /ja_JP/guide/installation

features:
  - title: GKI 専用サポート
    details: ApexSU はサポート対象の Android GKI デバイスのみ対応です。非 GKI デバイスは非対応で、インストールはブロックされます。
  - title: Boot image ワークフロー
    details: インストールには、現在のファームウェアと一致する boot.img が必要です。
  - title: LKM サポート
    details: サポート対象 GKI では Repository LKM が推奨です。Local LKM は上級者向けで GKI 専用です。
  - title: 安全性の明確化
    details: boot image の変更によるリスクを明示し、非対応インストール経路をブロックします。
---

<div class="apex-warning">
  <h2>GKI 専用サポート</h2>
  <p>ApexSU はサポート対象の Android GKI デバイスのみ対応です。非 GKI デバイスは非対応で、インストールは常にブロックされます。</p>
</div>

<div class="apex-quick-links">
  <h2>クイックリンク</h2>
  <ul>
    <li><a href="https://github.com/qrjhamron/ApexSU/releases">ApexSU をダウンロード</a></li>
    <li><a href="https://github.com/qrjhamron/ApexSU">GitHub リポジトリ</a></li>
    <li><a href="/ja_JP/guide/installation">インストールガイド</a></li>
    <li><a href="https://t.me/smoothlady">@smoothlady</a></li>
  </ul>
</div>

<div class="apex-safety">
  <h2>安全に関する注意</h2>
  <p>boot image を変更すると端末が bootloop する可能性があります。必ず同一ビルドの boot.img を使用し、重要なデータを事前にバックアップしてください。</p>
</div>
