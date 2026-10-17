//! 設定の読み書きに失敗した理由（ARC-009 / FR-014）。

/// 「無い」は失敗ではなく `Ok(None)`。ここにあるのは示すべき失敗だけ。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsStoreFailure {
    /// 置き場が決まらない（`XDG_CONFIG_HOME` も `HOME` も無い）。
    NoConfigDirectory,
    /// ファイルはあるが読めない（権限・入出力）。
    Unreadable,
    /// 読めたが、知らない版。
    UnknownVersion,
    /// 読めたが、形が違う（版の行が無い・鍵が足りない・値が読めない）。
    Malformed,
    /// 書けない（ディレクトリが作れない・権限・入出力）。
    Unwritable,
}

impl SettingsStoreFailure {
    /// 利用者に見せる短い文。
    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Self::NoConfigDirectory => "settings: no config directory (XDG_CONFIG_HOME / HOME)",
            Self::Unreadable => "settings: file could not be read; using defaults",
            Self::UnknownVersion => "settings: unknown file version; using defaults",
            Self::Malformed => "settings: file is malformed; using defaults",
            Self::Unwritable => "settings: could not be saved",
        }
    }
}
