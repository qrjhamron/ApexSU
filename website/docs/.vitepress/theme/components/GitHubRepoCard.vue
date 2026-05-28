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
  <section class="repo-row" aria-label="Repository status">
    <a :href="repoUrl" target="_blank" rel="noreferrer" class="repo-row__repo">
      <img src="/github.svg" alt="" aria-hidden="true" />
      <span>Repository</span>
      <strong>qrjhamron/ApexSU</strong>
    </a>
    <div class="repo-row__stat">
      <span>Stars</span>
      <strong>{{ stars }}</strong>
    </div>
    <div class="repo-row__stat">
      <span>Last update</span>
      <strong>{{ loading ? 'Loading…' : lastUpdate }}</strong>
    </div>
  </section>
</template>
