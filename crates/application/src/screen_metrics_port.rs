//! 画面の物理寸法を得るポート（ARC-003）。

use nene_ruler_core::screen_metrics::ScreenMetrics;
use nene_ruler_core::screen_metrics_failure::ScreenMetricsFailure;

/// `RandR` の物理寸法を中核へ渡す窓口。実装は adapters-x11。
pub trait ScreenMetricsPort {
    /// いまの画面の画素数とミリ寸法。
    ///
    /// # Errors
    ///
    /// 取れない・0 だった、など `ScreenMetricsFailure` の閉じた理由。
    fn screen_metrics(&self) -> Result<ScreenMetrics, ScreenMetricsFailure>;
}
