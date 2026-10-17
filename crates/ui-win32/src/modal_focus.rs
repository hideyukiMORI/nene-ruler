//! 設定モーダルにキーボードの焦点があるか（Issue #40 の Windows 版）。

/// 焦点があれば入力欄に棒（キャレット）と選択色の縁を描く。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalFocus {
    /// モーダルが焦点を持つ。打った鍵は入力欄に入る。
    Focused,
    /// 焦点が無い。
    Unfocused,
}
