//! `NeNe Ruler` の合成ルート（ARC-006）。
//!
//! ポートに実装を結び、UI を起動する唯一の場所。端末への出力とプロセスの終了コードを
//! 扱ってよいのもここだけである（RS-014）。出してよいのは起動できなかった理由 1 行だけで、
//! ログの手段として使わない。

use std::fmt;
use std::process::ExitCode;

use nene_ruler_adapters_x11::adapter_failure::AdapterFailure;
use nene_ruler_adapters_x11::randr_screen_metrics::RandrScreenMetrics;
use nene_ruler_application::ruler_controller::RulerController;
use nene_ruler_ui_x11::display::Display;
use nene_ruler_ui_x11::ruler_window::RulerWindow;
use nene_ruler_ui_x11::ui_failure::UiFailure;

/// 製品の版。`Cargo.toml` の `[workspace.package] version` から cargo が渡す唯一の経路（FR-011）。
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 起動できなかった理由。閉じた集合。
#[derive(Debug)]
enum StartupFailure {
    /// 物理寸法のアダプタが X に繋げなかった。
    Metrics(AdapterFailure),
    /// 画面側が失敗した。
    Ui(UiFailure),
}

impl fmt::Display for StartupFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Metrics(error) => write!(formatter, "RandR adapter: {error}"),
            Self::Ui(error) => write!(formatter, "{error}"),
        }
    }
}

/// 依存を結んで走らせる。
fn run() -> Result<(), StartupFailure> {
    let metrics = RandrScreenMetrics::connect().map_err(StartupFailure::Metrics)?;
    let display = Display::connect().map_err(StartupFailure::Ui)?;
    let mut controller = RulerController::new(&metrics, display.transparency());
    let mut window =
        RulerWindow::open(&display, &controller.frame()).map_err(StartupFailure::Ui)?;
    window.run(&mut controller).map_err(StartupFailure::Ui)
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
