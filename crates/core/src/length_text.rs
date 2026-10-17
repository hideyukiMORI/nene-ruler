//! 長さを単位に従って綴った文字列。表示とクリップボードで同じものを使う（FR-004 / FR-007）。

use alloc::format;
use alloc::string::String;

use crate::length::Length;
use crate::unit::Unit;

/// 1 ミリの何分の 1 がインチか。
const MILLIMETRES_PER_INCH: f64 = 25.4;

/// `400 px` / `105.8 mm` / `4.17 in` の形。整形はここだけが行う（ARC-001）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LengthText(String);

impl LengthText {
    /// 物理画素のまま綴る。
    #[must_use]
    pub fn in_pixels(length: Length) -> Self {
        Self(format!("{} {}", length.px(), Unit::Px.suffix()))
    }

    /// ミリで綴る（小数 1 桁）。`millimetres_per_pixel` は `ScreenMetrics` から。
    #[must_use]
    pub fn in_millimetres(length: Length, millimetres_per_pixel: f64) -> Self {
        let mm = f64::from(length.px()) * millimetres_per_pixel;
        Self(format!("{mm:.1} {}", Unit::Mm.suffix()))
    }

    /// インチで綴る（小数 2 桁）。
    #[must_use]
    pub fn in_inches(length: Length, millimetres_per_pixel: f64) -> Self {
        let inches = f64::from(length.px()) * millimetres_per_pixel / MILLIMETRES_PER_INCH;
        Self(format!("{inches:.2} {}", Unit::In.suffix()))
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
    fn spells_millimetres_with_one_decimal() {
        let text = LengthText::in_millimetres(Length::INITIAL_WIDTH, 0.264_583);
        assert_eq!(text.as_str(), "105.8 mm");
    }

    #[test]
    fn spells_inches_with_two_decimals() {
        let text = LengthText::in_inches(Length::INITIAL_WIDTH, 0.264_583);
        assert_eq!(text.as_str(), "4.17 in");
    }
}
