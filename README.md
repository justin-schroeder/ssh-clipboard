<div align="center">

# ssh-clipboard

### Your clipboard. Every machine. No cloud.

[![CI](https://github.com/justin-schroeder/ssh-clipboard/actions/workflows/ci.yml/badge.svg)](https://github.com/justin-schroeder/ssh-clipboard/actions/workflows/ci.yml)
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

The first-run TUI verifies your passwordless SSH connections, installs the right native Rust binary on every peer, and starts a per-user background service. After that, it just feels like one clipboard.

- **Native:** macOS pasteboard plus Linux Wayland/X11—not terminal escape tricks.
- **Private:** persistent peer-to-peer SSH; no relay, account, port, or new encryption key.
- **Faithful:** preserves every available representation, not only text or PNG.
- **Invisible:** Raycast and other clipboard managers see ordinary system clipboard writes.
- **Fast:** raw bytes, persistent connections, deduplication, and newest-value queues.

```sh
ssh-clipboard monitor          # delightful live dashboard
ssh-clipboard status --json    # automation-friendly health
ssh-clipboard setup            # add or repair peers
```

macOS and Linux · arm64 and x64 · Rust + [Ratatui](https://ratatui.rs)

<sub>Deep cuts: [architecture](docs/architecture.md) · [TUI design](docs/tui-design.md) · [npm distribution](docs/distribution.md)</sub>
