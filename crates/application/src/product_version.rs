//! 製品の版。`Cargo.toml` の `[workspace.package] version` から合成ルートが 1 か所で渡す（FR-011）。

/// 版の文字列。`env!("CARGO_PKG_VERSION")` の値を app が包む。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductVersion(&'static str);

impl ProductVersion {
    /// 文字列から作る。
    #[must_use]
    pub const fn new(text: &'static str) -> Self {
        Self(text)
    }

    /// そのままの文字列。
    #[must_use]
    pub const fn text(self) -> &'static str {
        self.0
    }
}
