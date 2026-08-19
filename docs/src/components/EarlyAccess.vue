<script setup>
import { ref } from 'vue'

const name = ref('')
const email = ref('')
const state = ref('idle') // idle | sending | done | error
const errorMsg = ref('')

const inputClass =
  'min-w-0 rounded-lg border border-[#262a40] bg-[#0a0c12] px-3.5 py-2.5 font-mono text-[0.88rem] text-bright transition-colors placeholder:text-faint focus:border-eaviolet focus:outline-none'

async function submit() {
  if (!name.value.trim() || !email.value.trim()) return
  state.value = 'sending'
  errorMsg.value = ''
  try {
    const res = await fetch('/api/early-access', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: name.value.trim(), email: email.value.trim() }),
    })
    if (!res.ok) {
      const data = await res.json().catch(() => ({}))
      throw new Error(data.error || 'Something went wrong. Please try again.')
    }
    state.value = 'done'
  } catch (error) {
    errorMsg.value = error.message
    state.value = 'error'
  }
}
</script>

<template>
  <section
    class="mt-14 rounded-xl border border-[#23263a] bg-eabg p-6 shadow-[0_20px_50px_-25px_rgba(0,0,0,0.7)]"
  >
    <div class="mb-3.5 flex items-center gap-3">
      <span class="inline-flex text-bright">
        <svg class="h-[22px] w-[22px]" viewBox="0 0 150 150" fill="currentColor" aria-hidden="true">
          <path d="M44.06,0v44.08H0v105.92h105.93v-44.07h44.07V0H44.06ZM19.09,130.91c-16.47-16.47-5.29-54.45,24.96-85.18v60.2h60.23c-30.73,30.27-68.71,41.47-85.2,24.98ZM105.93,104.29v-60.21h-60.21C76.46,13.8,114.42,2.6,130.91,19.09c16.51,16.49,5.31,54.47-24.98,85.2Z" />
        </svg>
      </span>
      <span class="text-[1.02rem] font-bold tracking-[-0.01em] text-bright">StandardAgents</span>
      <span
        class="rounded-md border border-[#383d6e] bg-[rgba(91,94,232,0.14)] px-2.5 py-0.5 text-[0.66rem] font-bold tracking-[0.14em] text-eaviolet"
      >EARLY ACCESS</span>
    </div>
    <p class="mb-5 max-w-[52rem] text-[0.92rem] leading-[1.7] text-dim">
      From the team behind
      <a class="border-b border-linebright text-ink transition-colors hover:border-eaviolet hover:text-eaviolet" href="https://formkit.com">FormKit</a>,
      <a class="border-b border-linebright text-ink transition-colors hover:border-eaviolet hover:text-eaviolet" href="https://tempo.formkit.com">Tempo</a>,
      <a class="border-b border-linebright text-ink transition-colors hover:border-eaviolet hover:text-eaviolet" href="https://dmux.ai">dmux</a>,
      <a class="border-b border-linebright text-ink transition-colors hover:border-eaviolet hover:text-eaviolet" href="https://auto-animate.formkit.com">AutoAnimate</a>, and
      <a class="border-b border-linebright text-ink transition-colors hover:border-eaviolet hover:text-eaviolet" href="https://drag-and-drop.formkit.com">Drag and Drop</a>
      — Standard Agents is an open standard for creating domain-specific agents you can
      distribute and compose together to form safe, efficient, and effective agents. Join
      the early access waitlist below.
    </p>
    <form
      v-if="state !== 'done'"
      class="grid grid-cols-[1fr_1fr_auto] gap-3 max-sm:grid-cols-1"
      @submit.prevent="submit"
    >
      <input v-model="name" :class="inputClass" type="text" placeholder="Name" autocomplete="name" required />
      <input v-model="email" :class="inputClass" type="email" placeholder="Email" autocomplete="email" required />
      <button
        class="cursor-pointer whitespace-nowrap rounded-lg border border-[#6a6ef2] bg-[linear-gradient(180deg,#6a6ef2,#5b5ee8)] px-4.5 py-2.5 font-mono text-[0.88rem] font-bold text-white transition-[filter,transform] duration-150 hover:-translate-y-px hover:brightness-112 disabled:cursor-default disabled:opacity-60 disabled:hover:translate-y-0"
        type="submit"
        :disabled="state === 'sending'"
      >
        {{ state === 'sending' ? 'Sending…' : 'Request Early Access' }}
      </button>
    </form>
    <p v-else class="text-[0.92rem] text-mint">✓ You're on the list — we'll be in touch.</p>
    <p v-if="state === 'error'" class="mt-2.5 text-[0.8rem] text-[#fb7185]">{{ errorMsg }}</p>
  </section>
</template>
