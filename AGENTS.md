# NeNe Ruler — Agent Guide

Short English entry point. The authoritative handbook is [CLAUDE.md](CLAUDE.md) (Japanese);
the normative documents live in [docs/](docs/).

## Project identity

- Product: NeNe Ruler — a tiny frameless screen ruler for X11 (Rust, `x11rb` + `cairo-rs`, no UI toolkit)
- Language / UI: Rust 1.98.0, edition 2024 / none (plain X11 through `x11rb`, pixels through cairo `ImageSurface`)
- Workspace: `crates/{core,application,adapters-x11,ui-x11,app}` + `xtask`
- Governing principle: **one meaning, one canonical implementation path, enforced by machines**

## Required reading before changing production code

0. `SPECIFICATION.md`
1. `docs/ARCHITECTURE_CONSTITUTION.md`
2. `docs/PROJECT_LAYOUT.md`
3. `docs/CODING_RULES.md`
4. `docs/QUALITY_GATES.md`
5. `docs/DEVELOPMENT_WORKFLOW.md`
6. `docs/COMMIT_CONVENTIONS.md`
7. `docs/GLOSSARY.md`

Then the active issue and the relevant accepted ADRs.

## Agent rules

- Do not invent a second implementation path because it is locally convenient.
- Do not weaken a gate to make a change pass. Fix the code instead. `forbid` lints cannot be suppressed; `deny` lints only with `#[expect(lint, reason = "<rule id>: ...")]`.
- Do not mark a rule `active` in `docs/QUALITY_GATES.md` before its enforcement exists and `docs/quality/gate-proofs.md` has its row.
- Do not read the current time, environment, files, or threads outside `crates/adapters-x11`. Core crates are `#![no_std]`.
- Do not write `_` in a `match` over a closed enum; it disables exhaustiveness checking.
- Do not add dependencies without an ADR and a measured `deny.toml` update. Core and `xtask` have zero dependencies.
- Do not claim a command passed unless it was actually executed.
- Prefer the smallest change that fully follows the canonical path.

## Verification

Pick the narrowest checks the diff needs (`make lint`, `make test`, `make conformance`, `make deny`, `make doc-check`).
`make check REASON='...'` runs everything and refuses an empty reason. `make ci` is what the required GitHub check runs.
`make prove` re-measures that the gates fire; run it whenever a gate changes.

## Required completion report

Issue and rule IDs, files and behavior changed, verification commands and results (and where the record lives),
documentation or schema changes, `#[expect]` suppressions added (or `none`), remaining risks.

Investigation-only requests do not authorize editing, committing, pushing, or opening PRs.
