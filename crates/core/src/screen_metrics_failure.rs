//! 画面の大きさが得られなかった理由（FR-005 / FR-016）。

/// `ScreenMetrics` を作れなかった、または取れなかった理由。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenMetricsFailure {
    /// 画素数が 0 だった。
    ZeroPixels,
    /// X から取れなかった（`RandR` が無い・出力が無い・要求が失敗した）。
    Unavailable,
}
