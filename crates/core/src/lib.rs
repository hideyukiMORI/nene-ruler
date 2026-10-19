//! `NeNe Ruler` の中核。値・不変条件・閉じた選択肢・整形・目盛と当たり判定の計算（ARC-003）。
//!
//! 🔴 `#![no_std]` である。X11・cairo・ファイル・時刻・環境変数には構文的に到達できない。
//! 外部クレートへの依存は 0（ARC-013 / CNF-013）。
//!
//! 幾何はすべて物理画素で、窓の左上を原点とする `u16` の座標で表す（FR-016 / ADR 0005）。
//! X の矩形（`i16` / `u16`）にも cairo の `f64` にも `From` / `TryFrom` で写せる（RS-007）。

#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub mod anchor;
pub mod app_icon;
pub mod axis;
pub mod device_pixel_ratio;
pub mod edge;
pub mod frame_detail;
pub mod frame_layout;
pub mod frame_size;
pub mod hex_entry;
pub mod hit_region;
pub mod hit_target;
pub mod hsv;
pub mod icon_file;
pub mod length;
pub mod length_failure;
pub mod length_label;
pub mod length_text;
pub mod measure;
pub mod modal_label;
pub mod modal_palette;
pub mod modal_theme;
pub mod nudge;
pub mod nudge_direction;
pub mod pixel_point;
pub mod pixel_rect;
pub mod png_file;
pub mod rgb;
pub mod rgb_failure;
pub mod ruler_settings;
pub mod screen_metrics;
pub mod screen_metrics_failure;
pub mod settings_control;
pub mod settings_hit;
pub mod settings_layout;
pub mod tick_kind;
pub mod tick_label;
pub mod tick_mark;
pub mod ui_scale;
pub mod unit;
pub mod window_layer;
