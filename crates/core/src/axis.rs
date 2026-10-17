//! 横（幅）か縦（高さ）か。

/// 目盛と数字がどちらの辺のものかを言う閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// 横。幅を測る。
    Horizontal,
    /// 縦。高さを測る。
    Vertical,
}
