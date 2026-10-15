//! 長さの検証に失敗した理由（RS-005 / ARC-010）。

/// `Length::new` が値を拒んだ理由。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthFailure {
    /// 下限（`Length::MIN_PX`）より短い。
    BelowMinimum,
    /// 上限（`Length::MAX_PX`）より長い。
    AboveMaximum,
}
