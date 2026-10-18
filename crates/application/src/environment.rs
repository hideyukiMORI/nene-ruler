//! 起動時に一度だけ取り込む環境の事実（ARC-004 の表）。

use crate::transparency::Transparency;

/// 透明に描けるかと、デスクトップの dpi（X11 は `Xft.dpi`、Windows はモニタの dpi）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Environment {
    transparency: Transparency,
    desktop_dpi: Option<u32>,
}

impl Environment {
    /// 透明に描けるかから作る。デスクトップの dpi は無いものとして始める。
    #[must_use]
    pub const fn new(transparency: Transparency) -> Self {
        Self {
            transparency,
            desktop_dpi: None,
        }
    }

    /// デスクトップの dpi を添える。UI の倍率の第一の元（FR-016・ADR 0011 / 0021）。
    /// X11 はルート窓の `RESOURCE_MANAGER` の `Xft.dpi`、Windows はモニタの dpi。
    #[must_use]
    pub const fn with_desktop_dpi(self, desktop_dpi: Option<u32>) -> Self {
        Self {
            desktop_dpi,
            ..self
        }
    }

    /// 透明に描けるか。
    #[must_use]
    pub const fn transparency(self) -> Transparency {
        self.transparency
    }

    /// デスクトップの dpi。分からなければ `None`。
    #[must_use]
    pub const fn desktop_dpi(self) -> Option<u32> {
        self.desktop_dpi
    }
}
