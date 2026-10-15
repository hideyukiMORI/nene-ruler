//! 主目盛に添える数字の置き場。

use crate::pixel_point::PixelPoint;

/// 数字の値と、その左下（ベースラインの左端）の位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickLabel {
    value: u16,
    baseline: PixelPoint,
}

impl TickLabel {
    /// 値と位置から作る。
    #[must_use]
    pub const fn new(value: u16, baseline: PixelPoint) -> Self {
        Self { value, baseline }
    }

    /// 原点からの距離。そのまま数字にする。
    #[must_use]
    pub const fn value(self) -> u16 {
        self.value
    }

    /// 文字のベースラインの左端。
    #[must_use]
    pub const fn baseline(self) -> PixelPoint {
        self.baseline
    }
}
