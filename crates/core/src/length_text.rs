//! 長さを綴った文字列。表示とクリップボードで同じものを使う（FR-004 / FR-007）。

use alloc::format;
use alloc::string::String;

use crate::length::Length;

/// `400 px` の形。単位は物理画素だけ（ADR 0012）。整形はここだけが行う（ARC-001）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LengthText(String);

impl LengthText {
    /// 物理画素で綴る。
    #[must_use]
    pub fn in_pixels(length: Length) -> Self {
        Self(format!("{} px", length.px()))
    }

    /// 幅と高さを 1 行で綴る（`100 × 20 px`）。詰めた表示で使う（ADR 0017）。
    #[must_use]
    pub fn size(width: Length, height: Length) -> Self {
        Self(format!("{} \u{d7} {} px", width.px(), height.px()))
    }

    /// 文字列そのもの。表示にもコピーにもこれを使う。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 文字数。当たり判定の幅の見積りに使う。
    #[must_use]
    pub fn chars(&self) -> u16 {
        u16::try_from(self.0.chars().count()).unwrap_or(u16::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spells_pixels_plainly() {
        let text = LengthText::in_pixels(Length::INITIAL_WIDTH);
        assert_eq!(text.as_str(), "400 px");
        assert_eq!(text.chars(), 6);
    }

    #[test]
    fn spells_both_lengths_on_one_line() {
        let text = LengthText::size(Length::new(100).unwrap(), Length::new(40).unwrap());
        assert_eq!(text.as_str(), "100 \u{d7} 40 px");
        assert_eq!(text.chars(), 11);
    }
}
