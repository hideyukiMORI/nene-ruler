//! 画面の画素数と、分かれば横のミリ寸法。枠の上限（FR-005）と UI の倍率（FR-016）の元。

use crate::axis::Axis;
use crate::screen_metrics_failure::ScreenMetricsFailure;

/// `RandR` が返す画面の画素数と横のミリ寸法。境界で一度だけ検証する（ARC-008）。
///
/// 長さの換算には使わない（mm / in は外した・ADR 0012）。ミリ寸法は dpi を出すためだけに持つ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenMetrics {
    width_px: u32,
    height_px: u32,
    width_mm: u32,
}

impl ScreenMetrics {
    /// 画素数が 0 でないときだけ作る。横のミリ寸法は 0 でもよい（`RandR` が知らない出力）。
    ///
    /// # Errors
    ///
    /// 画素数が 0 なら `ZeroPixels`。
    pub const fn new(
        width_px: u32,
        height_px: u32,
        width_mm: u32,
    ) -> Result<Self, ScreenMetricsFailure> {
        if width_px == 0 || height_px == 0 {
            Err(ScreenMetricsFailure::ZeroPixels)
        } else {
            Ok(Self {
                width_px,
                height_px,
                width_mm,
            })
        }
    }

    /// 軸ごとの画素数。枠の上限（FR-005）に使う。
    #[must_use]
    pub const fn pixels(self, axis: Axis) -> u32 {
        match axis {
            Axis::Horizontal => self.width_px,
            Axis::Vertical => self.height_px,
        }
    }

    /// 横の画素密度（dpi）。UI の倍率の元（FR-016）。整数に切り捨てる。ミリ寸法が 0 なら `None`。
    #[must_use]
    pub fn dots_per_inch(self) -> Option<u32> {
        if self.width_mm == 0 {
            return None;
        }
        // 1 in = 25.4 mm。整数のまま割るために 10 倍して 254 で掛ける。
        let tenths_of_mm = u64::from(self.width_mm) * 10;
        let dpi = u64::from(self.width_px) * 254 / tenths_of_mm;
        Some(u32::try_from(dpi).unwrap_or(u32::MAX))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_pixels_but_not_zero_millimetres() {
        assert_eq!(
            ScreenMetrics::new(0, 1080, 527),
            Err(ScreenMetricsFailure::ZeroPixels)
        );
        let no_size = ScreenMetrics::new(3840, 2160, 0).unwrap();
        assert_eq!(no_size.pixels(Axis::Horizontal), 3840);
        assert_eq!(no_size.dots_per_inch(), None);
    }

    #[test]
    fn measures_dots_per_inch_across() {
        assert_eq!(
            ScreenMetrics::new(3840, 2160, 600).unwrap().dots_per_inch(),
            Some(162)
        );
        assert_eq!(
            ScreenMetrics::new(1920, 1080, 508).unwrap().dots_per_inch(),
            Some(96)
        );
    }

    #[test]
    fn gives_pixels_per_axis() {
        let metrics = ScreenMetrics::new(1920, 1080, 480).unwrap();
        assert_eq!(metrics.pixels(Axis::Horizontal), 1920);
        assert_eq!(metrics.pixels(Axis::Vertical), 1080);
    }
}
