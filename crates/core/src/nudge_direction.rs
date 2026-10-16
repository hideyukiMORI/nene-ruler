//! 矢印の向き（FR-015）。

/// 4 方向の閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NudgeDirection {
    /// 左へ。
    Left,
    /// 右へ。
    Right,
    /// 上へ。
    Up,
    /// 下へ。
    Down,
}
