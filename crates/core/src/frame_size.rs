//! 枠の幅と高さの組（ARC-004 の「枠」）。

use crate::axis::Axis;
use crate::length::Length;

/// 枠が測っている幅と高さ。左上が原点。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameSize {
    width: Length,
    height: Length,
}

impl FrameSize {
    /// 初期の枠（SPECIFICATION 第 2 節）。
    pub const INITIAL: Self = Self {
        width: Length::INITIAL_WIDTH,
        height: Length::INITIAL_HEIGHT,
    };

    /// 検証済みの長さから組を作る。
    #[must_use]
    pub const fn new(width: Length, height: Length) -> Self {
        Self { width, height }
    }

    /// 幅。
    #[must_use]
    pub const fn width(self) -> Length {
        self.width
    }

    /// 高さ。
    #[must_use]
    pub const fn height(self) -> Length {
        self.height
    }

    /// 軸ごとの長さ。
    #[must_use]
    pub const fn along(self, axis: Axis) -> Length {
        match axis {
            Axis::Horizontal => self.width,
            Axis::Vertical => self.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_each_axis() {
        let size = FrameSize::INITIAL;
        assert_eq!(size.along(Axis::Horizontal), size.width());
        assert_eq!(size.along(Axis::Vertical), size.height());
        assert_eq!(size.width().px(), 400);
        assert_eq!(size.height().px(), 240);
    }
}
