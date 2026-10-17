//! 起動時に一度だけ取り込む環境の事実（ARC-004 の表）。

use crate::layering::Layering;
use crate::transparency::Transparency;

/// 透明に描けるか、最前面が効くか、`Xft.dpi` が何か。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Environment {
    transparency: Transparency,
    layering: Layering,
    xft_dpi: Option<u32>,
}

impl Environment {
    /// 透明と最前面を束ねる。`Xft.dpi` は無いものとして始める。
    #[must_use]
    pub const fn new(transparency: Transparency, layering: Layering) -> Self {
        Self {
            transparency,
            layering,
            xft_dpi: None,
        }
    }

    /// `Xft.dpi`（ルート窓の `RESOURCE_MANAGER`）を添える。UI の倍率の第一の元（FR-016・ADR 0011）。
    #[must_use]
    pub const fn with_xft_dpi(self, xft_dpi: Option<u32>) -> Self {
        Self { xft_dpi, ..self }
    }

    /// 透明に描けるか。
    #[must_use]
    pub const fn transparency(self) -> Transparency {
        self.transparency
    }

    /// 最前面が効くか。
    #[must_use]
    pub const fn layering(self) -> Layering {
        self.layering
    }

    /// `Xft.dpi`。設定されていなければ `None`。
    #[must_use]
    pub const fn xft_dpi(self) -> Option<u32> {
        self.xft_dpi
    }
}
