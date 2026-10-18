//! `NeNe Ruler` の Windows のアダプタ。主モニタの大きさと dpi・クリップボードを application のポートへ渡す（ARC-003・ADR 0021）。
//!
//! 決定性の禁止（ARC-007）を適用しない製品モジュールの 1 つ（CNF-012 の除外）。
//! `unsafe` を書ける 2 クレートの 1 つで、Win32 を呼ぶ関数ごとに `#[expect(unsafe_code)]` と SAFETY の注釈を置く。
//! 中身は Windows でだけ組み立てる。

#[cfg(windows)]
pub mod windows_clipboard;
#[cfg(windows)]
pub mod windows_screen_metrics;
