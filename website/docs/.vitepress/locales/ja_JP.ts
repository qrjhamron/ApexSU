import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'ja-JP',
  description: 'Android GKI デバイス向けのカーネルベースの root ソリューション',

  themeConfig: {
    logo: '/logo.svg',
    nav: nav(),

    lastUpdatedText: '最終更新',

    sidebar: {
      '/ja_JP/guide/': sidebarGuide()
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/qrjhamron/ApexSU' }
    ],

    footer: {
      message: 'Telegram: @smoothlady',
      copyright: 'Copyright © 2022-現在 ApexSU 開発者。'
    },

    editLink: {
      pattern: 'https://github.com/qrjhamron/ApexSU/edit/main/website/docs/:path',
      text: 'GitHub でこのページを編集'
    }
  }
})

function nav() {
  return [
    { text: 'ガイド', link: '/ja_JP/guide/what-is-apexsu' },
    { text: 'ダウンロード', link: 'https://github.com/qrjhamron/ApexSU/releases' },
    { text: '@smoothlady', link: 'https://t.me/smoothlady' }
  ]
}

function sidebarGuide() {
  return [
    {
      text: 'ガイド',
      items: [
        { text: 'ApexSU とは?', link: '/ja_JP/guide/what-is-apexsu' },
        { text: 'インストール', link: '/ja_JP/guide/installation' },
        { text: 'ビルドするには?', link: '/ja_JP/guide/how-to-build' },
        { text: 'GKI サポート方針', link: '/ja_JP/guide/how-to-integrate-for-non-gki' },
        { text: 'モジュールのガイド', link: '/ja_JP/guide/module.md' },
        { text: 'メタモジュール', link: '/ja_JP/guide/metamodule.md' },
        { text: 'モジュール設定', link: '/ja_JP/guide/module-config.md' },
        { text: 'ブートループからの復旧', link: '/ja_JP/guide/rescue-from-bootloop.md' },
        { text: 'よくある質問', link: '/ja_JP/guide/faq' },
        { text: '隠し機能', link: '/ja_JP/guide/hidden-features' },
      ]
    }
  ]
}
