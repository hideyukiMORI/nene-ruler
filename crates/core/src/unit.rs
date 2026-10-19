//! 長さの単位（FR-008・ADR 0027）。

/// 物理画素か CSS 画素かの閉じた集合。mm / in は置かない（ADR 0012 の理由はそのまま）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// 物理画素。換算なし。
    Px,
    /// CSS 画素。ブラウザの開発者ツールが示す px で、物理画素を dpi / 96 で割った値。
    CssPx,
}

impl Unit {
    /// モーダルに並べる順。
    pub const ALL: [Self; 2] = [Self::Px, Self::CssPx];

    /// 表示に添える語。モーダルのボタンの文字でもある。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Px => "px",
            Self::CssPx => "CSS px",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_both_in_the_order_of_the_modal() {
        assert_eq!(Unit::ALL, [Unit::Px, Unit::CssPx]);
        assert_eq!(Unit::Px.label(), "px");
        assert_eq!(Unit::CssPx.label(), "CSS px");
    }
}
