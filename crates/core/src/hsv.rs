//! 色相・彩度・明度。カラーピッカーが扱う色の座標（FR-010・ADR 0016）。
//!
//! 浮動小数を使わず整数で換算する（`no_std` で丸め関数が無い・結果を決定的にする）。

use crate::rgb::Rgb;

/// 1 周（0.1 度刻み）。整数の 1 度刻みでは往復で成分が 2 ずれる（`#FB8C00` で実測）。
const FULL_TURN: u16 = 3600;
/// 彩度と明度の最大（千分率）。
const PER_MILLE: u16 = 1000;

/// 色相（0.1 度刻みで 0〜3599）・彩度（0〜1000）・明度（0〜1000）。作るときに範囲へ収める。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hsv {
    hue: u16,
    saturation: u16,
    value: u16,
}

impl Hsv {
    /// 範囲へ収めて作る。色相は 3600 で回り、彩度と明度は 1000 で頭打ち。
    #[must_use]
    pub const fn new(hue: u16, saturation: u16, value: u16) -> Self {
        Self {
            hue: hue % FULL_TURN,
            saturation: if saturation > PER_MILLE {
                PER_MILLE
            } else {
                saturation
            },
            value: if value > PER_MILLE { PER_MILLE } else { value },
        }
    }

    /// RGB から。灰（彩度 0）の色相は 0、黒の彩度は 0。
    #[must_use]
    pub fn of(rgb: Rgb) -> Self {
        let (red, green, blue) = (
            i32::from(rgb.red()),
            i32::from(rgb.green()),
            i32::from(rgb.blue()),
        );
        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);
        let delta = max - min;
        let value = divide_rounded(max * 1000, 255);
        let saturation = if max == 0_i32 {
            0_i32
        } else {
            divide_rounded(delta * 1000, max)
        };
        let hue = if delta == 0_i32 {
            0_i32
        } else if max == red {
            divide_rounded(600 * (green - blue), delta)
        } else if max == green {
            divide_rounded(600 * (blue - red), delta) + 1200_i32
        } else {
            divide_rounded(600 * (red - green), delta) + 2400_i32
        };
        Self::new(
            narrow(hue.rem_euclid(i32::from(FULL_TURN))),
            narrow(saturation),
            narrow(value),
        )
    }

    /// RGB へ。各成分を 0〜255 に四捨五入する。
    #[must_use]
    pub fn to_rgb(self) -> Rgb {
        // 明度を 0〜255000 に広げて、割り算の切り捨てを最後の 1 回にまとめる。
        let value = u32::from(self.value) * 255;
        let chroma = value * u32::from(self.saturation) / u32::from(PER_MILLE);
        let within = u32::from(self.hue % 1200);
        let rising = chroma * (600 - within.abs_diff(600)) / 600;
        let floor = value - chroma;
        let (red, green, blue) = match self.hue / 600 {
            0 => (chroma, rising, 0),
            1 => (rising, chroma, 0),
            2 => (0, chroma, rising),
            3 => (0, rising, chroma),
            4 => (rising, 0, chroma),
            _sixth => (chroma, 0, rising),
        };
        Rgb::new(
            channel(red + floor),
            channel(green + floor),
            channel(blue + floor),
        )
    }

    /// 色相（0.1 度）。
    #[must_use]
    pub const fn hue(self) -> u16 {
        self.hue
    }

    /// 彩度（千分率）。
    #[must_use]
    pub const fn saturation(self) -> u16 {
        self.saturation
    }

    /// 明度（千分率）。
    #[must_use]
    pub const fn value(self) -> u16 {
        self.value
    }

    /// 色相だけを変えた色。
    #[must_use]
    pub const fn with_hue(self, hue: u16) -> Self {
        Self::new(hue, self.saturation, self.value)
    }

    /// 彩度と明度を変えた色。色相は保つ。
    #[must_use]
    pub const fn with_shade(self, saturation: u16, value: u16) -> Self {
        Self::new(self.hue, saturation, value)
    }
}

/// 0 以上の整数の四捨五入の割り算。負の分子は 0 側へ寄せずに床で丸める。
const fn divide_rounded(numerator: i32, denominator: i32) -> i32 {
    (2 * numerator + denominator).div_euclid(2 * denominator)
}

/// 範囲が分かっている値を `u16` に。外れたら 0（起こらない）。
fn narrow(value: i32) -> u16 {
    u16::try_from(value).unwrap_or(0)
}

/// 0〜255000 の成分を 0〜255 に四捨五入する。
fn channel(scaled: u32) -> u8 {
    u8::try_from((scaled + 500) / 1000).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primaries_and_greys_convert_exactly() {
        let to_rgb = [
            ((0, 1000, 1000), (255, 0, 0)),
            ((1200, 1000, 1000), (0, 255, 0)),
            ((2400, 1000, 1000), (0, 0, 255)),
            ((600, 1000, 1000), (255, 255, 0)),
            ((200, 0, 1000), (255, 255, 255)),
            ((200, 1000, 0), (0, 0, 0)),
        ];
        for ((hue, saturation, value), (red, green, blue)) in to_rgb {
            assert_eq!(
                Hsv::new(hue, saturation, value).to_rgb(),
                Rgb::new(red, green, blue)
            );
        }
        let to_hsv = [
            ((255, 0, 0), (0, 1000, 1000)),
            ((0, 0, 255), (2400, 1000, 1000)),
            ((128, 128, 128), (0, 0, 502)),
            ((0, 0, 0), (0, 0, 0)),
        ];
        for ((red, green, blue), (hue, saturation, value)) in to_hsv {
            assert_eq!(
                Hsv::of(Rgb::new(red, green, blue)),
                Hsv::new(hue, saturation, value)
            );
        }
    }

    #[test]
    fn presets_survive_a_round_trip_within_one_step() {
        for preset in Rgb::PRESETS {
            let back = Hsv::of(preset).to_rgb();
            for (a, b) in [
                (preset.red(), back.red()),
                (preset.green(), back.green()),
                (preset.blue(), back.blue()),
            ] {
                assert!(a.abs_diff(b) <= 1, "{preset:?} -> {back:?}");
            }
        }
    }

    #[test]
    fn new_keeps_values_in_range() {
        assert_eq!(Hsv::new(3600, 1200, 5000), Hsv::new(0, 1000, 1000));
        assert_eq!(Hsv::new(7205, 10, 20).hue(), 5);
        let shade = Hsv::new(30, 100, 200).with_shade(900, 800);
        assert_eq!(
            (shade.hue(), shade.saturation(), shade.value()),
            (30, 900, 800)
        );
        assert_eq!(shade.with_hue(300).hue(), 300);
    }
}
