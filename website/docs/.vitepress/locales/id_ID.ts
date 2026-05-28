import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'id-ID',
  description: 'Solusi root kernel-based untuk perangkat Android GKI.',

  themeConfig: {
    logo: '/apexsu_logo.svg',
    nav: nav(),

    lastUpdatedText: 'Update Terakhir',

    sidebar: {
      '/id_ID/guide/': sidebarGuide()
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/qrjhamron/ApexSU' }
    ],

    footer: {
        message: 'Telegram: <a href="https://t.me/smoothlady">@smoothlady</a>',
        copyright: 'Copyright © 2026 qrjhamron'
    },

    editLink: {
        pattern: 'https://github.com/qrjhamron/ApexSU/edit/main/website/docs/:path',
        text: 'Edit Halaman ini di GitHub'
    }
  }
})

function nav() {
  return [
    { text: 'Petunjuk', link: '/id_ID/guide/what-is-apexsu' },
    { text: 'Unduh', link: 'https://github.com/qrjhamron/ApexSU/releases' },
    { text: '@smoothlady', link: 'https://t.me/smoothlady' }
  ]
}

function sidebarGuide() {
  return [
    {
        text: 'Petunjuk',
        items: [
          { text: 'Apa itu ApexSU?', link: '/id_ID/guide/what-is-apexsu' },
          { text: 'Instalasi', link: '/id_ID/guide/installation' },
          { text: 'Kebijakan dukungan GKI', link: '/id_ID/guide/how-to-integrate-for-non-gki' },
          { text: 'Bagaimana cara buildnya?', link: '/id_ID/guide/how-to-build' },
          { text: 'Petunjuk module', link: '/id_ID/guide/module.md' },
          { text: 'Metamodule', link: '/id_ID/guide/metamodule.md' },
          { text: 'Konfigurasi Modul', link: '/id_ID/guide/module-config.md' },
          { text: 'Antisipasi dari bootloop', link: '/id_ID/guide/rescue-from-bootloop.md' },
          { text: 'FAQ', link: '/id_ID/guide/faq' },
        ]
    }
  ]
}
