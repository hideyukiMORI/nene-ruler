//! 色の文字列を読めなかった理由。

/// `Rgb::parse` が拒んだ理由。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RgbFailure {
    /// `#` で始まっていない。
    MissingHash,
    /// `#` の後が 6 桁ではない。
    WrongLength,
    /// 16 進でない文字がある。
    NotHexadecimal,
}
