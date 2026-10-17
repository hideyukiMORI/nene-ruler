//! 物理寸法が得られなかった理由（FR-008 / FR-014）。

/// `ScreenMetrics` を作れなかった、または取れなかった理由。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenMetricsFailure {
    /// 画素数が 0 だった。
    ZeroPixels,
    /// ミリ寸法が 0 だった（`RandR` が物理寸法を知らない出力）。
    ZeroMillimetres,
    /// X から取れなかった（`RandR` が無い・出力が無い・要求が失敗した）。
    Unavailable,
}
