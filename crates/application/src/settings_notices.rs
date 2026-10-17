//! 設定モーダルに出す通知の組（FR-014）。

use nene_ruler_core::screen_metrics_failure::ScreenMetricsFailure;

use crate::settings_store_failure::SettingsStoreFailure;

/// 設定の読み書きの失敗と、物理寸法が取れないこと。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsNotices {
    store_failure: Option<SettingsStoreFailure>,
    metrics_failure: Option<ScreenMetricsFailure>,
}

impl SettingsNotices {
    /// 両方を束ねる。
    #[must_use]
    pub const fn new(
        store_failure: Option<SettingsStoreFailure>,
        metrics_failure: Option<ScreenMetricsFailure>,
    ) -> Self {
        Self {
            store_failure,
            metrics_failure,
        }
    }

    /// 設定の読み書きの失敗。
    #[must_use]
    pub const fn store_failure(self) -> Option<SettingsStoreFailure> {
        self.store_failure
    }

    /// 物理寸法が取れなかった理由。mm / in が効かないことを示すため。
    #[must_use]
    pub const fn metrics_failure(self) -> Option<ScreenMetricsFailure> {
        self.metrics_failure
    }
}
