//! 字の横の置き方。

/// ベースラインの点を、字の左端に置くか中央に置くか。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    /// 点が字の左端。
    Left,
    /// 点が字の中央。
    Centre,
}
