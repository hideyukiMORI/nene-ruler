# NeNe Ruler

A tiny frameless screen ruler for X11. Rust, `x11rb` + `cairo`, no UI toolkit.

> **Status (2026-10-16):** Issues #2, #4, #6, #8, #10 and #12. `cargo run` opens a transparent frame with ticks,
> moves when you drag a line and resizes when you drag an anchor; the gear opens a settings window for ink colour, theme and always-on-top.
> Settings are saved to `~/.config/nene-ruler/settings.v1.txt`; clicking a number copies it to the clipboard.
> Arrow keys nudge the frame by 1 px (10 px with Shift). Lengths are always physical pixels (mm / in were dropped). With no compositor the frame falls back to opaque and says so.
> HiDPI is 1x or 2x, from `Xft.dpi` or the RandR dpi.
> Verified on Xvfb (`make screenshot`) and once on WSLg (see `docs/quality/wslg-2026-10-17.md`). Nothing to download.

## What it will be

- A transparent rectangle floating over the screen: four lines with tick marks, the width and height written on them in physical pixels.
- Everything except the lines, the numbers, the two bracket-shaped anchors and the gear is transparent and lets clicks through.
- Drag a line to move, drag an anchor to resize, click a number to copy it. One ink colour, chosen in the settings window.
- Always on top if the window manager supports it (WSLg does not; a native Windows build is planned). Settings are saved with a version (`settings.v1.txt`, plain `key = value`).
- Runs on Xorg and XWayland (WSLg included). Measures physical pixels.

The requirements are in [SPECIFICATION.md](SPECIFICATION.md) (Japanese).

## How it is built

This repository follows the AYANE strict policy: **one meaning, one canonical implementation path, enforced by machines.**
It is the fourth application of that policy and the second in Rust (after xi-tools).

- `docs/ARCHITECTURE_CONSTITUTION.md` — the rules (ARC-NNN) and how far Rust reaches for each of them
- `docs/CODING_RULES.md` — the approved subset of Rust (RS-NNN); suppressions are two-tiered (`forbid` cannot be overridden, `deny` needs `#[expect(..., reason = "<rule>: ...")]`)
- `docs/QUALITY_GATES.md` — what is enforced right now (active / planned / impossible), the only source of truth
- `docs/quality/gate-proofs.md` — real tool output for every gate that claims to be active
- `docs/adr/` — decisions, including why `x11rb` + `cairo` and no toolkit

Verification is chosen per change (`make lint`, `make test`, `make conformance`, ...). The full run needs a reason:

```bash
make check REASON='why a narrower check cannot cover this change'
```

It needs the toolchain pinned in `rust-toolchain.toml` (rustup installs it), `cargo-deny`, and `libcairo2-dev` for the UI crate.
`make doctor` lists what is missing.

## License

MIT. See [LICENSE](LICENSE).
