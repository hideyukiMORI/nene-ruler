# NeNe Ruler

A tiny frameless screen ruler for X11. Rust, `x11rb` + `cairo`, no UI toolkit.

> **Status (2026-10-16):** Issues #2, #4 and #6. `cargo run` opens a transparent frame with ticks,
> moves when you drag a line and resizes when you drag an anchor; the gear opens a settings window for ink colour, unit and always-on-top.
> Settings are saved to `~/.config/nene-ruler/settings.v1.txt`. Copying the number is not there yet.
> Verified on Xvfb (`make screenshot`); not yet on a real desktop. Nothing to download.

## What it will be

- A transparent rectangle floating over the screen: four lines with tick marks, the width and height written on them, in px / mm / in.
- Everything except the lines, the numbers, the two bracket-shaped anchors and the gear is transparent and lets clicks through.
- Drag a line to move, drag an anchor to resize, click a number to copy it. One ink colour, chosen in the settings window.
- Always on top if you want. Settings are saved with a version (`settings.v1.txt`, plain `key = value`).
- Runs on Xorg and XWayland (WSLg included). Measures physical pixels; converts to mm through RandR.

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
