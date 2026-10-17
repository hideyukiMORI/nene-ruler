//! 描くための値の組。最新の状態から生成する（用語集「表示フレーム」）。

use alloc::vec::Vec;

use nene_ruler_core::axis::Axis;
use nene_ruler_core::frame_layout::FrameLayout;
use nene_ruler_core::hit_region::HitRegion;

use nene_ruler_core::length_text::LengthText;
use nene_ruler_core::rgb::Rgb;
use nene_ruler_core::window_layer::WindowLayer;

use crate::copy_mark::CopyMark;
use crate::frame_notices::FrameNotices;
use crate::frame_style::FrameStyle;
use crate::frame_texts::FrameTexts;
use crate::transparency::Transparency;

/// UI が読むすべて。UI はこれを描いて意図を返すだけ（ARC-011）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulerFrame {
    layout: FrameLayout,
    texts: FrameTexts,
    style: FrameStyle,
}

impl RulerFrame {
    /// 値を束ねる。生成は `RulerController::frame` だけが行う。
    #[must_use]
    pub const fn new(layout: FrameLayout, texts: FrameTexts, style: FrameStyle) -> Self {
        Self {
            layout,
            texts,
            style,
        }
    }

    /// 幾何。
    #[must_use]
    pub const fn layout(&self) -> FrameLayout {
        self.layout
    }

    /// 軸ごとの表示文字列。表示とコピーで同じもの（FR-007）。
    #[must_use]
    pub const fn text(&self, axis: Axis) -> &LengthText {
        self.texts.along(axis)
    }

    /// インク色。
    #[must_use]
    pub const fn ink(&self) -> Rgb {
        self.style.ink()
    }

    /// 重なり順。窓の属性であって画素ではないが、UI が読む値なのでここに乗せる。
    #[must_use]
    pub const fn layer(&self) -> WindowLayer {
        self.style.layer()
    }

    /// 透明に描けるか。
    #[must_use]
    pub const fn transparency(&self) -> Transparency {
        self.style.transparency()
    }

    /// コピーの印。ポインタが数字から離れるまで（FR-007）。
    #[must_use]
    pub const fn copy_mark(&self) -> Option<CopyMark> {
        self.style.copy_mark()
    }

    /// 枠に出す通知（FR-014）。
    #[must_use]
    pub const fn notices(&self) -> FrameNotices {
        self.style.notices()
    }

    /// 当たり判定。文字数は表示文字列から取る。
    #[must_use]
    pub fn hit_regions(&self) -> Vec<HitRegion> {
        self.layout.hit_regions(
            self.text(Axis::Horizontal).chars(),
            self.text(Axis::Vertical).chars(),
        )
    }
}
