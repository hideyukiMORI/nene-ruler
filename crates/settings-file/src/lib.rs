//! `NeNe Ruler` の設定ファイル。版 1 の `key = value` の形（ADR 0006）と、OS ごとの置き場（ADR 0021）。
//!
//! 標準ライブラリだけで書き、Linux（XDG）と Windows（`%APPDATA%`）で共有する。
//! 環境変数とファイルを読むので、決定性の禁止（ARC-007）を適用しない製品モジュールの 1 つ（CNF-012 の除外）。

pub mod file_settings_store;
pub mod settings_file_v1;
