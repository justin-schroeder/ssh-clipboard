# Architecture

```mermaid
flowchart LR
    A["Native clipboard<br/>macOS / X11 / Wayland"] --> B["Local daemon<br/>snapshot + dedupe"]
    B --> C["Persistent ssh process<br/>raw framed stream"]
    C --> D["Remote bridge command<br/>Unix socket only"]
    D --> E["Remote daemon<br/>atomic native publish"]
    E --> F["Native clipboard manager<br/>Raycast / desktop history"]
    E --> C
```

Each daemon owns a mode-`0600` Unix socket. `ssh-clipboard bridge` contains no clipboard or network logic; it copies stdin/stdout to that socket. The initiating daemon starts one persistent OpenSSH child per configured peer and speaks the same binary protocol in both directions.

## Clipboard pipeline

1. A platform backend enumerates every offered format and reads its raw bytes.
2. Sensitive clipboard markers exclude the entire value.
3. Common macOS types gain additive portable aliases.
4. File URL lists gain a private file bundle containing regular files/directories.
5. A SHA-256 fingerprint detects unchanged clipboard snapshots.
6. The event receives a UUID and is sent over every peer’s newest-value channel.
7. The receiver deduplicates the UUID, relays it to other peers, materializes file bundles, and atomically publishes all safe representations.
8. The hash of the clipboard actually published by the OS suppresses the watcher echo.

The protocol has an eight-byte prefix (`SCB1` plus a big-endian header length), a bounded JSON header, and raw representation bodies. Both header and aggregate body sizes are bounded before allocation.

## Service model

macOS runs a user LaunchAgent in the GUI session so `NSPasteboard` is available. Linux runs a user systemd service and imports the desktop display/session environment. No root service or privileged install is used.

## Multi-peer behavior

An SSH stream is intrinsically bidirectional, so the remote does not need to SSH back. The machine whose configuration lists several peers relays between those direct streams. UUID deduplication prevents cycles; newest-value channels bound memory during bursts.
