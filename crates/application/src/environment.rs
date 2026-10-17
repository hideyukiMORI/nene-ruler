//! 起動時に一度だけ取り込む環境の事実（ARC-004 の表）。

use crate::layering::Layering;
use crate::transparency::Transparency;

/// 透明に描けるかと、最前面が効くか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Environment {
    transparency: Transparency,
    layering: Layering,
}

impl Environment {
    /// 両方を束ねる。
    #[must_use]
    pub const fn new(transparency: Transparency, layering: Layering) -> Self {
        Self {
            transparency,
            layering,
        }
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
}
