<script setup>
import { computed, ref } from 'vue'

const props = defineProps({
  code: { type: String, required: true },
  copyable: { type: Boolean, default: true },
})

const kindClass = {
  cmd: 'text-bright',
  out: 'text-dim',
  comment: 'text-faint italic',
}

const lines = computed(() =>
  props.code
    .replace(/^\n+|\n+$/g, '')
    .split('\n')
    .map((raw) => {
      if (raw.startsWith('$ ')) return { kind: 'cmd', text: raw.slice(2) }
      if (raw.startsWith('#')) return { kind: 'comment', text: raw }
      return { kind: 'out', text: raw }
    })
)

const copyText = computed(() =>
  lines.value
    .filter((l) => l.kind === 'cmd')
    .map((l) => l.text)
    .join('\n')
)

const copied = ref(false)

async function copy() {
  await navigator.clipboard.writeText(copyText.value)
  copied.value = true
  setTimeout(() => (copied.value = false), 1600)
}
</script>

<template>
  <div class="group relative my-5 overflow-hidden border border-line bg-panel">
    <button
      v-if="copyable"
      class="absolute right-2 top-2 cursor-pointer bg-panel px-1 py-0.5 font-mono text-xs text-dim opacity-0 transition-[opacity,color] duration-150 group-hover:opacity-100 focus-visible:opacity-100 hover:text-mint"
      type="button"
      :aria-label="copied ? 'Copied' : 'Copy to clipboard'"
      @click="copy"
    >{{ copied ? '[ copied ✓ ]' : '[ copy ]' }}</button>
    <pre class="overflow-x-auto px-5 py-4 leading-[1.65]"><code class="whitespace-pre border-0 bg-transparent p-0 text-[0.95em]"><template v-for="(line, i) in lines" :key="i"><span :class="kindClass[line.kind]"><span v-if="line.kind === 'cmd'" class="select-none text-mint">$ </span>{{ line.text }}</span>
</template></code></pre>
  </div>
</template>
