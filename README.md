# NeNe Ruler

A tiny frameless screen ruler for Windows and X11. Rust, no UI toolkit: Win32 + Direct2D on Windows, `x11rb` + `cairo` on Linux.

> **Status (2026-10-19):** v1.0.0 is out for Windows (Microsoft Store and GitHub) and Linux.
> The Windows build has been tried by hand on Windows 11 (frame, settings window, icon); the Linux build on Xvfb (`make screenshot`) and WSLg.
> Lengths are always physical pixels. Settings are saved to `~/.config/nene-ruler/settings.v1.txt` or `%APPDATA%\nene-ruler\settings.v1.txt`.
> The Store version uses that folder only if it already exists; otherwise Windows keeps the file in the app's package storage and removes it on uninstall (see [PRIVACY.md](PRIVACY.md)).

## Download

On Windows, from the [Microsoft Store](https://apps.microsoft.com/detail/9N6JVHKL9LM3). The Store installs and updates it.

Or from [Releases](https://github.com/hideyukiMORI/nene-ruler/releases). Each file has a `.sha256` next to it.

| File | For |
| --- | --- |
| `NeNe-Ruler-Setup.msi` | Windows 11 (x64; Windows 10 is untested). Installs to Program Files and adds a Start menu entry. Not code-signed, so SmartScreen may ask once |
| `NeNe-Ruler-windows-portable.zip` | Windows without installing. Unzip and run `nene-ruler.exe` |
| `nene-ruler_amd64.deb` | Ubuntu / Debian (amd64), WSLg included: `sudo apt install ./nene-ruler_amd64.deb` |
| `nene-ruler-linux-x86_64.zip` | Other Linux on x86_64. Needs `libcairo2` and a compositor for transparency |

## What it is

- A transparent rectangle floating over the screen: four lines with tick marks, the width and height written on them in physical pixels.
- Everything except the lines with their ticks, the numbers, the two bracket-shaped anchors and the gear is transparent and lets clicks through.
- Drag a line or its ticks to move, drag an anchor to resize, click a number to copy it. One ink colour, chosen in the settings window.
- Always on top on Windows and on window managers that support it (WSLg does not). Settings are saved with a version (`settings.v1.txt`, plain `key = value`).
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
