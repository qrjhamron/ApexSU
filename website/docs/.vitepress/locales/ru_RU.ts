import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'ru-RU',
  description: 'Решение на основе ядра root для устройств Android GKI.',

  themeConfig: {
    logo: '/logo.svg',
    nav: nav(),

    lastUpdatedText: 'последнее обновление',

    sidebar: {
      '/ru_RU/guide/': sidebarGuide()
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/qrjhamron/ApexSU' }
    ],

    footer: {
        message: 'Telegram: @smoothlady',
        copyright: 'Авторские права © 2022-текущее Разработчики ApexSU.'
    },

    editLink: {
        pattern: 'https://github.com/qrjhamron/ApexSU/edit/main/website/docs/:path',
        text: 'Редактировать эту страницу на GitHub'
    }
  }
})

function nav() {
  return [
    { text: 'Руководство', link: '/ru_RU/guide/what-is-apexsu' },
    { text: 'Загрузки', link: 'https://github.com/qrjhamron/ApexSU/releases' },
    { text: '@smoothlady', link: 'https://t.me/smoothlady' }
  ]
}

function sidebarGuide() {
  return [
    {
        text: 'Руководство',
        items: [
          { text: 'Что такое ApexSU?', link: '/ru_RU/guide/what-is-apexsu' },
          { text: 'Установка', link: '/ru_RU/guide/installation' },
          { text: 'Как собрать?', link: '/ru_RU/guide/how-to-build' },
          { text: 'Политика поддержки GKI', link: '/ru_RU/guide/how-to-integrate-for-non-gki'},
          { text: 'Руководство по разработке модулей', link: '/ru_RU/guide/module.md' },
          { text: 'Метамодуль', link: '/ru_RU/guide/metamodule.md' },
          { text: 'Конфигурация модулей', link: '/ru_RU/guide/module-config.md' },
          { text: 'Профиль приложений', link: '/ru_RU/guide/app-profile.md' },
          { text: 'Выход из циклической загрузки', link: '/ru_RU/guide/rescue-from-bootloop.md' },
          { text: 'FAQ', link: '/ru_RU/guide/faq' },
          { text: 'Скрытые возможности', link: '/ru_RU/guide/hidden-features' },
        ]
    }
  ]
}
