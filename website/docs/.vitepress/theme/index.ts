import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'
import './custom.css'
import ApexHome from './components/ApexHome.vue'
import GitHubRepoCard from './components/GitHubRepoCard.vue'

export default {
  extends: DefaultTheme,
  enhanceApp(ctx) {
    DefaultTheme.enhanceApp?.(ctx)
    const { app } = ctx
    app.component('ApexHome', ApexHome)
    app.component('GitHubRepoCard', GitHubRepoCard)
  }
} satisfies Theme
