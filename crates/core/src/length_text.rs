//! 長さを単位に従って綴った文字列。表示とクリップボードで同じものを使う（FR-004 / FR-007）。

use alloc::format;
use alloc::string::String;

use crate::length::Length;
use crate::measure::Measure;

/// `400 px` / `266.7 logical px` の形。整形はここだけが行う（ARC-001）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LengthText(String);

impl LengthText {
    /// 1 つの長さを綴る。
    #[must_use]
    pub fn single(length: Length, measure: Measure) -> Self {
        Self(format!(
            "{} {}",
            spell(measure.tenths(length)),
            measure.unit().suffix()
        ))
    }

    /// 幅と高さを 1 行で綴る（`100 × 20 px`）。詰めた表示で使う（ADR 0017）。
    #[must_use]
    pub fn size(width: Length, height: Length, measure: Measure) -> Self {
        Self(format!(
            "{} \u{d7} {} {}",
            spell(measure.tenths(width)),
            spell(measure.tenths(height)),
            measure.unit().suffix()
        ))
    }

    /// 幅と高さの短い形（`16×16`）。線を消した表示で、窓に収まるように空白と単位を省く（ADR 0019）。
    #[must_use]
    pub fn short_size(width: Length, height: Length, measure: Measure) -> Self {
        Self(format!(
            "{}\u{d7}{}",
            spell(measure.tenths(width)),
            spell(measure.tenths(height))
        ))
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

/// 10 分の 1 の単位の整数を、整数ならそのまま、そうでなければ小数 1 桁で綴る。
fn spell(tenths: u32) -> String {
    let (whole, fraction) = (tenths / 10, tenths % 10);
    if fraction == 0 {
        format!("{whole}")
    } else {
        format!("{whole}.{fraction}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_pixel_ratio::DevicePixelRatio;
    use crate::unit::Unit;

    fn logical(dpi: u32) -> Measure {
        Measure::new(Unit::Logical, DevicePixelRatio::from_dpi(Some(dpi)))
    }

    #[test]
    fn spells_pixels_plainly() {
        let text = LengthText::single(Length::INITIAL_WIDTH, Measure::PHYSICAL);
        assert_eq!(text.as_str(), "400 px");
        assert_eq!(text.chars(), 6);
    }

    #[test]
    fn spells_logical_pixels_with_a_decimal_only_when_needed() {
        let fractional = LengthText::single(Length::INITIAL_WIDTH, logical(144));
        assert_eq!(fractional.as_str(), "266.7 logical px");
        assert_eq!(fractional.chars(), 16);
        let whole = LengthText::single(Length::INITIAL_HEIGHT, logical(144));
        assert_eq!(whole.as_str(), "160 logical px");
        let one_to_one = LengthText::single(Length::INITIAL_WIDTH, logical(96));
        assert_eq!(one_to_one.as_str(), "400 logical px");
    }

    #[test]
    fn spells_both_lengths_on_one_line() {
        let (width, height) = (Length::new(100).unwrap(), Length::new(40).unwrap());
        let text = LengthText::size(width, height, Measure::PHYSICAL);
        assert_eq!(text.as_str(), "100 \u{d7} 40 px");
        assert_eq!(text.chars(), 11);
        let in_logical = LengthText::size(width, height, logical(192));
        assert_eq!(in_logical.as_str(), "50 \u{d7} 20 logical px");
    }

    #[test]
    fn spells_the_short_size_without_spaces_or_unit() {
        let side = Length::new(16).unwrap();
        let text = LengthText::short_size(side, side, Measure::PHYSICAL);
        assert_eq!(text.as_str(), "16\u{d7}16");
        assert_eq!(text.chars(), 5);
        let in_logical = LengthText::short_size(side, side, logical(144));
        assert_eq!(in_logical.as_str(), "10.7\u{d7}10.7");
    }
}
