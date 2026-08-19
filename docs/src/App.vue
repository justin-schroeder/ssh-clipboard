<script setup>
import { onBeforeUnmount, onMounted, ref } from 'vue'
import CodeBlock from './components/CodeBlock.vue'
import EarlyAccess from './components/EarlyAccess.vue'
import InstallPill from './components/InstallPill.vue'
import MeshDemo from './components/MeshDemo.vue'
import StarButton from './components/StarButton.vue'

const sections = [
  { id: 'install', title: 'Install' },
  { id: 'quick-start', title: 'Quick start' },
  { id: 'commands', title: 'Commands' },
  { id: 'how-it-works', title: 'How it works' },
  { id: 'updates', title: 'Updates' },
]

const activeId = ref('install')
let observer

onMounted(() => {
  observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) activeId.value = entry.target.id
      }
    },
    { rootMargin: '-20% 0px -70% 0px' }
  )
  for (const section of sections) {
    const el = document.getElementById(section.id)
    if (el) observer.observe(el)
  }
})

onBeforeUnmount(() => observer?.disconnect())

const features = [
  {
    name: 'native',
    text: 'macOS pasteboard plus Linux Wayland and X11 — real system clipboards, not terminal escape tricks.',
  },
  {
    name: 'private',
    text: 'Persistent peer-to-peer SSH. No relay, no account, no open port, no new encryption key to manage.',
  },
  {
    name: 'faithful',
    text: 'Preserves every available clipboard representation — rich text, images, files — not just text or PNG.',
  },
  {
    name: 'invisible',
    text: 'Raycast and other clipboard managers see ordinary system clipboard writes. Nothing to configure.',
  },
  {
    name: 'fast',
    text: 'Raw bytes over persistent connections, with deduplication and newest-value queues.',
  },
  {
    name: 'self-updating',
    text: 'Daemons gossip verified versions peer-to-peer and converge on the latest stable release — no coordinator.',
  },
]

const commands = [
  {
    cmd: 'ssh-clipboard',
    desc: 'First run: setup TUI. After that: the live monitor dashboard.',
  },
  {
    cmd: 'ssh-clipboard setup',
    desc: 'Add, verify, or repair peers. Re-runs installation where needed.',
  },
  {
    cmd: 'ssh-clipboard monitor',
    desc: 'Watch clipboard values and peer health in a Ratatui dashboard. --plain streams readable lines, --json streams NDJSON.',
  },
  {
    cmd: 'ssh-clipboard status',
    desc: 'Daemon and connection status. --json for automation.',
  },
  {
    cmd: 'ssh-clipboard update',
    desc: 'Install the latest stable release. --check compares versions without installing.',
  },
  {
    cmd: 'ssh-clipboard service',
    desc: 'Manage the per-user background service: install, start, stop, restart.',
  },
]

const docLink =
  'text-mint border-b border-mint/35 transition-colors hover:border-mint'
</script>

