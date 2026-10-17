//! Windows の合成ルート（ARC-006・ADR 0021）。ポートに Windows のアダプタを結び、Windows の画面を起動する。

use std::fmt;

use nene_ruler_adapters_windows::windows_clipboard::WindowsClipboard;
use nene_ruler_adapters_windows::windows_screen_metrics::WindowsScreenMetrics;
use nene_ruler_application::environment::Environment;
use nene_ruler_application::ports::Ports;
use nene_ruler_application::product_version::ProductVersion;
use nene_ruler_application::ruler_controller::RulerController;
use nene_ruler_application::settings_store_failure::SettingsStoreFailure;
use nene_ruler_application::transparency::Transparency;
use nene_ruler_settings_file::file_settings_store::FileSettingsStore;
use nene_ruler_ui_win32::frame_window;
use nene_ruler_ui_win32::win32_failure::Win32Failure;

/// 起動できなかった理由。閉じた集合。
#[derive(Debug)]
pub(crate) enum StartupFailure {
    /// 設定の置き場が決まらなかった（`APPDATA` が無い）。
    Store(SettingsStoreFailure),
    /// 画面側が失敗した。
    Ui(Win32Failure),
}

impl fmt::Display for StartupFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(failure) => formatter.write_str(failure.describe()),
            Self::Ui(error) => write!(formatter, "{error}"),
        }
    }
}

/// 依存を結んで走らせる。dpi 対応を先に宣言してから、画面の大きさと dpi を読む（FR-016）。
pub(crate) fn run(version: &'static str) -> Result<(), StartupFailure> {
    frame_window::declare_dpi_awareness();
    let metrics = WindowsScreenMetrics;
    let store = FileSettingsStore::in_app_data().map_err(StartupFailure::Store)?;
    let clipboard = WindowsClipboard;
    // Windows の layered 窓は常に透明に描ける（合成は OS が持つ）。
    let environment = Environment::new(Transparency::Composited)
        .with_desktop_dpi(WindowsScreenMetrics::desktop_dpi());
    let mut controller = RulerController::new(
        Ports::new(&metrics, &store, &clipboard),
        environment,
        ProductVersion::new(version),
    );
    frame_window::run(&mut controller).map_err(StartupFailure::Ui)
}
