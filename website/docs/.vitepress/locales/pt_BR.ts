import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'pt-BR',
  description: 'Uma solução root baseada em kernel para dispositivos Android GKI.',

  themeConfig: {
    logo: '/apexsu_logo.svg',
    nav: nav(),

    lastUpdatedText: 'Última atualização',

    sidebar: {
      '/pt_BR/guide/': sidebarGuide()
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/qrjhamron/ApexSU' }
    ],

    footer: {
        message: 'Telegram: @smoothlady',
        copyright: 'Copyright © 2022-presente Desenvolvedores do ApexSU.'
    },

    editLink: {
        pattern: 'https://github.com/qrjhamron/ApexSU/edit/main/website/docs/:path',
        text: 'Edite esta página no GitHub'
    }
  }
})

function nav() {
  return [
    { text: 'Guia', link: '/pt_BR/guide/what-is-apexsu' },
    { text: 'Download', link: 'https://github.com/qrjhamron/ApexSU/releases' },
    { text: '@smoothlady', link: 'https://t.me/smoothlady' }
  ]
}

function sidebarGuide() {
  return [
    {
        text: 'Guia',
        items: [
          { text: 'O que é ApexSU?', link: '/pt_BR/guide/what-is-apexsu' },
          { text: 'Diferenças com Magisk', link: '/pt_BR/guide/difference-with-magisk' },
          { text: 'Instalação', link: '/pt_BR/guide/installation' },
          { text: 'Como compilar', link: '/pt_BR/guide/how-to-build' },
          { text: 'Política de suporte GKI', link: '/pt_BR/guide/how-to-integrate-for-non-gki'},
          { text: 'Guias de módulo', link: '/pt_BR/guide/module.md' },
          { text: 'Metamódulo', link: '/pt_BR/guide/metamodule.md' },
          { text: 'Módulo WebUI', link: '/pt_BR/guide/module-webui.md' },
          { text: 'Configuração de Módulo', link: '/pt_BR/guide/module-config.md' },
          { text: 'Perfil do Aplicativo', link: '/pt_BR/guide/app-profile.md' },
          { text: 'Resgate do bootloop', link: '/pt_BR/guide/rescue-from-bootloop.md' },
          { text: 'Perguntas frequentes', link: '/pt_BR/guide/faq' },
          { text: 'Recursos ocultos', link: '/pt_BR/guide/hidden-features' },
        ]
    }
  ]
}
