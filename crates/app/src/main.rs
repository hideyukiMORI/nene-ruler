//! `NeNe Ruler` の合成ルート（ARC-006）。
//!
//! ポートに実装を結び、UI を起動する唯一の場所。端末への出力とプロセスの終了コードを
//! 扱ってよいのもここだけである（RS-014）。出してよいのは起動できなかった理由 1 行だけで、
//! ログの手段として使わない。

use std::fmt;
use std::process::ExitCode;

use nene_ruler_adapters_x11::adapter_failure::AdapterFailure;
use nene_ruler_adapters_x11::file_settings_store::FileSettingsStore;
use nene_ruler_adapters_x11::randr_screen_metrics::RandrScreenMetrics;
use nene_ruler_adapters_x11::x11_clipboard::X11Clipboard;
use nene_ruler_application::ports::Ports;
use nene_ruler_application::product_version::ProductVersion;
use nene_ruler_application::ruler_controller::RulerController;
use nene_ruler_application::settings_store_failure::SettingsStoreFailure;
use nene_ruler_ui_x11::display::Display;
use nene_ruler_ui_x11::event_loop;
use nene_ruler_ui_x11::ui_failure::UiFailure;

/// 製品の版。`Cargo.toml` の `[workspace.package] version` から cargo が渡す唯一の経路（FR-011）。
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 起動できなかった理由。閉じた集合。
#[derive(Debug)]
enum StartupFailure {
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
fn run() -> Result<(), StartupFailure> {
    let metrics = RandrScreenMetrics::connect().map_err(StartupFailure::Metrics)?;
    let store = FileSettingsStore::in_xdg_config().map_err(StartupFailure::Store)?;
    let clipboard = X11Clipboard::connect().map_err(StartupFailure::Clipboard)?;
    let display = Display::connect().map_err(StartupFailure::Ui)?;
    let mut controller = RulerController::new(
        Ports::new(&metrics, &store, &clipboard),
        display.transparency(),
        ProductVersion::new(VERSION),
    );
    event_loop::run(&display, &mut controller).map_err(StartupFailure::Ui)
}

// 🔴 標準エラーへ書いてよい唯一の場所（RS-014）。この #[expect] が宣言そのものである。
#[expect(
    clippy::print_stderr,
    reason = "RS-014: 端末への出力は合成ルートだけが行う"
)]
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("NeNe Ruler {VERSION}: {failure}");
            ExitCode::FAILURE
        }
    }
}
