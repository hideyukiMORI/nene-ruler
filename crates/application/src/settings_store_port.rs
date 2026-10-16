//! 設定の永続化のポート（ARC-003 / ARC-009）。実装は adapters-x11。

use nene_ruler_core::ruler_settings::RulerSettings;

use crate::settings_store_failure::SettingsStoreFailure;

/// 保存された設定を読み、いまの設定を書く窓口。
pub trait SettingsStorePort {
    /// 保存されている設定。まだ無ければ `Ok(None)`。
    ///
    /// # Errors
    ///
    /// 読めない・知らない版・形が違う。既定値へ黙って落とさず、application が示す。
    fn load(&self) -> Result<Option<RulerSettings>, SettingsStoreFailure>;

    /// 設定を書く。
    ///
    /// # Errors
    ///
    /// 書けない。
    fn save(&self, settings: RulerSettings) -> Result<(), SettingsStoreFailure>;
}
