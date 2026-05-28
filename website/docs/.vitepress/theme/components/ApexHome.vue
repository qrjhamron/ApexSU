<template>
  <div class="apex-home">
    <section class="apex-hero">
      <div class="apex-hero__main">
        <div class="apex-brand">
          <img src="/apexsu_logo.svg" alt="ApexSU" class="apex-brand__logo" />
          <p class="apex-brand__name">ApexSU</p>
        </div>

        <h1 class="apex-hero__title">ApexSU</h1>
        <p class="apex-hero__tagline">
          Modern root workflow for supported Android GKI devices
        </p>
        <p class="apex-hero__desc">
          ApexSU provides a focused root workflow for supported Android GKI kernels,
          with boot image installation, LKM support, and a modern Android manager experience.
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
          <a
            class="apex-btn apex-btn--ghost"
            href="https://github.com/qrjhamron/ApexSU"
            target="_blank"
            rel="noreferrer"
          >
            Repository
          </a>
          <a
            class="apex-btn apex-btn--ghost"
            href="https://t.me/smoothlady"
            target="_blank"
            rel="noreferrer"
          >
            @smoothlady
          </a>
        </div>
      </div>

      <GitHubRepoCard />
    </section>

    <section class="apex-warning">
      <h2>GKI-only support</h2>
      <p>
        ApexSU only supports supported Android GKI devices. Non-GKI devices are not supported,
        and installation must be blocked.
      </p>
    </section>

    <section class="apex-feature-grid">
      <article class="apex-feature">
        <h3>GKI-focused support</h3>
        <p>ApexSU only supports supported GKI devices and blocks unsupported install paths.</p>
      </article>
      <article class="apex-feature">
        <h3>Boot image workflow</h3>
        <p>Installation requires a boot.img that matches your exact firmware/build.</p>
      </article>
      <article class="apex-feature">
        <h3>LKM support</h3>
        <p>Repository LKM is recommended by default. Local LKM is advanced and GKI-only.</p>
      </article>
      <article class="apex-feature">
        <h3>Modern manager UI</h3>
        <p>Manager-driven checks provide clear install gating and module lifecycle control.</p>
      </article>
      <article class="apex-feature">
        <h3>Open-source development</h3>
        <p>Source code, issue tracking, and releases are public on GitHub.</p>
      </article>
      <article class="apex-feature">
        <h3>Safety-first installation flow</h3>
        <p>ApexSU keeps compatibility checks explicit and refuses unsupported device paths.</p>
      </article>
    </section>

    <section class="apex-links">
      <h2>Quick links</h2>
      <div class="apex-links__grid">
        <a href="https://github.com/qrjhamron/ApexSU/releases" target="_blank" rel="noreferrer">Download</a>
        <a href="https://github.com/qrjhamron/ApexSU" target="_blank" rel="noreferrer">GitHub</a>
        <a href="https://github.com/qrjhamron/ApexSU/releases" target="_blank" rel="noreferrer">Releases</a>
        <a href="/guide/installation">Installation guide</a>
        <a href="https://t.me/smoothlady" target="_blank" rel="noreferrer">@smoothlady</a>
      </div>
    </section>

    <section class="apex-safety">
      <h2>Safety warning</h2>
      <p>
        Modifying boot images can bootloop devices. Use boot.img matching the exact firmware/build,
        and back up important data before installation.
      </p>
    </section>
  </div>
</template>