<template>
  <header
    class="sticky top-0 z-10 border-b border-line bg-bg/80 backdrop-blur-[10px]"
  >
    <div class="flex w-full items-center justify-between gap-4 px-6 py-3 max-md:px-4 max-md:py-2.5">
      <a class="whitespace-nowrap font-semibold text-bright" href="#top">
        ssh-clipboard<span class="text-mint animate-[blink_1.2s_steps(1,end)_infinite]">_</span>
      </a>
      <nav class="flex items-center gap-4 text-[0.85rem] max-md:gap-3 max-md:text-[0.78rem]">
        <a class="text-dim transition-colors hover:text-mint max-[560px]:hidden" href="#install">install</a>
        <a class="text-dim transition-colors hover:text-mint max-[560px]:hidden" href="#commands">commands</a>
        <a class="text-dim transition-colors hover:text-mint max-[560px]:hidden" href="#how-it-works">how&nbsp;it&nbsp;works</a>
        <StarButton />
      </nav>
    </div>
  </header>

  <main id="top">
    <!-- ── hero ─────────────────────────────── -->
    <section class="relative px-5 pb-10 pt-14 text-left max-xl:text-center max-md:pt-10">
      <div
        class="pointer-events-none absolute -inset-x-96 -top-32 h-[34rem] blur-[10px] [background:radial-gradient(38rem_20rem_at_38%_20%,rgba(78,229,133,0.1),transparent_70%)] animate-[drift-a_14s_ease-in-out_infinite_alternate]"
      ></div>
      <div
        class="pointer-events-none absolute -inset-x-96 -top-24 h-[30rem] blur-[10px] [background:radial-gradient(32rem_18rem_at_66%_30%,rgba(167,139,250,0.07),transparent_70%)] animate-[drift-b_18s_ease-in-out_infinite_alternate]"
      ></div>

      <div
        class="mx-auto grid w-[min(90rem,calc(100vw-7rem))] grid-cols-[minmax(21rem,27rem)_minmax(0,1fr)] items-center gap-12 max-xl:grid-cols-1 max-md:gap-6 max-md:w-[calc(100vw-2.5rem)]"
      >
        <div>
          <p
            class="mb-7 flex flex-wrap justify-start gap-2 opacity-0 animate-[rise_0.9s_cubic-bezier(0.16,1,0.3,1)_forwards] [animation-delay:0.05s] max-xl:justify-center"
          >
            <span class="rounded-full border border-line bg-raised/70 px-3 py-0.5 text-[0.72rem] tracking-[0.06em] text-dim backdrop-blur-sm">macOS</span>
            <span class="rounded-full border border-line bg-raised/70 px-3 py-0.5 text-[0.72rem] tracking-[0.06em] text-dim backdrop-blur-sm">Linux</span>
            <span class="rounded-full border border-line bg-raised/70 px-3 py-0.5 text-[0.72rem] tracking-[0.06em] text-dim backdrop-blur-sm">arm64 · x64</span>
            <span class="rounded-full border border-mintdim bg-raised/70 px-3 py-0.5 text-[0.72rem] tracking-[0.06em] text-mint backdrop-blur-sm">MIT</span>
          </p>
          <h1
            class="mb-6 text-[clamp(2.6rem,4.2vw,4.1rem)] font-semibold leading-[1.12] tracking-[-0.03em] text-bright opacity-0 animate-[rise_0.9s_cubic-bezier(0.16,1,0.3,1)_forwards] [animation-delay:0.15s]"
          >
            <span>Your clipboard,</span><br />
            <span
              class="bg-clip-text text-transparent [background-image:linear-gradient(90deg,#4ee585_20%,#6dd3e8_40%,#4ee585_60%,#a7f3c8_80%,#4ee585)] [background-size:300%_100%] [filter:drop-shadow(0_0_22px_rgba(78,229,133,0.35))] animate-[shimmer_5s_linear_infinite]"
            >everywhere.</span>
          </h1>
          <p
            class="mb-8 max-w-[38rem] text-[1.05rem] text-dim opacity-0 animate-[rise_0.9s_cubic-bezier(0.16,1,0.3,1)_forwards] [animation-delay:0.3s] max-xl:mx-auto"
          >
            Peer-to-peer clipboard sync over encrypted SSH.
            Copy here, paste there — text, images, files, rich content, native formats intact.
            <span class="text-amberish">Written in Rust ackchyually.</span>
          </p>
          <div class="opacity-0 animate-[rise_0.9s_cubic-bezier(0.16,1,0.3,1)_forwards] [animation-delay:0.45s]">
            <InstallPill />
          </div>
        </div>
        <div class="min-w-0 opacity-0 animate-[rise_0.9s_cubic-bezier(0.16,1,0.3,1)_forwards] [animation-delay:0.6s]">
          <MeshDemo />
        </div>
      </div>

      <p
        class="mt-10 flex flex-wrap justify-center gap-3.5 text-[0.85rem] tracking-[0.03em] text-dim opacity-0 animate-[rise_0.9s_cubic-bezier(0.16,1,0.3,1)_forwards] [animation-delay:0.8s] max-md:mt-7 max-md:gap-2 max-md:text-[0.78rem]"
      >
        <span><b class="font-bold text-mint">0</b> relays</span>
        <span class="text-faint">·</span>
        <span><b class="font-bold text-mint">0</b> accounts</span>
        <span class="text-faint">·</span>
        <span><b class="font-bold text-mint">0</b> open ports</span>
        <span class="text-faint">·</span>
        <span><b class="font-bold text-mint">∞</b> machines</span>
      </p>
    </section>

    <!-- ── features ─────────────────────────── -->
    <section
      class="mx-auto grid max-w-[46rem] grid-cols-[repeat(auto-fit,minmax(15rem,1fr))] gap-3.5 px-5 pb-4 pt-10"
    >
      <div
        v-for="f in features"
        :key="f.name"
        class="rounded-lg border border-line bg-panel p-4 transition-colors hover:border-linebright"
      >
        <h3 class="mb-1.5 text-[0.95rem] font-semibold text-bright">
          <span class="font-semibold text-mint">*</span> {{ f.name }}
        </h3>
        <p class="text-[0.85rem] leading-[1.6] text-dim">{{ f.text }}</p>
      </div>
    </section>

    <!-- ── docs with jump-link sidebar ──────── -->
    <div
      class="mx-auto grid w-[min(66rem,calc(100vw-2.5rem))] grid-cols-[9.5rem_minmax(0,46rem)] justify-center gap-14 max-[1150px]:grid-cols-[minmax(0,46rem)]"
    >
      <aside
        class="sticky top-[5.5rem] flex flex-col gap-0.5 self-start pt-12 text-[0.82rem] max-[1150px]:hidden"
        aria-label="On this page"
      >
        <span class="mb-2 text-[0.68rem] tracking-[0.14em] text-faint">
          <span class="text-mint">#</span> docs
        </span>
        <a
          v-for="s in sections"
          :key="s.id"
          :href="'#' + s.id"
          class="relative w-fit py-1 transition-colors after:absolute after:inset-x-0 after:bottom-[0.05rem] after:h-px after:origin-left after:bg-mint after:transition-transform after:content-['']"
          :class="activeId === s.id ? 'text-mint after:scale-x-100' : 'text-dim after:scale-x-0 hover:text-bright'"
        >{{ s.title }}</a>
      </aside>

      <div>
        <section id="install" class="pb-2 pt-11">
          <h2 class="mb-4 text-[1.35rem] font-semibold text-bright">
            <span class="text-mint">#</span> Install
          </h2>
          <p class="mb-4 max-w-[42rem]">
            One package, two commands. The npm package is just the installer — the thing that
            runs is a native Rust binary with a per-user background service
            (<code>launchd</code> on macOS, <code>systemd</code> on Linux).
          </p>
          <CodeBlock
            code="$ npm i -g ssh-clipboard
