//! `NeNe Ruler` の Windows の画面。Win32 の透明な窓（`WS_EX_LAYERED`）に Direct2D / DirectWrite で描き、
//! `UpdateLayeredWindow` で出す（ADR 0021）。状態を描き、意図を発行する（ARC-011）。
//!
//! 置き場と当たり判定は core の `FrameLayout` から出す（ARC-001）。描く手続きだけが X11 版と別になる。
//! `unsafe` を書ける 2 クレートの 1 つ。中身は Windows でだけ組み立てる。

#[cfg(windows)]
pub mod frame_painter;
#[cfg(windows)]
pub mod frame_session;
#[cfg(windows)]
pub mod frame_window;
#[cfg(windows)]
pub mod layered_bitmap;
pub mod non_client_hit;
#[cfg(windows)]
pub mod win32_failure;
