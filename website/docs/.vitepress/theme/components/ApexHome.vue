<script setup lang="ts">
import { withBase } from 'vitepress'
</script>

<template>
  <div class="apex-home">
    <section class="apex-hero">
      <div class="apex-brand">
        <img src="/apexsu_logo.svg" alt="ApexSU" class="apex-brand__logo" />
        <span class="apex-brand__name">ApexSU</span>
      </div>
      <h1 class="apex-hero__title">ApexSU</h1>
      <p class="apex-hero__tagline">Modern root workflow for Android GKI devices.</p>
      <p class="apex-hero__desc">
        ApexSU provides a focused root workflow for supported Android GKI kernels, with boot image
        installation, LKM support, and a clean manager experience.
      </p>
      <div class="apex-actions">
        <a
          class="apex-btn apex-btn--primary"
          href="https://github.com/qrjhamron/ApexSU/releases"
          target="_blank"
          rel="noreferrer"
        >
          Download
        </a>
        <a
          class="apex-btn apex-btn--secondary"
          href="https://github.com/qrjhamron/ApexSU"
          target="_blank"
          rel="noreferrer"
        >
          <img src="/github.svg" alt="" aria-hidden="true" />
          Star on GitHub
        </a>
      </div>
      <a class="apex-telegram" href="https://t.me/smoothlady" target="_blank" rel="noreferrer">
        @smoothlady
      </a>
    </section>

    <GitHubRepoCard />

    <section class="apex-warning" aria-label="GKI support warning">
      <h2>GKI-only support</h2>
      <p>
        ApexSU only supports supported Android GKI devices. Non-GKI devices are not supported and
        installation must be blocked.
      </p>
    </section>

    <a class="install-row" :href="withBase('/guide/what-is-apexsu.html')">
      <span class="install-row__content">
        <strong>Installation guide</strong>
        <small>Check requirements and supported GKI install flow.</small>
      </span>
      <span class="install-row__arrow" aria-hidden="true">&gt;</span>
    </a>

    <section class="feature-list" aria-label="Feature summary">
      <article class="feature-list__item">
        <h3>GKI-focused</h3>
        <p>Built for supported Android GKI kernels.</p>
      </article>
      <article class="feature-list__item">
        <h3>LKM support</h3>
        <p>Repository LKM by default, local LKM for advanced GKI usage.</p>
      </article>
      <article class="feature-list__item">
        <h3>Clean manager workflow</h3>
        <p>Clear install checks with explicit unsupported-device blocking.</p>
      </article>
    </section>

    <section class="apex-safety">
      <h2>Safety warning</h2>
      <p>
        Modifying boot images can bootloop devices. Use a boot.img matching the exact
        firmware/build and back up important data before installation.
      </p>
    </section>
  </div>
</template>
