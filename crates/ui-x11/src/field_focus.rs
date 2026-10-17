//! 設定モーダルにキーボードの焦点があるか（Issue #40）。UI の区画の状態（ARC-005）。

/// 焦点があれば入力欄に棒（キャレット）と選択色の縁を描く。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldFocus {
    /// モーダルが焦点を持つ。打った鍵は入力欄に入る。
    Focused,
    /// 焦点が無い。打っても入らない。
    Unfocused,
}
