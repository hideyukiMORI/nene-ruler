//! 設定モーダルが受ける鍵（Windows）。

/// モーダルが受ける鍵。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalKey {
    /// 閉じる。
    Escape,
    /// 入力欄の末尾を消す。
    Backspace,
    /// 入力欄の文字列を色として送る。
    Return,
}
