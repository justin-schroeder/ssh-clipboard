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
    text: 'Writes the real system clipboard: NSPasteboard on macOS, Wayland or X11 on Linux.',
  },
  {
    name: 'private',
    text: 'Persistent peer-to-peer SSH. No relay, no account, no open port, no new encryption key.',
  },
  {
    name: 'faithful',
    text: 'Ships every clipboard representation: rich text, images, files.',
  },
  {
    name: 'invisible',
    text: 'Raycast and other clipboard managers see ordinary clipboard writes. Nothing to configure.',
  },
  {
    name: 'fast',
    text: 'Sends raw bytes over persistent connections through deduplicated, newest-value queues.',
  },
  {
    name: 'self-updating',
    text: 'Daemons gossip verified versions and converge on the latest stable release. No central coordinator.',
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
  <header class="sticky top-0 z-10 border-b border-line bg-bg">
    <div class="mx-auto flex w-[min(90rem,calc(100vw-7rem))] items-center justify-between gap-4 py-4 max-md:w-[calc(100vw-2.5rem)] max-md:py-3">
      <a class="whitespace-nowrap font-semibold text-bright" href="#top">
        ssh-clipboard<span class="text-mint">_</span>
      </a>
      <nav class="flex items-center gap-9 text-[0.9rem] max-md:gap-5 max-md:text-[0.8rem]">
        <a class="text-ink transition-colors hover:text-mint max-[560px]:hidden" href="#install">install</a>
        <a class="text-ink transition-colors hover:text-mint max-[560px]:hidden" href="#commands">commands</a>
        <a class="text-ink transition-colors hover:text-mint max-[560px]:hidden" href="#how-it-works">how&nbsp;it&nbsp;works</a>
        <StarButton />
      </nav>
    </div>
  </header>

  <main id="top">
    <!-- ── hero ─────────────────────────────── -->
    <section class="relative overflow-x-clip px-5 pb-24 pt-32 text-left max-xl:text-center max-md:pt-16">
      <div
        class="mx-auto grid w-[min(90rem,calc(100vw-7rem))] grid-cols-[minmax(21rem,27rem)_minmax(0,1fr)] items-center gap-16 max-xl:grid-cols-1 max-md:gap-8 max-md:w-[calc(100vw-2.5rem)]"
      >
        <div>
          <h1
            class="mb-8 text-[clamp(2.8rem,5.2vw,4.1rem)] font-semibold leading-[1.12] tracking-[-0.03em] text-bright"
          >
            <span>copy/paste,</span><br />
            <span class="text-mint">everywhere.</span>
          </h1>
          <p class="mb-10 max-w-[38rem] text-[1.05rem] leading-[1.85] text-dim max-xl:mx-auto">
            Peer-to-peer clipboard sync over SSH.
            Copy anything on one machine, paste on another.
            <span class="text-mint">Written in Rust ackchyually.</span>
          </p>
          <InstallPill />
        </div>
        <div class="relative isolate min-w-0">
          <div
            class="dither pointer-events-none absolute -inset-x-48 -inset-y-28 -z-10 text-mint opacity-[0.13] [mask-image:radial-gradient(70%_78%_at_50%_46%,black,transparent_82%)] max-md:-inset-x-3"
          ></div>
          <MeshDemo />
        </div>
      </div>

    </section>

    <!-- ── features ─────────────────────────── -->
    <section
      class="mx-auto grid w-[min(90rem,calc(100vw-7rem))] grid-cols-[repeat(auto-fit,minmax(16rem,1fr))] gap-x-16 gap-y-16 pb-8 pt-24 max-md:w-[calc(100vw-2.5rem)] lg:grid-cols-3"
    >
      <div v-for="f in features" :key="f.name">
        <h3 class="mb-2 text-[0.95rem] font-semibold text-bright">
          <span class="text-mint">*</span> {{ f.name }}
        </h3>
        <p class="pl-4 text-[0.85rem] leading-[1.7] text-dim">{{ f.text }}</p>
      </div>
    </section>

    <!-- ── docs with jump-link sidebar ──────── -->
    <div
      class="mx-auto mt-28 grid w-[min(90rem,calc(100vw-7rem))] grid-cols-[13rem_minmax(0,1fr)] gap-14 max-md:w-[calc(100vw-2.5rem)] max-[1150px]:grid-cols-[minmax(0,1fr)]"
    >
      <aside
        class="sticky top-[6.5rem] mt-[5.25rem] flex flex-col gap-1.5 self-start text-[0.95rem] max-[1150px]:hidden"
        aria-label="On this page"
      >
        <a
          v-for="s in sections"
          :key="s.id"
          :href="'#' + s.id"
          class="w-fit whitespace-pre py-1 transition-colors"
          :class="activeId === s.id ? 'text-mint' : 'text-dim hover:text-bright'"
        >{{ (activeId === s.id ? '▌ ' : '  ') + s.title }}</a>
      </aside>

      <div class="mx-auto w-full min-w-0 max-w-[46rem]">
        <section id="install" class="pb-6 pt-20">
          <h2 class="mb-6 text-[1.4rem] font-semibold text-bright">
            <span class="text-mint">#</span> Install
          </h2>
          <p class="mb-4 max-w-[42rem]">
            The npm package installs a native Rust binary and a per-user service
            (<code>launchd</code> on macOS, <code>systemd</code> on Linux).
          </p>
          <CodeBlock
            code="$ npm i -g ssh-clipboard
$ ssh-clipboard"
          />
          <p class="mb-4 max-w-[42rem]">You'll need:</p>
          <ul class="mb-4 ml-6 list-disc space-y-1.5">
            <li>macOS or Linux (Wayland or X11), on arm64 or x64</li>
            <li>Node ≥ 18 for <code>npm install</code>; the daemon has no Node dependency</li>
            <li>
              <a :class="docLink" href="https://tailscale.com/kb/1193/tailscale-ssh">Tailscale SSH</a>
              is recommended but passwordless <code>~/.ssh</code> keys work too
            </li>
          </ul>
        </section>

        <section id="quick-start" class="pb-6 pt-20">
          <h2 class="mb-6 text-[1.4rem] font-semibold text-bright">
            <span class="text-mint">#</span> Quick start
          </h2>
          <p class="mb-4 max-w-[42rem]">
            Run <code>ssh-clipboard</code> with no arguments. The first-run TUI lists compatible
            online Tailscale machines, or accepts any passwordless SSH destination.
            For each peer it:
          </p>
          <ol class="mb-4 ml-6 list-decimal space-y-1.5">
            <li>verifies the connection,</li>
            <li>installs the right binary over SSH,</li>
            <li>starts the per-user service on both ends.</li>
          </ol>
          <p class="mb-4 max-w-[42rem]">
            Copy on one machine, paste on another. After setup it behaves like one clipboard.
          </p>
          <CodeBlock
            code="$ ssh-clipboard status
running as macbook (node-a1, pasteboard, version 0.2.0)
connected: fedora
connected: macbookserver"
          />
        </section>

        <section id="commands" class="pb-6 pt-20">
          <h2 class="mb-6 text-[1.4rem] font-semibold text-bright">
            <span class="text-mint">#</span> Commands
          </h2>
          <dl class="my-8 overflow-hidden border border-line">
            <template v-for="(c, i) in commands" :key="c.cmd">
              <dt class="bg-panel px-5 pt-4">
                <code class="border-0 bg-transparent p-0 font-semibold">
                  <span :class="i === 0 ? 'text-mint' : 'text-bright'">ssh-clipboard</span><span
                    class="text-mint"
                  >{{ c.cmd.slice('ssh-clipboard'.length) }}</span>
                </code>
              </dt>
              <dd class="border-b border-line bg-panel px-5 pb-4 pt-1 text-[0.88rem] text-dim last:border-b-0">
                {{ c.desc }}
              </dd>
            </template>
          </dl>
          <p class="mb-4 max-w-[42rem]">
            Every command has a machine-readable twin:
            <code>status --json</code> for health checks,
            <code>monitor --json</code> for an NDJSON event stream you can pipe anywhere.
          </p>
        </section>

        <section id="how-it-works" class="pb-6 pt-20">
          <h2 class="mb-6 text-[1.4rem] font-semibold text-bright">
            <span class="text-mint">#</span> How it works
          </h2>
          <pre class="my-8 overflow-x-auto border border-line bg-panel px-6 py-5 text-[0.78rem] leading-[1.45] text-dim">
┌──────────────┐         encrypted SSH         ┌──────────────┐
│    macbook   │ ◀═══════════════════════════▶ │    fedora    │
│  pasteboard  │    persistent · deduplicated  │ wayland/x11  │
└──────────────┘         newest-wins           └──────────────┘</pre>
          <p class="mb-4 max-w-[42rem]">
            A small Rust daemon on each machine watches the system clipboard through
            native backends. On change, it ships the raw bytes of
            <em>every representation</em> to its peers over persistent SSH and writes
            them back natively.
          </p>
          <p class="mb-4 max-w-[42rem]">
            No relay, cloud account, additional port forwarding required. Values are
            deduplicated, and per-peer queues always deliver the newest value.
          </p>
        </section>

        <section id="updates" class="pb-6 pt-20">
          <h2 class="mb-6 text-[1.4rem] font-semibold text-bright">
            <span class="text-mint">#</span> Updates
          </h2>
          <p class="mb-4 max-w-[42rem]">
            Each daemon checks npm for the latest stable release and tells its peers
            what it found, so any machine that's online can update the whole mesh.
            There's no central update server.
          </p>
          <p class="mb-4 max-w-[42rem]">
            Before installing anything, a daemon verifies the npm SHA-512 hash, the
            bundled SHA-256 manifest, the executable target, and the version the binary
            actually reports. Updates keep the old executable around, swap the new one
            in atomically, and let launchd or systemd restart the daemon.
          </p>
          <CodeBlock
            code="$ ssh-clipboard update --check
current: 0.2.0
latest:  0.2.0"
          />
        </section>

        <EarlyAccess />
      </div>
    </div>

    <footer class="mt-24 border-t border-line/40 pt-8 pb-24 text-[0.85rem] text-dim">
      <div
        class="mx-auto flex w-[min(90rem,calc(100vw-7rem))] flex-wrap items-baseline justify-between gap-4 max-md:w-[calc(100vw-2.5rem)]"
      >
        <p>Built by Standard Agents. Open Source under MIT.</p>
        <p>
          <a :class="docLink" href="https://github.com/standardagents/ssh-clipboard">GitHub</a> ·
          <a :class="docLink" href="https://www.npmjs.com/package/ssh-clipboard">npm</a> ·
          <a :class="docLink" href="https://github.com/standardagents/ssh-clipboard/blob/main/LICENSE">MIT license</a>
        </p>
      </div>
    </footer>
  </main>
</template>
