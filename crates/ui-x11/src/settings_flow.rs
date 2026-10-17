//! 設定モーダルのイベントを処理した後の分岐。

/// 続けるか、設定が変わったか、閉じるか。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsFlow {
    /// 何も変わっていない。
    Continue,
    /// 意図を渡した。枠とモーダルを描き直す。
    Changed,
    /// モーダルを閉じる。
    Close,
}
