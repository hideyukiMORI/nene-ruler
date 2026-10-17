//! `NeNe Ruler` の画面。x11rb で透明な窓を持ち、cairo の `ImageSurface` に描いて `PutImage` で転送する。
//!
//! application が作った `RulerFrame` を描き、利用者の操作を意図として渡すだけ（ARC-011）。
//! 可変性の隔離区画（ARC-005）。業務状態は複製しない。
//! 時刻・環境・ファイルには到達しない（ARC-007 / CNF-012）。

pub mod alt_f4_gesture;
pub mod atoms;
pub mod canvas;
pub mod display;
pub mod drag;
pub mod event_loop;
mod flow;
pub mod key_input;
pub mod key_map;
pub mod pointer_shapes;
pub mod ruler_window;
pub mod settings_canvas;
mod settings_flow;
pub mod settings_window;
mod transfer;
pub mod ui_failure;
mod undecorated;
pub mod window_geometry;
pub mod window_visual;
