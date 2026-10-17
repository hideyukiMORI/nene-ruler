//! CLIPBOARD へ文字列を置くポート（ARC-003 / ADR 0007）。実装は adapters-x11。

use crate::clipboard_failure::ClipboardFailure;

/// 表示しているその文字列をそのまま置く（FR-007）。
pub trait ClipboardPort {
    /// 文字列の所有を取る。戻ったときには他のアプリが貼り付けられる状態である。
    ///
    /// # Errors
    ///
    /// 待ち受けが無い・所有を取れなかった。
    fn copy(&self, text: &str) -> Result<(), ClipboardFailure>;
}
