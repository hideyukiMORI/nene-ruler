//! X11（Linux）の合成ルート（ARC-006）。ポートに X11 のアダプタを結び、X11 の画面を起動する。

use std::fmt;

use nene_ruler_adapters_x11::adapter_failure::AdapterFailure;
use nene_ruler_adapters_x11::randr_screen_metrics::RandrScreenMetrics;
use nene_ruler_adapters_x11::x11_clipboard::X11Clipboard;
use nene_ruler_application::ports::Ports;
use nene_ruler_application::product_version::ProductVersion;
use nene_ruler_application::ruler_controller::RulerController;
use nene_ruler_application::settings_store_failure::SettingsStoreFailure;
use nene_ruler_settings_file::file_settings_store::FileSettingsStore;
use nene_ruler_ui_x11::display::Display;
use nene_ruler_ui_x11::event_loop;
use nene_ruler_ui_x11::ui_failure::UiFailure;

/// 起動できなかった理由。閉じた集合。
#[derive(Debug)]
pub(crate) enum StartupFailure {
    /// 物理寸法のアダプタが X に繋げなかった。
    Metrics(AdapterFailure),
    /// クリップボードのアダプタが X に繋げなかった、または窓を作れなかった。
    Clipboard(AdapterFailure),
    /// 設定の置き場が決まらなかった。
    Store(SettingsStoreFailure),
    /// 画面側が失敗した。
    Ui(UiFailure),
}

impl fmt::Display for StartupFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Metrics(error) => write!(formatter, "RandR adapter: {error}"),
            Self::Clipboard(error) => write!(formatter, "clipboard adapter: {error}"),
            Self::Store(failure) => formatter.write_str(failure.describe()),
            Self::Ui(error) => write!(formatter, "{error}"),
        }
    }
}

/// 依存を結んで走らせる。
pub(crate) fn run(version: &'static str) -> Result<(), StartupFailure> {
    let metrics = RandrScreenMetrics::connect().map_err(StartupFailure::Metrics)?;
    let store = FileSettingsStore::in_xdg_config().map_err(StartupFailure::Store)?;
    let clipboard = X11Clipboard::connect().map_err(StartupFailure::Clipboard)?;
    let display = Display::connect().map_err(StartupFailure::Ui)?;
    let mut controller = RulerController::new(
        Ports::new(&metrics, &store, &clipboard),
        display.environment(),
        ProductVersion::new(version),
    );
    event_loop::run(&display, &mut controller).map_err(StartupFailure::Ui)
}
