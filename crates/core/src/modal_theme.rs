//! 設定モーダルの見え（FR-010）。枠の窓には効かない（FR-018）。

/// ダークかライトか。「システムに従う」は置かない（FR-010・Issue #26）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalTheme {
    /// 暗い背景に明るい文字。
    Dark,
    /// 明るい背景に暗い文字。
    Light,
}

impl ModalTheme {
    /// モーダルに並べる順。
    pub const ALL: [Self; 2] = [Self::Dark, Self::Light];

    /// 表示の語。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }
}
