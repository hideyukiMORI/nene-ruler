//! 長さの単位。

/// px / mm / in の閉じた集合（FR-008）。増やすときは仕様と `match` の両方が変わる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// 物理画素。換算なし。
    Px,
    /// ミリメートル。`RandR` の物理寸法から換算する。
    Mm,
    /// インチ。同上。
    In,
}

impl Unit {
    /// 切替の巡回順。ここが正本（用語集）。
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Px => Self::Mm,
            Self::Mm => Self::In,
            Self::In => Self::Px,
        }
    }

    /// 表示に添える語。
    #[must_use]
    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Px => "px",
            Self::Mm => "mm",
            Self::In => "in",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_through_all_three() {
        assert_eq!(Unit::Px.next(), Unit::Mm);
        assert_eq!(Unit::Mm.next(), Unit::In);
        assert_eq!(Unit::In.next(), Unit::Px);
    }
}