$ ssh-clipboard"
          />
          <p class="mb-4 max-w-[42rem]">You'll need:</p>
          <ul class="mb-4 ml-6 list-disc space-y-1.5">
            <li>macOS or Linux (Wayland or X11), on arm64 or x64</li>
            <li>Node ≥ 18 — only for <code>npm install</code>; the daemon has no Node dependency</li>
            <li>
              Passwordless SSH between your machines —
              <a :class="docLink" href="https://tailscale.com/kb/1193/tailscale-ssh">Tailscale SSH</a> is perfect,
              plain old <code>~/.ssh</code> keys work too
            </li>
          </ul>
        </section>

        <section id="quick-start" class="pb-2 pt-11">
          <h2 class="mb-4 text-[1.35rem] font-semibold text-bright">
            <span class="text-mint">#</span> Quick start
          </h2>
          <p class="mb-4 max-w-[42rem]">
            Run <code>ssh-clipboard</code> with no arguments. The first-run TUI offers compatible
            online machines from Tailscale when it's installed, or accepts any passwordless SSH
            destination. For each peer it:
          </p>
          <ol class="mb-4 ml-6 list-decimal space-y-1.5">
            <li>verifies the connection actually works,</li>
            <li>installs the right native binary for that platform over SSH,</li>
            <li>starts the per-user background service on both ends.</li>
          </ol>
          <p class="mb-4 max-w-[42rem]">
            That's it. Copy on one machine, paste on another. After setup it just feels like
            one clipboard — there is no step two.
          </p>
          <CodeBlock
            code="$ ssh-clipboard status
