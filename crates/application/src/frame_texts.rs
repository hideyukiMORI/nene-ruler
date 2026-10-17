//! 幅・高さ・両方の表示文字列の組。

use nene_ruler_core::length_label::LengthLabel;
use nene_ruler_core::length_text::LengthText;

/// 数字の種類ごとの表示文字列。表示とコピーで同じもの（FR-007）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameTexts {
    width: LengthText,
    height: LengthText,
    size: LengthText,
}

impl FrameTexts {
    /// 幅・高さ・両方（`W × H px`）の文字列から作る。
    #[must_use]
    pub const fn new(width: LengthText, height: LengthText, size: LengthText) -> Self {
        Self {
            width,
            height,
            size,
        }
    }

    /// 種類ごとの文字列。
    #[must_use]
    pub const fn text(&self, label: LengthLabel) -> &LengthText {
        match label {
            LengthLabel::Width => &self.width,
            LengthLabel::Height => &self.height,
            LengthLabel::Size => &self.size,
        }
    }
}
