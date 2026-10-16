//! 合成ルートが中核へ渡すポートの束（ARC-006）。

use crate::clipboard_port::ClipboardPort;
use crate::screen_metrics_port::ScreenMetricsPort;
use crate::settings_store_port::SettingsStorePort;

/// 物理寸法・設定の保存・クリップボード。増えたらここに足し、`RulerController::new` の引数は増やさない。
#[derive(Clone, Copy)]
pub struct Ports<'p> {
    metrics: &'p dyn ScreenMetricsPort,
    store: &'p dyn SettingsStorePort,
    clipboard: &'p dyn ClipboardPort,
}

impl<'p> Ports<'p> {
    /// 実装を束ねる。
    #[must_use]
    pub const fn new(
        metrics: &'p dyn ScreenMetricsPort,
        store: &'p dyn SettingsStorePort,
        clipboard: &'p dyn ClipboardPort,
    ) -> Self {
        Self {
            metrics,
            store,
            clipboard,
        }
    }

    /// 物理寸法。
    #[must_use]
    pub const fn metrics(&self) -> &'p dyn ScreenMetricsPort {
        self.metrics
    }

    /// 設定の保存。
    #[must_use]
    pub const fn store(&self) -> &'p dyn SettingsStorePort {
        self.store
    }

    /// クリップボード。
    #[must_use]
    pub const fn clipboard(&self) -> &'p dyn ClipboardPort {
        self.clipboard
    }
}

impl core::fmt::Debug for Ports<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("Ports { metrics, store, clipboard }")
    }
}
