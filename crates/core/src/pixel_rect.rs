//! 窓の中の矩形。

use crate::pixel_point::PixelPoint;

/// 窓の左上を原点とする物理画素の矩形。X の `Rectangle` と同じ形で、cairo へも `f64` で写せる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl PixelRect {
    /// 左上と大きさから作る。
    #[must_use]
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// 左端。
    #[must_use]
    pub const fn x(self) -> u16 {
        self.x
    }

    /// 上端。
    #[must_use]
    pub const fn y(self) -> u16 {
        self.y
    }

    /// 幅。
    #[must_use]
    pub const fn width(self) -> u16 {
        self.width
    }

    /// 高さ。
    #[must_use]
    pub const fn height(self) -> u16 {
        self.height
    }

    /// 右端の画素（含む）。幅 0 なら左端と同じ。
    #[must_use]
    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width.saturating_sub(1))
    }

    /// 下端の画素（含む）。高さ 0 なら上端と同じ。
    #[must_use]
    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height.saturating_sub(1))
    }

    /// 点を含むか。幅か高さが 0 なら何も含まない。
    #[must_use]
    pub const fn contains(self, point: PixelPoint) -> bool {
        self.width > 0
            && self.height > 0
            && point.x() >= self.x
            && point.x() <= self.right()
            && point.y() >= self.y
            && point.y() <= self.bottom()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_its_edges_but_not_beyond() {
        let rect = PixelRect::new(10, 20, 5, 3);
        assert!(rect.contains(PixelPoint::new(10, 20)));
        assert!(rect.contains(PixelPoint::new(14, 22)));
        assert!(!rect.contains(PixelPoint::new(15, 22)));
        assert!(!rect.contains(PixelPoint::new(14, 23)));
        assert!(!rect.contains(PixelPoint::new(9, 20)));
    }

    #[test]
    fn empty_rect_contains_nothing() {
        assert!(!PixelRect::new(1, 1, 0, 5).contains(PixelPoint::new(1, 1)));
        assert_eq!(PixelRect::new(1, 1, 0, 5).right(), 1);
    }
}
