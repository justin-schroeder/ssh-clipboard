<script setup>
import { computed, onMounted, ref } from 'vue'

const stars = ref(null)

onMounted(async () => {
  try {
    // Served by the Cloudflare Worker (edge-cached, token-backed).
    const res = await fetch('/api/stars')
    if (res.ok) {
      const data = await res.json()
      if (typeof data.stars === 'number') {
        stars.value = data.stars
        return
      }
    }
  } catch {
    /* fall through to direct fetch */
  }
  try {
    // Local dev fallback: vite has no worker, ask GitHub directly.
    const res = await fetch('https://api.github.com/repos/standardagents/ssh-clipboard')
    if (!res.ok) return
    const data = await res.json()
    if (typeof data.stargazers_count === 'number') stars.value = data.stargazers_count
  } catch {
    /* no count shown on failure */
  }
})

const showCount = computed(() => stars.value !== null && stars.value >= 50)

const label = computed(() => {
  if (stars.value >= 1000) {
    return (stars.value / 1000).toFixed(1).replace(/\.0$/, '') + 'k'
  }
  return String(stars.value)
})
</script>

<template>
  <a
    class="group inline-flex items-center gap-1.5 border border-linebright bg-raised px-2.5 py-1 text-[0.85rem] leading-[1.4] text-dim transition-colors hover:border-mintdim hover:text-mint"
    href="https://github.com/standardagents/ssh-clipboard"
    aria-label="Star ssh-clipboard on GitHub"
  >
    <svg
      class="h-[0.85em] w-[0.85em] transition-colors group-hover:text-amberish"
      viewBox="0 0 16 16"
      fill="currentColor"
      aria-hidden="true"
    >
      <path d="M8 .25a.75.75 0 0 1 .673.418l1.882 3.815 4.21.612a.75.75 0 0 1 .416 1.279l-3.046 2.97.719 4.192a.75.75 0 0 1-1.088.791L8 12.347l-3.766 1.98a.75.75 0 0 1-1.088-.79l.72-4.194L.818 6.374a.75.75 0 0 1 .416-1.28l4.21-.611L7.327.668A.75.75 0 0 1 8 .25Z" />
    </svg>
    <span>star</span>
    <span
      v-if="showCount"
      class="ml-0.5 border-l border-linebright pl-2.5 font-semibold tabular-nums text-bright"
    >{{ label }}</span>
  </a>
</template>
