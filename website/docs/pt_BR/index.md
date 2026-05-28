---
layout: home
title: ApexSU

hero:
  name: ApexSU
  text: Solução root moderna para dispositivos Android GKI suportados
  tagline: "O ApexSU oferece um fluxo de root focado para kernels Android GKI suportados, com instalação por boot image, suporte a LKM e um gerenciador Android moderno."
  image:
    src: /apexsu_logo.svg
    alt: ApexSU
  actions:
    - theme: brand
      text: Download
      link: https://github.com/qrjhamron/ApexSU/releases
    - theme: alt
      text: GitHub
      link: https://github.com/qrjhamron/ApexSU
    - theme: alt
      text: Guia
      link: /pt_BR/guide/installation

features:
  - title: Suporte apenas a GKI
    details: O ApexSU suporta somente dispositivos Android GKI suportados. Dispositivos non-GKI não são suportados.
  - title: Fluxo de boot image
    details: A instalação exige boot.img correspondente ao firmware/build exato do dispositivo.
  - title: Suporte a LKM
    details: Repository LKM é o padrão recomendado para GKI suportado. Local LKM é opção avançada e apenas para GKI suportado.
  - title: Avisos de segurança claros
    details: O ApexSU destaca risco de bootloop e bloqueia caminhos de instalação não suportados.
---

<div class="apex-warning">
  <h2>Suporte apenas a GKI</h2>
  <p>Dispositivos non-GKI não são suportados. A instalação deve ser bloqueada nesses dispositivos.</p>
</div>

<div class="apex-quick-links">
  <h2>Links rápidos</h2>
  <ul>
    <li><a href="https://github.com/qrjhamron/ApexSU/releases">Baixar ApexSU</a></li>
    <li><a href="https://github.com/qrjhamron/ApexSU">Repositório GitHub</a></li>
    <li><a href="/pt_BR/guide/installation">Guia de instalação</a></li>
    <li><a href="https://t.me/smoothlady">@smoothlady</a></li>
  </ul>
</div>

<div class="apex-safety">
  <h2>Aviso de segurança</h2>
  <p>Modificar boot image pode causar bootloop. Use sempre boot.img do mesmo firmware/build e faça backup dos dados importantes.</p>
</div>
