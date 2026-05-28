---
layout: home
title: ApexSU

hero:
  name: ApexSU
  text: Giải pháp root hiện đại cho thiết bị Android GKI được hỗ trợ
  tagline: "ApexSU cung cấp quy trình root tập trung cho kernel Android GKI được hỗ trợ, với cài đặt qua boot image, hỗ trợ LKM và trình quản lý Android hiện đại."
  image:
    src: /apexsu_logo.svg
    alt: ApexSU
  actions:
    - theme: brand
      text: Tải xuống
      link: https://github.com/qrjhamron/ApexSU/releases
    - theme: alt
      text: GitHub
      link: https://github.com/qrjhamron/ApexSU
    - theme: alt
      text: Hướng dẫn
      link: /vi_VN/guide/installation

features:
  - title: Chỉ hỗ trợ GKI
    details: ApexSU chỉ hỗ trợ thiết bị Android GKI được hỗ trợ. Thiết bị non-GKI không được hỗ trợ.
  - title: Quy trình boot image
    details: Cài đặt yêu cầu boot.img khớp chính xác firmware/build hiện tại.
  - title: Hỗ trợ LKM
    details: Repository LKM là lựa chọn mặc định cho thiết bị GKI được hỗ trợ. Local LKM là tùy chọn nâng cao chỉ dành cho GKI.
  - title: Cảnh báo an toàn rõ ràng
    details: ApexSU nêu rõ rủi ro bootloop và chặn các đường cài đặt không được hỗ trợ.
---

<div class="apex-warning">
  <h2>Chỉ hỗ trợ GKI</h2>
  <p>Thiết bị non-GKI không được hỗ trợ. Cài đặt phải luôn bị chặn trên thiết bị non-GKI.</p>
</div>

<div class="apex-quick-links">
  <h2>Liên kết nhanh</h2>
  <ul>
    <li><a href="https://github.com/qrjhamron/ApexSU/releases">Tải ApexSU</a></li>
    <li><a href="https://github.com/qrjhamron/ApexSU">Kho GitHub</a></li>
    <li><a href="/vi_VN/guide/installation">Hướng dẫn cài đặt</a></li>
    <li><a href="https://t.me/smoothlady">@smoothlady</a></li>
  </ul>
</div>

<div class="apex-safety">
  <h2>Cảnh báo an toàn</h2>
  <p>Sửa đổi boot image có thể gây bootloop. Luôn dùng boot.img khớp đúng firmware/build và sao lưu dữ liệu quan trọng trước khi cài đặt.</p>
</div>
