//! 重なり順（FR-013）。

/// 常に最前面か、通常の重なり順か。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowLayer {
    /// `_NET_WM_STATE_ABOVE` を付ける。既定。
    AlwaysOnTop,
    /// 他の窓と同じ。
    Normal,
}

impl WindowLayer {
    /// 切り替えた先。
    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            Self::AlwaysOnTop => Self::Normal,
            Self::Normal => Self::AlwaysOnTop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_back_and_forth() {
        assert_eq!(WindowLayer::AlwaysOnTop.toggled(), WindowLayer::Normal);
        assert_eq!(WindowLayer::Normal.toggled(), WindowLayer::AlwaysOnTop);
    }
}
