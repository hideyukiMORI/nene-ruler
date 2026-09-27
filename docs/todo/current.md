# いまのタスク — NeNe Ruler

> GitHub Issue と PR が進行状態の正本。この文書は 2026-09-27 の初期化（Issue #1）完了時点の要約。

## 状態

| | |
| --- | --- |
| 言語と土台 | Rust 1.98.0 / x11rb ＋ cairo-rs（ADR 0002）。施主決定 2026-09-27 |
| 規範文書・ADR 0001〜0004 | ✅ Issue #1 |
| ゲート（clippy 二段構え・xtask・cargo-deny・rustfmt・rustdoc）と `make` の個別 target | ✅ Issue #1 |
| ゲート発火の証明 `make prove`（26 件）と `cargo test -p xtask`（68 件） | ✅ Issue #1・[gate-proofs](../quality/gate-proofs.md) |
| CI の必須 check（`make ci`）と commit-msg フック | 配線済み。最初の PR で実行証拠を得るまで planned |
| ruleset（main の保護・squash-only） | 未設定。施主が GitHub で設定したら gate-proofs に記録する |
| 製品コード | 無し。`crates/app` は起動できない理由を返す stub |

## 次に行うこと

[Issue #2](https://github.com/hideyukiMORI/nene-ruler/issues/2) — 最初の縦切り。
`crates/core` / `crates/application` / `crates/adapters-x11` / `crates/ui-x11` を作り、枠なしの窓に目盛と長さを描いて Xvfb で撮る（FR-001 / FR-003 / FR-004 / FR-016）。
依存 `x11rb` / `cairo-rs` を足し、`deny.toml` の `allow` を実測で更新する。ARC-003 / QLT-009 / CNF-007 の実ファイル発火を記録して active にする。

## 残る穴

- CNF-012 はマクロ・別名・間接呼び出しを見ない。ui-x11 は `std` を使うので字句検査だけが残る
- CNF-005（不変条件を持つ型の単独モジュール）は判定方法が未定
- `PutImage` の速さと WSLg での見え方は未実測（ADR 0002）
