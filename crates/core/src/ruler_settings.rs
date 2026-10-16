//! 利用者が選ぶ設定の組。保存される値の正本（FR-010 / FR-012）。

use crate::modal_theme::ModalTheme;
use crate::rgb::Rgb;
use crate::unit::Unit;
use crate::window_layer::WindowLayer;

/// インク色・モーダルの見え・単位・重なり順。不変で、変更は新しい値を返す（ARC-005）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RulerSettings {
    ink: Rgb,
    modal_theme: ModalTheme,
    unit: Unit,
    layer: WindowLayer,
}

impl RulerSettings {
    /// 初期値。既定のインク・ダーク・px・最前面。
    pub const INITIAL: Self = Self {
        ink: Rgb::INITIAL_INK,
        modal_theme: ModalTheme::Dark,
        unit: Unit::Px,
        layer: WindowLayer::AlwaysOnTop,
    };

    /// インク色。
    #[must_use]
    pub const fn ink(self) -> Rgb {
        self.ink
    }

    /// モーダルの見え。
    #[must_use]
    pub const fn modal_theme(self) -> ModalTheme {
        self.modal_theme
    }

    /// 単位。
    #[must_use]
    pub const fn unit(self) -> Unit {
        self.unit
    }

    /// 重なり順。
    #[must_use]
    pub const fn layer(self) -> WindowLayer {
        self.layer
    }

    /// インク色を変えた設定。
    #[must_use]
    pub const fn with_ink(self, ink: Rgb) -> Self {
        Self { ink, ..self }
    }

    /// 見えを変えた設定。
    #[must_use]
    pub const fn with_modal_theme(self, modal_theme: ModalTheme) -> Self {
        Self {
            modal_theme,
            ..self
        }
    }

    /// 単位を変えた設定。
    #[must_use]
    pub const fn with_unit(self, unit: Unit) -> Self {
        Self { unit, ..self }
    }

    /// 重なり順を変えた設定。
    #[must_use]
    pub const fn with_layer(self, layer: WindowLayer) -> Self {
        Self { layer, ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changes_one_field_at_a_time() {
        let changed = RulerSettings::INITIAL
            .with_ink(Rgb::new(1, 2, 3))
            .with_unit(Unit::Mm)
            .with_modal_theme(ModalTheme::Light)
            .with_layer(WindowLayer::Normal);
        assert_eq!(changed.ink(), Rgb::new(1, 2, 3));
        assert_eq!(changed.unit(), Unit::Mm);
        assert_eq!(changed.modal_theme(), ModalTheme::Light);
        assert_eq!(changed.layer(), WindowLayer::Normal);
        assert_eq!(RulerSettings::INITIAL.unit(), Unit::Px);
    }
}