running as macbook (node-a1, pasteboard, version 0.2.0)
connected: fedora
connected: macbookserver"
          />
        </section>

        <section id="commands" class="pb-2 pt-11">
          <h2 class="mb-4 text-[1.35rem] font-semibold text-bright">
            <span class="text-mint">#</span> Commands
          </h2>
          <dl class="my-5 overflow-hidden rounded-lg border border-line">
            <template v-for="c in commands" :key="c.cmd">
              <dt class="bg-panel px-4 pt-3">
                <code class="border-0 bg-transparent p-0 font-semibold text-mint">{{ c.cmd }}</code>
              </dt>
              <dd class="border-b border-line bg-panel px-4 pb-3 pt-0.5 text-[0.88rem] text-dim last:border-b-0">
                {{ c.desc }}
              </dd>
            </template>
          </dl>
          <p class="mb-4 max-w-[42rem]">
            Everything human-readable has a machine-readable twin:
            <code>status --json</code> for health checks,
            <code>monitor --json</code> for a newline-delimited event stream you can pipe into
            whatever you're building.
          </p>
        </section>

        <section id="how-it-works" class="pb-2 pt-11">
          <h2 class="mb-4 text-[1.35rem] font-semibold text-bright">
            <span class="text-mint">#</span> How it works
          </h2>
          <pre class="my-5 overflow-x-auto rounded-lg border border-line bg-panel px-5 py-4 text-[0.78rem] leading-[1.45] text-dim">
┌──────────────┐         encrypted SSH         ┌──────────────┐
│    macbook   │ ◀═══════════════════════════▶ │    fedora    │
│  pasteboard  │    persistent · deduplicated  │ wayland/x11  │
└──────────────┘         newest-wins           └──────────────┘</pre>
          <p class="mb-4 max-w-[42rem]">
            A small Rust daemon on each machine watches the system clipboard through native
            backends — the macOS pasteboard, or Wayland/X11 on Linux. When the clipboard
            changes, it ships the raw bytes of <em>every available representation</em> to its
            peers over persistent SSH connections and writes them back natively on the other
            side.
          </p>
          <p class="mb-4 max-w-[42rem]">
            There's no relay server, no cloud account, and no new cryptography — transport
            security is exactly the SSH trust you already have between your machines. Values
            are deduplicated, and per-peer queues always deliver the newest value rather than
            replaying history.
          </p>
        </section>

        <section id="updates" class="pb-2 pt-11">
          <h2 class="mb-4 text-[1.35rem] font-semibold text-bright">
            <span class="text-mint">#</span> Updates
          </h2>
          <p class="mb-4 max-w-[42rem]">
            Every daemon independently checks npm for the latest stable release and gossips
            its verified desired version to connected peers, so any online machine can trigger
            the whole mesh to converge — there is no permanent update coordinator.
          </p>
          <p class="mb-4 max-w-[42rem]">
            Packages are accepted only after the npm SHA-512 integrity hash, the bundled
            SHA-256 manifest, the executable target, and the reported binary version all
            agree. Updates keep the previous executable around, swap the live binary
            atomically, and let launchd/systemd restart the daemon.
          </p>
          <CodeBlock
            code="$ ssh-clipboard update --check
current: 0.2.0
latest:  0.2.0"
          />
        </section>

        <EarlyAccess />

        <footer class="mt-16 border-t border-line pt-6 pb-16 text-[0.85rem] text-dim">
          <p>
            <a :class="docLink" href="https://github.com/standardagents/ssh-clipboard">GitHub</a> ·
            <a :class="docLink" href="https://www.npmjs.com/package/ssh-clipboard">npm</a> ·
            <a :class="docLink" href="https://github.com/standardagents/ssh-clipboard/blob/main/LICENSE">MIT license</a>
          </p>
        </footer>
      </div>
    </div>
  </main>
</template>
