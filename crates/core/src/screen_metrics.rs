//! 画面の物理寸法。mm / in の換算の元（FR-008 / FR-016）。

use crate::axis::Axis;
use crate::screen_metrics_failure::ScreenMetricsFailure;

/// `RandR` が返す画面の画素数とミリ寸法。境界で一度だけ検証する（ARC-008）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenMetrics {
    width_px: u32,
    height_px: u32,
    width_mm: u32,
    height_mm: u32,
}

impl ScreenMetrics {
    /// 画素数とミリ寸法がどちらも 0 でないときだけ作る。
    ///
    /// # Errors
    ///
    /// 画素数が 0 なら `ZeroPixels`、ミリ寸法が 0 なら `ZeroMillimetres`。
    pub const fn new(
        width_px: u32,
        height_px: u32,
        width_mm: u32,
        height_mm: u32,
    ) -> Result<Self, ScreenMetricsFailure> {
        if width_px == 0 || height_px == 0 {
            Err(ScreenMetricsFailure::ZeroPixels)
        } else if width_mm == 0 || height_mm == 0 {
            Err(ScreenMetricsFailure::ZeroMillimetres)
        } else {
            Ok(Self {
                width_px,
                height_px,
                width_mm,
                height_mm,
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

    /// 軸ごとの 1 画素あたりのミリ数。
    #[must_use]
    pub fn millimetres_per_pixel(self, axis: Axis) -> f64 {
        let (px, mm) = match axis {
            Axis::Horizontal => (self.width_px, self.width_mm),
            Axis::Vertical => (self.height_px, self.height_mm),
        };
        f64::from(mm) / f64::from(px)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_dimensions() {
        assert_eq!(
            ScreenMetrics::new(0, 1080, 527, 296),
            Err(ScreenMetricsFailure::ZeroPixels)
        );
        assert_eq!(
            ScreenMetrics::new(1920, 1080, 527, 0),
            Err(ScreenMetricsFailure::ZeroMillimetres)
        );
    }

    #[test]
    fn converts_per_axis() {
        let metrics = ScreenMetrics::new(1920, 1080, 480, 270).unwrap();
        assert!((metrics.millimetres_per_pixel(Axis::Horizontal) - 0.25_f64).abs() < 1e-9_f64);
        assert!((metrics.millimetres_per_pixel(Axis::Vertical) - 0.25_f64).abs() < 1e-9_f64);
        assert_eq!(metrics.pixels(Axis::Vertical), 1080);
    }
}
