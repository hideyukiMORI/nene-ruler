//! `NeNe Ruler` の合成ルート（ARC-006）。
//!
//! ポートに実装を結び、UI を起動する唯一の場所。端末への出力とプロセスの終了コードを
//! 扱ってよいのもここだけである（RS-014）。出してよいのは起動できなかった理由 1 行だけで、
//! ログの手段として使わない。
//!
//! Issue #1 の時点では製品モジュールが無く、起動できない理由をそのまま返す。

use std::process::ExitCode;

/// 製品の版。`Cargo.toml` の `[workspace.package] version` から cargo が渡す唯一の経路（FR-011）。
const VERSION: &str = env!("CARGO_PKG_VERSION");

// 🔴 標準エラーへ書いてよい唯一の場所（RS-014）。この #[expect] が宣言そのものである。
#[expect(
    clippy::print_stderr,
    reason = "RS-014: 端末への出力は合成ルートだけが行う"
)]
fn main() -> ExitCode {
    eprintln!("NeNe Ruler {VERSION}: the window is not implemented yet (Issue #2)");
    ExitCode::FAILURE
}
