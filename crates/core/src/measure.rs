//! 長さを綴るときの単位と換算の組（ADR 0027）。

use crate::device_pixel_ratio::DevicePixelRatio;
use crate::length::Length;
use crate::unit::Unit;

/// 選んだ単位と、いまの環境の比。長さを 10 分の 1 の単位の整数にするのはここだけ（ARC-001）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measure {
    unit: Unit,
    ratio: DevicePixelRatio,
}

impl Measure {
    /// 物理画素。換算が要らないときの形。
    pub const PHYSICAL: Self = Self {
        unit: Unit::Physical,
        ratio: DevicePixelRatio::ONE,
    };

    /// 単位と比から。
    #[must_use]
    pub const fn new(unit: Unit, ratio: DevicePixelRatio) -> Self {
        Self { unit, ratio }
    }

    /// 単位。
    #[must_use]
    pub const fn unit(self) -> Unit {
        self.unit
    }

    /// 長さを、選んだ単位の 10 分の 1 を 1 とする整数に。
    #[must_use]
    pub fn tenths(self, length: Length) -> u32 {
        match self.unit {
            Unit::Physical => u32::from(length.px()) * 10,
            Unit::Logical => self.ratio.logical_tenths(length.px()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_pixels_are_never_converted() {
        let dense = Measure::new(Unit::Physical, DevicePixelRatio::from_dpi(Some(192)));
        assert_eq!(dense.tenths(Length::INITIAL_WIDTH), 4000);
        assert_eq!(Measure::PHYSICAL.unit(), Unit::Physical);
    }

    #[test]
    fn logical_pixels_follow_the_ratio() {
        let logical = Measure::new(Unit::Logical, DevicePixelRatio::from_dpi(Some(192)));
        assert_eq!(logical.tenths(Length::INITIAL_WIDTH), 2000);
        assert_eq!(logical.unit(), Unit::Logical);
    }
}
