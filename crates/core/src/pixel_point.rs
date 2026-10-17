//! 窓の中の 1 点。

/// 窓の左上を原点とする物理画素の座標。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelPoint {
    x: u16,
    y: u16,
}

impl PixelPoint {
    /// 座標から作る。
    #[must_use]
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    /// X のイベント座標（`i16`。窓の外は負になりうる）から作る。負なら窓の外なので `None`。
    #[must_use]
    pub fn from_event(x: i16, y: i16) -> Option<Self> {
        Some(Self::new(u16::try_from(x).ok()?, u16::try_from(y).ok()?))
    }

    /// 横座標。
    #[must_use]
    pub const fn x(self) -> u16 {
        self.x
    }

    /// 縦座標。
    #[must_use]
    pub const fn y(self) -> u16 {
        self.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_event_coordinates_are_outside() {
        assert_eq!(PixelPoint::from_event(-1, 3), None);
        assert_eq!(PixelPoint::from_event(3, -1), None);
        assert_eq!(PixelPoint::from_event(3, 4), Some(PixelPoint::new(3, 4)));
    }
}
