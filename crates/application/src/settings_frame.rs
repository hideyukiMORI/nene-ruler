//! 設定モーダルを描くための値の組。

use alloc::format;
use alloc::string::String;

use nene_ruler_core::device_pixel_ratio::DevicePixelRatio;
use nene_ruler_core::modal_palette::ModalPalette;
use nene_ruler_core::ruler_settings::RulerSettings;
use nene_ruler_core::settings_layout::SettingsLayout;

use crate::modal_session::ModalSession;
use crate::product_version::ProductVersion;
use crate::settings_store_failure::SettingsStoreFailure;

/// 脚注のコピーライト。版の前に付く。
const COPYRIGHT: &str = "(c) 2026 Hideyuki Mori / MIT";

/// 設定・編集状態・配色・換算の説明・脚注。UI はこれを描いて部品の操作を意図として返す（ARC-011）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsFrame {
    layout: SettingsLayout,
    settings: RulerSettings,
    session: ModalSession,
    ratio_note: String,
    footer: String,
    store_failure: Option<SettingsStoreFailure>,
}

impl SettingsFrame {
    /// 値を束ねる。生成は `RulerController::settings_frame` だけが行う。
    #[must_use]
    pub fn new(
        layout: SettingsLayout,
        (settings, session): (RulerSettings, ModalSession),
        (version, ratio): (ProductVersion, DevicePixelRatio),
        store_failure: Option<SettingsStoreFailure>,
    ) -> Self {
        Self {
            layout,
            settings,
            session,
            ratio_note: ratio.describe(),
            footer: format!("NeNe Ruler {} {COPYRIGHT}", version.text()),
            store_failure,
        }
    }

    /// 幾何。描く置き場と当たり判定は同じこれから出す（ARC-001）。
    #[must_use]
    pub const fn layout(&self) -> SettingsLayout {
        self.layout
    }

    /// いまの設定。
    #[must_use]
    pub const fn settings(&self) -> RulerSettings {
        self.settings
    }

    /// 編集状態。
    #[must_use]
    pub const fn session(&self) -> &ModalSession {
        &self.session
    }

    /// 見えに対応する配色。
    #[must_use]
    pub const fn palette(&self) -> ModalPalette {
        ModalPalette::of(self.settings.modal_theme())
    }

    /// 単位のボタンの下に出す換算の説明（`1 logical px = 1.5 physical px`・ADR 0027）。
    #[must_use]
    pub fn ratio_note(&self) -> &str {
        &self.ratio_note
    }

    /// 脚注（版とコピーライト）。
    #[must_use]
    pub fn footer(&self) -> &str {
        &self.footer
    }

    /// 設定の読み書きが失敗していれば、その理由（FR-014）。
    #[must_use]
    pub const fn store_failure(&self) -> Option<SettingsStoreFailure> {
        self.store_failure
    }
}
