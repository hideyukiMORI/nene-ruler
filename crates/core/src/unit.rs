//! 長さの単位（FR-008・ADR 0027）。

/// 物理画素か論理画素かの閉じた集合。mm / in は置かない（ADR 0012 の理由はそのまま）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// 物理画素。換算なし。
    Physical,
    /// 論理画素。物理画素をデスクトップの拡大率（dpi / 96）で割った値。
    /// ブラウザの CSS px、Windows の DIP、Android の dp、Apple の point と同じ数。
    Logical,
}

impl Unit {
    /// モーダルに並べる順。
    pub const ALL: [Self; 2] = [Self::Physical, Self::Logical];

    /// モーダルのボタンの文字。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Physical => "Physical px",
            Self::Logical => "Logical px",
        }
    }

    /// 数字に添える語。物理画素はこの製品の `px` そのものなので裸の `px`（FR-003 / FR-016）、論理画素は換算した値だと分かるように `logical px`。
    #[must_use]
    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Physical => "px",
            Self::Logical => "logical px",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_both_in_the_order_of_the_modal() {
        assert_eq!(Unit::ALL, [Unit::Physical, Unit::Logical]);
        assert_eq!(Unit::Physical.label(), "Physical px");
        assert_eq!(Unit::Logical.label(), "Logical px");
        assert_eq!(Unit::Physical.suffix(), "px");
        assert_eq!(Unit::Logical.suffix(), "logical px");
    }
}
