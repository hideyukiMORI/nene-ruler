//! 幅・高さ・両方の表示文字列の組。

use nene_ruler_core::length_label::LengthLabel;
use nene_ruler_core::length_text::LengthText;

/// 数字の種類ごとの表示文字列。表示とコピーで同じもの（FR-007）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameTexts {
    width: LengthText,
    height: LengthText,
    size: LengthText,
    short_size: LengthText,
}

impl FrameTexts {
    /// 幅・高さ・両方（`W × H px`）・両方の短い形（`W×H`）の文字列から作る。
    #[must_use]
    pub fn new(
        width: LengthText,
        height: LengthText,
        (size, short_size): (LengthText, LengthText),
    ) -> Self {
        Self {
            width,
            height,
            size,
            short_size,
        }
    }

    /// 両方の短い形。線を消した表示で描く（ADR 0019）。コピーには使わない。
    #[must_use]
    pub const fn short_size(&self) -> &LengthText {
        &self.short_size
    }

    /// 種類ごとの文字列。コピーするのはいつもこれ（FR-007）。
    #[must_use]
    pub const fn text(&self, label: LengthLabel) -> &LengthText {
        match label {
            LengthLabel::Width => &self.width,
            LengthLabel::Height => &self.height,
            LengthLabel::Size => &self.size,
        }
    }
}
