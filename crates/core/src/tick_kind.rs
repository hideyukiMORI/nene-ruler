//! 目盛の種類。

/// 小・中・主。間隔と長さは SPECIFICATION 第 2 節（決め打ちしない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickKind {
    /// 5 px ごと。
    Minor,
    /// 10 px ごと。
    Middle,
    /// 50 px ごと。数字を添える（FR-003）。
    Major,
}

impl TickKind {
    /// 小目盛の間隔。目盛はこの倍数の位置にだけある。
    pub const SPACING_PX: u16 = 5;

    /// 原点からの距離で種類が決まる。
    #[must_use]
    pub const fn at(offset: u16) -> Self {
        if offset.is_multiple_of(50) {
            Self::Major
        } else if offset.is_multiple_of(10) {
            Self::Middle
        } else {
            Self::Minor
        }
    }

    /// 線から内側へ伸ばす長さ。
    #[must_use]
    pub const fn length_px(self) -> u16 {
        match self {
            Self::Minor => 4,
            Self::Middle => 7,
            Self::Major => 12,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_follows_the_offset() {
        assert_eq!(TickKind::at(5), TickKind::Minor);
        assert_eq!(TickKind::at(10), TickKind::Middle);
        assert_eq!(TickKind::at(50), TickKind::Major);
        assert_eq!(TickKind::at(100), TickKind::Major);
        assert_eq!(TickKind::at(120), TickKind::Middle);
    }

    #[test]
    fn major_is_the_longest() {
        assert!(TickKind::Minor.length_px() < TickKind::Middle.length_px());
        assert!(TickKind::Middle.length_px() < TickKind::Major.length_px());
    }
}
