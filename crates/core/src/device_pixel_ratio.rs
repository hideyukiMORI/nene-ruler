//! 物理画素と論理画素の比（ADR 0027）。ブラウザの `devicePixelRatio` と同じもの。

use alloc::format;
use alloc::string::String;

/// 1 論理画素あたりの物理画素の数を、dpi（96 で 1 倍）で持つ。
///
/// 出どころはデスクトップの dpi（X11 は `Xft.dpi`、Windows はモニタの dpi）だけで、
/// `RandR` のミリ寸法から出した dpi は使わない。ブラウザもそこを見ないからである。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DevicePixelRatio {
    dpi: u32,
}

impl DevicePixelRatio {
    /// 1 倍の基準の dpi。
    pub const BASE_DPI: u32 = 96;
    /// 1 倍。dpi が分からないときもこれ。
    pub const ONE: Self = Self {
        dpi: Self::BASE_DPI,
    };

    /// デスクトップの dpi から。無い・0 なら 1 倍。
    #[must_use]
    pub const fn from_dpi(dpi: Option<u32>) -> Self {
        match dpi {
            Some(dpi) if dpi > 0 => Self { dpi },
            Some(_zero) => Self::ONE,
            None => Self::ONE,
        }
    }

    /// 物理画素を論理画素の 10 分の 1 の単位に換算する（四捨五入）。
    #[must_use]
    pub fn logical_tenths(self, px: u16) -> u32 {
        let scaled = u32::from(px) * Self::BASE_DPI * 10;
        (scaled + self.dpi / 2) / self.dpi
    }

    /// 論理画素の寸法を物理画素に換算する（四捨五入）。字の大きさと、論理画素で刻む目盛の位置に使う（ADR 0028）。
    /// `u16` に収まらなければ `u16::MAX`。
    #[must_use]
    pub fn physical(self, logical: u16) -> u16 {
        let scaled = u32::from(logical) * self.dpi + Self::BASE_DPI / 2;
        u16::try_from(scaled / Self::BASE_DPI).unwrap_or(u16::MAX)
    }

    /// 換算の説明（`1 logical px = 1.5 physical px`）。小数は 2 桁までで、末尾の 0 は書かない。
    #[must_use]
    pub fn describe(self) -> String {
        let hundredths = (self.dpi * 100 + Self::BASE_DPI / 2) / Self::BASE_DPI;
        let (whole, fraction) = (hundredths / 100, hundredths % 100);
        let factor = if fraction == 0 {
            format!("{whole}")
        } else if fraction % 10 == 0 {
            format!("{whole}.{}", fraction / 10)
        } else {
            format!("{whole}.{fraction:02}")
        };
        format!("1 logical px = {factor} physical px")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_or_zero_dpi_is_one_to_one() {
        assert_eq!(DevicePixelRatio::from_dpi(None), DevicePixelRatio::ONE);
        assert_eq!(DevicePixelRatio::from_dpi(Some(0)), DevicePixelRatio::ONE);
        assert_eq!(DevicePixelRatio::ONE.logical_tenths(400), 4000);
    }

    #[test]
    fn divides_by_the_scale_and_rounds_to_a_tenth() {
        let half_again = DevicePixelRatio::from_dpi(Some(144));
        assert_eq!(half_again.logical_tenths(400), 2667);
        assert_eq!(half_again.logical_tenths(240), 1600);
        assert_eq!(half_again.logical_tenths(16), 107);
        let quarter = DevicePixelRatio::from_dpi(Some(120));
        assert_eq!(quarter.logical_tenths(400), 3200);
        assert_eq!(quarter.logical_tenths(401), 3208);
        assert_eq!(
            DevicePixelRatio::from_dpi(Some(192)).logical_tenths(u16::MAX),
            327_675
        );
    }

    #[test]
    fn scales_logical_sizes_to_physical_pixels_and_rounds() {
        let half_again = DevicePixelRatio::from_dpi(Some(144));
        assert_eq!(half_again.physical(9), 14);
        assert_eq!(half_again.physical(11), 17);
        assert_eq!(half_again.physical(5), 8);
        assert_eq!(half_again.physical(15), 23);
        assert_eq!(half_again.physical(50), 75);
        assert_eq!(DevicePixelRatio::ONE.physical(11), 11);
        assert_eq!(DevicePixelRatio::from_dpi(Some(120)).physical(9), 11);
        assert_eq!(DevicePixelRatio::from_dpi(Some(192)).physical(11), 22);
        assert_eq!(
            DevicePixelRatio::from_dpi(Some(192)).physical(u16::MAX),
            u16::MAX
        );
    }

    #[test]
    fn describes_the_factor_without_trailing_zeros() {
        let cases = [
            (96, "1 logical px = 1 physical px"),
            (120, "1 logical px = 1.25 physical px"),
            (144, "1 logical px = 1.5 physical px"),
            (168, "1 logical px = 1.75 physical px"),
            (192, "1 logical px = 2 physical px"),
            (100, "1 logical px = 1.04 physical px"),
        ];
        for (dpi, expected) in cases {
            assert_eq!(DevicePixelRatio::from_dpi(Some(dpi)).describe(), expected);
        }
    }
}
