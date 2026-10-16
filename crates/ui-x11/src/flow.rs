//! イベントを処理した後の分岐。

/// 続けるか、閉じるか、設定モーダルを開くか。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Flow {
    /// 次のイベントを待つ。
    Continue,
    /// 窓を閉じて戻る。
    Quit,
    /// ギアが押された。設定モーダルを開く。
    OpenSettings,
}
