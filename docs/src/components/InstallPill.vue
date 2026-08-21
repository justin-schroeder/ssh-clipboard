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
      class="group inline-flex cursor-pointer items-center gap-2.5 border border-mintdim bg-raised px-5 py-3 font-mono text-base text-bright transition-colors duration-150 hover:border-mint max-sm:gap-2 max-sm:px-4 max-sm:text-[0.85rem]"
      type="button"
      aria-label="Copy install command"
      @click="copy"
    >
      <span class="font-bold text-mint">$</span>
      <span class="whitespace-nowrap">npm i -g ssh-clipboard</span>
      <span class="h-[1.15em] w-[0.55em] bg-mint animate-[blink_1.1s_steps(1,end)_infinite]"></span>
      <span
        class="ml-2 whitespace-nowrap text-[0.72rem] tracking-[0.05em] text-dim transition-colors group-hover:text-mint"
      >{{ copied ? '[ copied ✓ ]' : '[ copy ]' }}</span>
    </button>
    <p class="text-[0.8rem] text-faint">
      then run <code class="text-dim">ssh-clipboard</code> to set up your machines
    </p>
  </div>
</template>
