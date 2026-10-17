//! 枠の窓に出す通知（FR-014）。

/// 利用者に見せるべき降格。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameNotice {
    /// 合成器が無く、不透明に落ちている（FR-017）。
    Opaque,
}

impl FrameNotice {
    /// 利用者に見せる短い文。
    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Self::Opaque => "no compositor: drawn opaque",
        }
    }
}
