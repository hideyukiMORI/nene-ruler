//! `NeNe Ruler` の X11 のアダプタ。`RandR` の画面の大きさと CLIPBOARD を application のポートへ渡す（ARC-003）。
//! 設定ファイルは `nene-ruler-settings-file` に切り出した（ADR 0021）。
//!
//! 決定性の禁止（ARC-007）を適用しない唯一の製品モジュール。CNF-012 の除外に明示してある。
//! ui-x11 とは別の X 接続を持つ（ADR 0002 の 1）。

pub mod adapter_failure;
pub mod randr_screen_metrics;
pub mod selection_answer;
pub mod selection_atoms;
pub mod x11_clipboard;
