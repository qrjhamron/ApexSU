<script setup lang="ts">
import { onMounted, ref } from 'vue'

const stars = ref('—')
const lastUpdate = ref('Unavailable')
const loading = ref(true)

const repoUrl = 'https://github.com/qrjhamron/ApexSU'
const apiUrl = 'https://api.github.com/repos/qrjhamron/ApexSU'

function formatDate(input: string): string {
  const date = new Date(input)
  if (Number.isNaN(date.getTime())) {
    return 'Unavailable'
  }
  return date.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  })
}

onMounted(async () => {
  try {
    const response = await fetch(apiUrl, {
      headers: { Accept: 'application/vnd.github+json' }
    })
    if (!response.ok) {
      throw new Error(`GitHub API returned ${response.status}`)
    }
    const payload = await response.json()
    if (typeof payload.stargazers_count === 'number') {
      stars.value = payload.stargazers_count.toLocaleString()
    }
    const timestamp =
      typeof payload.pushed_at === 'string'
        ? payload.pushed_at
        : typeof payload.updated_at === 'string'
          ? payload.updated_at
          : ''
    if (timestamp) {
      lastUpdate.value = formatDate(timestamp)
    }
  } catch {
    // Keep safe fallback values when API fails/offline/rate-limited.
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <section class="repo-card">
    <header class="repo-card__header">
      <p class="repo-card__label">Repository</p>
      <a :href="repoUrl" target="_blank" rel="noreferrer" class="repo-card__link">
        <img src="/github.svg" alt="" aria-hidden="true" />
        <span>qrjhamron/ApexSU</span>
      </a>
    </header>

    <div class="repo-card__stats">
      <div class="repo-stat">
        <p class="repo-stat__name">Stars</p>
        <p class="repo-stat__value">{{ stars }}</p>
      </div>
      <div class="repo-stat">
        <p class="repo-stat__name">Last update</p>
        <p class="repo-stat__value">
          {{ loading ? 'Loading…' : lastUpdate }}
        </p>
      </div>
    </div>

    <a
      :href="repoUrl"
      target="_blank"
      rel="noreferrer"
      class="repo-card__star"
      aria-label="Star on GitHub"
    >
      <img src="/github.svg" alt="" aria-hidden="true" />
      <span>Star on GitHub</span>
    </a>
  </section>
</template>
