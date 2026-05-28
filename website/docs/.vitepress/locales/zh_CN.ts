import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'zh-CN',
  description: '一个基于内核，为安卓 GKI 准备的 root 方案。',

  themeConfig: {
    logo: '/logo.svg',
    nav: nav(),

    lastUpdatedText: '最后更新',

    sidebar: {
      '/zh_CN/guide/': sidebarGuide()
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/qrjhamron/ApexSU' }
    ],

    footer: {
        message: 'Telegram: @smoothlady',
        copyright: 'Copyright © 2022-现在 ApexSU 开发者。'
    },

    editLink: {
        pattern: 'https://github.com/qrjhamron/ApexSU/edit/main/website/docs/:path',
        text: '在 GitHub 中编辑本页'
    }
  }
})

function nav() {
  return [
    { text: '指南', link: '/zh_CN/guide/what-is-apexsu' },
    { text: '下载', link: 'https://github.com/qrjhamron/ApexSU/releases' },
    { text: '@smoothlady', link: 'https://t.me/smoothlady' }
  ]
}

function sidebarGuide() {
  return [
    {
        text: 'Guide',
        items: [
          { text: '什么是 ApexSU？', link: '/zh_CN/guide/what-is-apexsu' },
          { text: 'ApexSU 模块与 Magisk 的差异', link: '/zh_CN/guide/difference-with-magisk' },
          { text: '安装', link: '/zh_CN/guide/installation' },
          { text: '如何构建？', link: '/zh_CN/guide/how-to-build' },
          { text: 'GKI 支持策略', link: '/zh_CN/guide/how-to-integrate-for-non-gki'},
          { text: '模块开发指南', link: '/zh_CN/guide/module.md' },
          { text: '元模块', link: '/zh_CN/guide/metamodule.md' },
          { text: '模块 Web 界面', link: '/zh_CN/guide/module-webui.md' },
          { text: '模块配置', link: '/zh_CN/guide/module-config.md' },
          { text: 'App Profile', link: '/zh_CN/guide/app-profile.md' },
          { text: '救砖', link: '/zh_CN/guide/rescue-from-bootloop.md' },
          { text: '常见问题', link: '/zh_CN/guide/faq' },
          { text: '隐藏功能', link: '/zh_CN/guide/hidden-features' },
        ]
    }
  ]
}
