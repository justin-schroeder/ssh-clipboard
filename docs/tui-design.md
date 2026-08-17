# TUI design brief

This document is the handoff surface for a future Fable or human design pass. It intentionally contains no transport or daemon decisions.

## Product feeling

The interface should feel calm, private, and immediate—not like server administration. Visual language: “a small trusted mesh.” Use restrained violet/cyan accents, green only for verified/live states, rounded panels, and short language.

## Setup journey

Four persistent steps anchor the top of the screen:

```text
● WELCOME ──── ○ PEERS ──── ○ INSTALL ──── ○ READY
```

The welcome panel establishes three promises: no cloud account, no format conversion, starts at login. Peer entry accepts a whole SSH command and visibly states that password prompts are disabled. Verification becomes a focused progress state rather than freezing the form. Success shows hostname and OS/architecture, with one primary action (install) and one secondary action (add another).

Installation is a short checklist with peer name, current action, a progress bar, and a quiet performance note. Failures stay in context and offer an idempotent retry. The ready state explains that normal copy/paste and clipboard history now work; it does not invent a new interaction model.

## Monitor

The header combines the wordmark and one state pill: LIVE, PAUSED, or OFFLINE. A peer strip shows this machine, peer health, backend, and session transfer totals. Activity is a dense but readable table:

```text
TIME          FLOW                 CONTENT                 SIZE       FORMATS
14:03:18.042  ← macbookserver      design.pdf              4.0 MiB    3
14:03:13.910  ◆ copied here        hello from the Mac      18 B       2
```

Clipboard previews must remove terminal control characters and be Unicode-safe. Images/binary values fall back to a format description. `p` pauses visual ingestion, `c` clears session rows/totals, and `q` exits; the daemon is never stopped from the monitor.

## Non-negotiables

- Fully usable at 80×24; richer spacing up to 120 columns.
- Color is redundant with symbols and words.
- Bracketed paste works in the SSH command field.
- No clipboard value is ever rendered as terminal escape sequences.
- Long commands, peer names, and values truncate without breaking UTF-8.
- Async operations always show an active state and remain cancellable with Ctrl-C.
