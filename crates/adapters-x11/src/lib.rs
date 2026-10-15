//! `NeNe Ruler` のアダプタ。`RandR` の物理寸法を application のポートへ渡す（ARC-003）。
//!
//! 決定性の禁止（ARC-007）を適用しない唯一の製品モジュール。CNF-012 の除外に明示してある。
//! ui-x11 とは別の X 接続を持つ（ADR 0002 の 1）。

pub mod adapter_failure;
pub mod randr_screen_metrics;
