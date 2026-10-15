//! 幅と高さの表示文字列の組。

use nene_ruler_core::axis::Axis;
use nene_ruler_core::length_text::LengthText;

/// 軸ごとの表示文字列。表示とコピーで同じもの（FR-007）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameTexts {
    width: LengthText,
    height: LengthText,
}

impl FrameTexts {
    /// 幅と高さの文字列から作る。
    #[must_use]
    pub const fn new(width: LengthText, height: LengthText) -> Self {
        Self { width, height }
    }

    /// 軸ごとの文字列。
    #[must_use]
    pub const fn along(&self, axis: Axis) -> &LengthText {
        match axis {
            Axis::Horizontal => &self.width,
            Axis::Vertical => &self.height,
        }
    }
}
