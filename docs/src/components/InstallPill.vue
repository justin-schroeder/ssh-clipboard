<script setup>
import { ref } from 'vue'

const copied = ref(false)

async function copy() {
  await navigator.clipboard.writeText('npm i -g ssh-clipboard')
  copied.value = true
  setTimeout(() => (copied.value = false), 1600)
}
</script>

<template>
  <div class="flex flex-col items-center gap-2.5 xl:items-start">
    <button
      class="group inline-flex cursor-pointer items-center gap-2.5 rounded-full border border-mintdim bg-raised px-5.5 py-3 font-mono text-base text-bright transition-[border-color,transform] duration-150 [background-image:linear-gradient(180deg,rgba(78,229,133,0.07),rgba(78,229,133,0.02))] animate-[pill-glow_3.2s_ease-in-out_infinite] hover:-translate-y-px hover:border-mint active:translate-y-0 max-sm:gap-2 max-sm:px-4 max-sm:text-[0.85rem]"
      type="button"
      aria-label="Copy install command"
      @click="copy"
    >
      <span class="font-bold text-mint">$</span>
      <span class="whitespace-nowrap">npm i -g ssh-clipboard</span>
      <span class="h-[1.15em] w-[0.55em] bg-mint animate-[blink_1.1s_steps(1,end)_infinite]"></span>
      <span
        class="ml-2 border-l border-linebright pl-3.5 text-[0.72rem] tracking-[0.05em] text-dim transition-colors group-hover:text-mint"
      >{{ copied ? 'copied ✓' : 'copy' }}</span>
    </button>
    <p class="text-[0.8rem] text-faint">
      then run <code class="text-dim">ssh-clipboard</code> — that's the whole setup
    </p>
  </div>
</template>
