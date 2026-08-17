# ssh-clipboard

Native clipboard sync for macOS and Linux, carried over persistent passwordless SSH connections.

Run `ssh-clipboard` once, enter the SSH commands you already use, and the setup TUI verifies every connection, installs the correct binary on each peer, and configures a per-user service. After that, copy and paste normally. The destination system clipboard is updated through its native API, so Raycast and other clipboard managers observe the change naturally.

## Install

```sh
npm i -g ssh-clipboard
ssh-clipboard
```

The npm package includes native arm64 and x64 binaries for both macOS and Linux. Its tiny Node launcher selects the correct local binary, then gives the Rust setup process access to every other target so it can install mixed-OS peers over SSH. There is no binary download or JavaScript clipboard implementation at runtime.

## What is preserved

`ssh-clipboard` enumerates the native clipboard rather than reducing it to text or PNG. One clipboard event can carry multiple original representations, including:

- plain text, HTML, RTF, and application-specific rich text;
- PNG, TIFF, JPEG, HEIC, SVG, PDF, and other image/document formats;
- native file URL lists and the underlying contents of copied files/directories;
- arbitrary custom MIME types or macOS pasteboard types.

Portable aliases are additive. For example, `public.tiff` remains intact and may also be offered as `image/tiff` on Linux. Data is framed as raw bytes—never Base64—on the SSH stream.

## Build from source

Requirements: Rust 1.88+ and OpenSSH. Linux needs a desktop X11 or Wayland session; the Wayland path uses the compositor’s data-control protocol and falls back to X11 when available.

```sh
git clone https://github.com/justin-schroeder/ssh-clipboard.git
cd ssh-clipboard
cargo build --release
./target/release/ssh-clipboard
```

The first-run setup requires SSH public-key authentication. Password and keyboard-interactive prompts are explicitly disabled during verification.

For a source-built copy, cross-platform peer installation requires a release bundle containing these sibling binaries:

```text
ssh-clipboard-darwin-arm64
ssh-clipboard-darwin-amd64
ssh-clipboard-linux-arm64
ssh-clipboard-linux-amd64
```

When the peer matches the current machine’s OS/architecture, the running binary is uploaded directly.

## Commands

```text
ssh-clipboard                  first-run setup, then live dashboard
ssh-clipboard setup            add, verify, and install peers
ssh-clipboard monitor          Ratatui activity dashboard
ssh-clipboard monitor --plain  readable event stream
ssh-clipboard monitor --json   newline-delimited JSON events
ssh-clipboard status [--json]  daemon and connection health
ssh-clipboard service install  install the per-user service
ssh-clipboard service restart  restart it
```

The setup installs:

- `~/.local/bin/ssh-clipboard`;
- `~/Library/LaunchAgents/dev.ssh-clipboard.plist` on macOS; or
- `~/.config/systemd/user/ssh-clipboard.service` on Linux.

Configuration lives at `~/.config/ssh-clipboard/config.json`. Runtime state and received copied files live under `~/.local/state/ssh-clipboard`.

## Security and privacy

- Every network byte travels inside authenticated SSH. There is no cloud relay, discovery service, listening TCP port, or separate encryption key to manage.
- The daemon’s local Unix socket is mode `0600`; configuration is written mode `0600`.
- SSH is forced into batch mode with password, keyboard-interactive auth, TTY allocation, and forwarding disabled.
- Clipboards marked concealed/transient by common password managers are excluded as a whole.
- Incoming file bundles reject absolute paths, `..`, symlinks, and trailing/unaccounted data before publishing the destination clipboard.
- Clipboard values are not retained as app history. Copied files must remain on disk so native clipboard managers can paste them later; they are materialized under the state directory by clip UUID.

Peer connections are direct and full-duplex. With several configured peers, the setup machine is the hub for that group: an update received from one SSH stream is deduplicated and relayed to the others without a cloud service.

## Performance

- SSH processes are persistent; a copy does not create a new connection.
- The protocol uses a small JSON metadata header followed by contiguous raw representation bodies.
- Peer queues are newest-value channels: if a huge clipboard is still sending, stale intermediate copies do not build up.
- macOS and X11 use lightweight native change detection before reading large data. Wayland snapshots at the configured interval to correctly detect same-MIME image changes.
- The default payload limit is 256 MiB and is configurable.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release
npm ci
npm test
```

The suite covers raw multi-format framing, size limits, SSH parsing and probe responses, configuration permissions, relay/apply behavior, loop safety primitives, password-manager exclusions, additive aliases, copied-file round trips and path-traversal rejection, service generation, and Ratatui render snapshots.

See [architecture](docs/architecture.md), the [TUI design brief](docs/tui-design.md), and the [npm distribution guide](docs/distribution.md).
