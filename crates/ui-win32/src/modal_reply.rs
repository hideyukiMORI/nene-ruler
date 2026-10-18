//! モーダル宛のメッセージを処理した後に、窓の側ですること。

/// 状態を変えたか・窓をどうするか。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalReply {
    /// 何も変わっていない。
    Unchanged,
    /// 設定か編集状態が変わった。枠とモーダルを描き直す。
    Changed,
    /// 最前面の設定が変わった。枠の窓の重なり順を変えて描き直す。
    LayerChanged,
    /// モーダルを閉じる（× と Esc）。
    Close,
    /// アプリを終える（Quit）。
    Quit,
}
