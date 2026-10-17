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
}
