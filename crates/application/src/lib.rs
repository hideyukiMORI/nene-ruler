//! `NeNe Ruler` の振る舞いの調整。ポート（`trait`）・状態の所有者・表示フレーム（ARC-003 / ARC-004）。
//!
//! 🔴 `#![no_std]` である。X11・永続化・ファイル・時刻を知らない。プラットフォームはポートから入る。

#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub mod frame_texts;
pub mod modal_session;
pub mod ports;
pub mod product_version;
pub mod ruler_controller;
pub mod ruler_frame;
pub mod ruler_intent;
pub mod screen_metrics_port;
pub mod settings_frame;
pub mod settings_store_failure;
pub mod settings_store_port;
pub mod transparency;
