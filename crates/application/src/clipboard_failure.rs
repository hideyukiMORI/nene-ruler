//! クリップボードへ置けなかった理由（FR-007 / FR-014）。

/// 閉じた集合。利用者には数字の上の印で示す（ADR 0007 の 5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardFailure {
    /// 接続か待ち受けが無い（起動時に繋げなかった・切れた）。
    Unavailable,
    /// 所有を取れなかった（X が別の所有者を返した）。
    Rejected,
}
