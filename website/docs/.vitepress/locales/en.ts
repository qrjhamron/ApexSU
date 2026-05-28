import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'en-US',
  description: 'Modern root solution for supported Android GKI devices.',

  themeConfig: {
    logo: '/apexsu_logo.svg',
    nav: nav(),
    lastUpdatedText: 'Last updated',
    sidebar: {
      '/guide/': sidebarGuide()
    },
    socialLinks: [
      { icon: 'github', link: 'https://github.com/qrjhamron/ApexSU' }
    ],
    footer: {
      message: 'Telegram: @smoothlady',
      copyright: 'Copyright © 2022-present ApexSU developers.'
    },
    editLink: {
      pattern: 'https://github.com/qrjhamron/ApexSU/edit/main/website/docs/:path',
      text: 'Edit this page on GitHub'
    }
  }
})

function nav() {
  return [
    { text: 'Guide', link: '/guide/what-is-apexsu' },
    { text: 'Download', link: 'https://github.com/qrjhamron/ApexSU/releases' },
    { text: '@smoothlady', link: 'https://t.me/smoothlady' }
  ]
}

function sidebarGuide() {
  return [
    {
      text: 'Guide',
      items: [
        { text: 'What is ApexSU?', link: '/guide/what-is-apexsu' },
        { text: 'Installation', link: '/guide/installation' },
        { text: 'GKI Support Policy', link: '/guide/how-to-integrate-for-non-gki' },
        { text: 'FAQ', link: '/guide/faq' },
        { text: 'Module Guide', link: '/guide/module.md' },
        { text: 'Metamodule', link: '/guide/metamodule.md' },
        { text: 'Rescue from bootloop', link: '/guide/rescue-from-bootloop.md' },
        { text: 'How to build', link: '/guide/how-to-build' }
      ]
    }
  ]
}
