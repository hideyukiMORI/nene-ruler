//! 枠の窓に出す通知（FR-014）。

/// 利用者に見せるべき降格。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameNotice {
    /// 合成器が無く、不透明に落ちている（FR-017）。
    Opaque,
    /// 物理寸法が取れず、mm / in を選んでも px で出している（FR-008）。
    NoScreenMetrics,
    /// 最前面を選んでいるが、WM が対応していない（FR-013 / ADR 0008）。
    NoAlwaysOnTop,
}

impl FrameNotice {
    /// 利用者に見せる短い文。
    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Self::Opaque => "no compositor: drawn opaque",
            Self::NoScreenMetrics => "no screen size: px only",
            Self::NoAlwaysOnTop => "always on top: not supported by this window manager",
        }
    }
}
