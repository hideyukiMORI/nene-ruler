//! 設定モーダルに置く文字と、そのベースライン。

use crate::pixel_point::PixelPoint;

/// 見出し・部品の語・脚注。UI は英語だけ（SPECIFICATION 第 3 節）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModalLabel {
    baseline: PixelPoint,
    text: &'static str,
}

impl ModalLabel {
    /// 位置と文字から作る。
    #[must_use]
    pub const fn new(baseline: PixelPoint, text: &'static str) -> Self {
        Self { baseline, text }
    }

    /// 文字の左端のベースライン。
    #[must_use]
    pub const fn baseline(self) -> PixelPoint {
        self.baseline
    }

    /// 文字。
    #[must_use]
    pub const fn text(self) -> &'static str {
        self.text
    }
}
