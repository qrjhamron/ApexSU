---
layout: home
title: ApexSU

hero:
  name: ApexSU
  text: Современное root-решение для поддерживаемых Android GKI устройств
  tagline: "ApexSU предоставляет сфокусированный root-процесс для поддерживаемых Android GKI ядер: установка через boot image, поддержка LKM и современный менеджер."
  image:
    src: /apexsu_logo.svg
    alt: ApexSU
  actions:
    - theme: brand
      text: Скачать
      link: https://github.com/qrjhamron/ApexSU/releases
    - theme: alt
      text: GitHub
      link: https://github.com/qrjhamron/ApexSU
    - theme: alt
      text: Руководство
      link: /ru_RU/guide/installation

features:
  - title: Поддержка только GKI
    details: ApexSU поддерживает только поддерживаемые Android GKI устройства. Устройства non-GKI не поддерживаются.
  - title: Процесс через boot image
    details: Для установки требуется boot.img, точно соответствующий текущей прошивке/сборке.
  - title: Поддержка LKM
    details: Repository LKM — рекомендуемый вариант для поддерживаемых GKI. Local LKM — продвинутый вариант только для GKI.
  - title: Четкие предупреждения
    details: ApexSU показывает риск bootloop и блокирует неподдерживаемые пути установки.
---

<div class="apex-warning">
  <h2>Поддержка только GKI</h2>
  <p>Устройства non-GKI не поддерживаются. Установка на таких устройствах должна быть заблокирована.</p>
</div>

<div class="apex-quick-links">
  <h2>Быстрые ссылки</h2>
  <ul>
    <li><a href="https://github.com/qrjhamron/ApexSU/releases">Скачать ApexSU</a></li>
    <li><a href="https://github.com/qrjhamron/ApexSU">GitHub репозиторий</a></li>
    <li><a href="/ru_RU/guide/installation">Руководство по установке</a></li>
    <li><a href="https://t.me/smoothlady">@smoothlady</a></li>
  </ul>
</div>

<div class="apex-safety">
  <h2>Предупреждение по безопасности</h2>
  <p>Изменение boot image может привести к bootloop. Используйте boot.img только от точной версии прошивки/сборки и заранее сделайте резервную копию важных данных.</p>
</div>
