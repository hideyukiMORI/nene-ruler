//! 設定モーダルに出す通知の組（FR-014）。

use nene_ruler_core::screen_metrics_failure::ScreenMetricsFailure;

use crate::layering::Layering;
use crate::settings_store_failure::SettingsStoreFailure;

/// 設定の読み書きの失敗・物理寸法が取れないこと・最前面が効かないこと。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsNotices {
    store_failure: Option<SettingsStoreFailure>,
    metrics_failure: Option<ScreenMetricsFailure>,
    layering: Layering,
}

impl SettingsNotices {
    /// 両方を束ねる。
    #[must_use]
    pub const fn new(
        store_failure: Option<SettingsStoreFailure>,
        metrics_failure: Option<ScreenMetricsFailure>,
        layering: Layering,
    ) -> Self {
        Self {
            store_failure,
            metrics_failure,
            layering,
        }
    }

    /// 最前面が効くか。効かなければ「Always on top」の横に示す。
    #[must_use]
    pub const fn layering(self) -> Layering {
        self.layering
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
