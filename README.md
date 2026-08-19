<div align="center">

# ssh-clipboard

### Your clipboard. Every machine. No cloud.

[![CI](https://github.com/standardagents/ssh-clipboard/actions/workflows/ci.yml/badge.svg)](https://github.com/standardagents/ssh-clipboard/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/ssh-clipboard?color=cb3837&logo=npm)](https://www.npmjs.com/package/ssh-clipboard)
[![MIT](https://img.shields.io/badge/license-MIT-7c3aed)](LICENSE)

```text
┌──────────────┐        encrypted SSH        ┌──────────────┐
│   your Mac   │  ◀══════════════════════▶  │  Mac / Linux │
└──────────────┘                             └──────────────┘
```

Copy here. Paste there. Text, images, files, rich content—native formats intact.

</div>

```sh
npm i -g ssh-clipboard
ssh-clipboard
```

The first-run TUI offers compatible online machines from Tailscale when it is installed, or accepts any passwordless SSH connection. It verifies each connection, inspects any existing installation, installs or upgrades only when needed, and starts a per-user background service without replacing that machine’s identity or peer configuration. After that, it just feels like one clipboard.

- **Native:** macOS pasteboard plus Linux Wayland/X11—not terminal escape tricks.
- **Private:** persistent peer-to-peer SSH; no relay, account, port, or new encryption key.
- **Faithful:** preserves every available representation, with native Finder file paste on macOS.
- **Invisible:** Raycast and other clipboard managers see ordinary system clipboard writes.
- **Fast:** raw bytes, persistent connections, deduplication, and newest-value queues.

```sh
ssh-clipboard monitor          # delightful live dashboard
ssh-clipboard status --json    # automation-friendly health
ssh-clipboard setup            # add or repair peers
ssh-clipboard update --check   # compare this node with npm @latest
```

The manual update command also reconciles the per-user service, so it can recover an installed binary whose launchd or systemd job is missing.

The monitor shows each machine on its own row with installed and target versions. Press `u` to queue an immediate npm check and notify every connected client that supports update events.

Every installed daemon independently checks the stable npm release and gossips its verified desired version to connected peers. Any online machine can therefore trigger convergence; there is no permanent update coordinator. Packages are accepted only after npm SHA-512 integrity, the bundled SHA-256 manifest, executable target, and reported binary version all agree. Updates retain the previous executable, replace the live binary atomically, and explicitly ask launchd/systemd to restart the daemon.

macOS and Linux · arm64 and x64 · Rust + [Ratatui](https://ratatui.rs)

<sub>Deep cuts: [architecture](docs/architecture.md) · [TUI design](docs/tui-design.md) · [npm distribution](docs/distribution.md)</sub>
