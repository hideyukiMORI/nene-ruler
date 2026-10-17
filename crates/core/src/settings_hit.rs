//! 設定モーダルの当たり判定の矩形と部品の対。

use crate::pixel_point::PixelPoint;
use crate::pixel_rect::PixelRect;
use crate::settings_control::SettingsControl;

/// クリックを受ける矩形と、その部品。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsHit {
    rect: PixelRect,
    control: SettingsControl,
}

impl SettingsHit {
    /// 矩形と部品から作る。
    #[must_use]
    pub const fn new(rect: PixelRect, control: SettingsControl) -> Self {
        Self { rect, control }
    }

    /// 矩形。
    #[must_use]
    pub const fn rect(self) -> PixelRect {
        self.rect
    }

    /// 部品。
    #[must_use]
    pub const fn control(self) -> SettingsControl {
        self.control
    }
}

/// 点を含む最初の部品。
#[must_use]
pub fn control_at(hits: &[SettingsHit], point: PixelPoint) -> Option<SettingsControl> {
    hits.iter()
        .find(|hit| hit.rect.contains(point))
        .map(|hit| hit.control())
}
