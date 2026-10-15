//! 枠の四辺。

use crate::axis::Axis;

/// 枠の線。四辺すべてに目盛を描く（FR-003）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// 上辺。
    Top,
    /// 右辺。
    Right,
    /// 下辺。
    Bottom,
    /// 左辺。
    Left,
}

impl Edge {
    /// 描く順。決定的（RS-016）。
    pub const ALL: [Self; 4] = [Self::Top, Self::Right, Self::Bottom, Self::Left];

    /// その辺が測る軸。上下は幅、左右は高さ。
    #[must_use]
    pub const fn axis(self) -> Axis {
        match self {
            Self::Top | Self::Bottom => Axis::Horizontal,
            Self::Left | Self::Right => Axis::Vertical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_edges_measure_width() {
        assert_eq!(Edge::Top.axis(), Axis::Horizontal);
        assert_eq!(Edge::Bottom.axis(), Axis::Horizontal);
        assert_eq!(Edge::Left.axis(), Axis::Vertical);
        assert_eq!(Edge::Right.axis(), Axis::Vertical);
    }
}
