import { defineConfig, SiteConfig } from 'vitepress'
import locales from './locales'
import { readdir, writeFile } from 'fs/promises'
import { resolve } from 'path'

export default defineConfig({
  base: '/ApexSU/',
  title: 'ApexSU',
  description: 'Modern root solution for supported Android GKI devices.',
  locales: locales.locales,
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
    ['meta', { name: 'theme-color', content: '#111315' }],
    ['meta', { name: 'og:title', content: 'ApexSU' }],
    [
      'meta',
      {
        name: 'og:description',
        content:
          'Modern root solution for supported Android GKI devices with boot image workflow and LKM support.'
      }
    ],
    ['meta', { name: 'og:image', content: '/apexsu_logo.svg' }],
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }]
  ],
  sitemap: {
    hostname: 'https://qrjhamron.github.io/ApexSU/'
  },
  buildEnd: async (config: SiteConfig) => {
    const templateDir = resolve(config.outDir, 'templates')
    const templateList = resolve(templateDir, 'index.json')
    let files: string[] = []
    try {
      files = await readdir(templateDir)
      files = files.filter((file) => !file.startsWith('.'))
    } catch {
      // ignore
    }
    await writeFile(templateList, JSON.stringify(files))
  }
})
