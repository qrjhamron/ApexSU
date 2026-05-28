import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'vi-VN',
  description: 'Một giải pháp root trực tiếp trên kernel dành cho các thiết bị hỗ trợ GKI.',

  themeConfig: {
    logo: '/apexsu_logo.svg',
    nav: nav(),

    lastUpdatedText: 'cập nhật lần cuối',

    sidebar: {
      '/vi_VN/guide/': sidebarGuide()
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
        text: 'Chỉnh sửa trang này trên GitHub'
    }
  }
})

function nav() {
  return [
    { text: 'Hướng Dẫn', link: '/vi_VN/guide/what-is-apexsu' },
    { text: 'Tải xuống', link: 'https://github.com/qrjhamron/ApexSU/releases' },
    { text: '@smoothlady', link: 'https://t.me/smoothlady' }
  ]
}

function sidebarGuide() {
  return [
    {
        text: 'Hướng Dẫn',
        items: [
          { text: 'ApexSU là gì?', link: '/vi_VN/guide/what-is-apexsu' },
          { text: 'Cách cài đặt', link: '/vi_VN/guide/installation' },
          { text: 'Cách để build?', link: '/vi_VN/guide/how-to-build' },
          { text: 'Chính sách hỗ trợ GKI', link: '/vi_VN/guide/how-to-integrate-for-non-gki'},
          { text: 'Metamodule', link: '/vi_VN/guide/metamodule.md' },
          { text: 'Cấu hình module', link: '/vi_VN/guide/module-config.md' },
          { text: 'FAQ - Câu hỏi thường gặp', link: '/vi_VN/guide/faq' },
        ]
    }
  ]
}
