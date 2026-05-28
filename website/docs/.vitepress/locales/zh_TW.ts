import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'zh-TW',
  description: '一個基於核心，適用於 Android GKI 的 Root 解決方案。',

  themeConfig: {
    logo: '/apexsu_logo.svg',
    nav: nav(),

    lastUpdatedText: '上次更新',

    sidebar: {
      '/zh_TW/guide/': sidebarGuide()
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/qrjhamron/ApexSU' }
    ],

    footer: {
        message: 'Telegram: @smoothlady',
        copyright: 'Copyright © 2022-目前 ApexSU 開發人員。'
    },

    editLink: {
        pattern: 'https://github.com/qrjhamron/ApexSU/edit/main/website/docs/:path',
        text: '在 GitHub 中編輯本頁面'
    }
  }
})

function nav() {
  return [
    { text: '指南', link: '/zh_TW/guide/what-is-apexsu' },
    { text: '下載', link: 'https://github.com/qrjhamron/ApexSU/releases' },
    { text: '@smoothlady', link: 'https://t.me/smoothlady' }
  ]
}

function sidebarGuide() {
  return [
    {
        text: 'Guide',
        items: [
          { text: '什麼是 ApexSU？', link: '/zh_TW/guide/what-is-apexsu' },
          { text: 'ApexSU 與 Magisk 的差異', link: '/zh_TW/guide/difference-with-magisk' },
          { text: '安裝', link: '/zh_TW/guide/installation' },
          { text: '如何建置？', link: '/zh_TW/guide/how-to-build' },
          { text: 'GKI 支援政策', link: '/zh_TW/guide/how-to-integrate-for-non-gki'},
          { text: '模組指南', link: '/zh_TW/guide/module.md' },
          { text: '元模組', link: '/zh_TW/guide/metamodule.md' },
          { text: '模組 WebUI', link: '/zh_TW/guide/module-webui.md' },
          { text: '模組配置', link: '/zh_TW/guide/module-config.md' },
          { text: 'App Profile', link: '/zh_TW/guide/app-profile.md' },
          { text: '搶救開機迴圈', link: '/zh_TW/guide/rescue-from-bootloop.md' },
          { text: '常見問題', link: '/zh_TW/guide/faq' },
          { text: '隱藏功能', link: '/zh_TW/guide/hidden-features' },
        ]
    }
  ]
}
