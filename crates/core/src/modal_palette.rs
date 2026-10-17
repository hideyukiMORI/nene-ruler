//! 設定モーダルの配色。見えごとに 1 組。

use crate::modal_theme::ModalTheme;
use crate::rgb::Rgb;

/// 背景・文字・入力欄・枠線・選択の印。X11 にも cairo にも依存しない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModalPalette {
    background: Rgb,
    text: Rgb,
    field: Rgb,
    border: Rgb,
    accent: Rgb,
}

impl ModalPalette {
    /// 見えに対応する配色。
    #[must_use]
    pub const fn of(theme: ModalTheme) -> Self {
        match theme {
            ModalTheme::Dark => Self {
                // 施主指定 2026-10-17: 真っ黒。
                background: Rgb::new(0x00, 0x00, 0x00),
                text: Rgb::new(0xE6, 0xE6, 0xE6),
                field: Rgb::new(0x2C, 0x2C, 0x2C),
                border: Rgb::new(0x55, 0x55, 0x55),
                accent: Rgb::new(0xFF, 0xFF, 0xFF),
            },
            ModalTheme::Light => Self {
                background: Rgb::new(0xF5, 0xF5, 0xF5),
                text: Rgb::new(0x21, 0x21, 0x21),
                field: Rgb::new(0xFF, 0xFF, 0xFF),
                border: Rgb::new(0xBD, 0xBD, 0xBD),
                accent: Rgb::new(0x21, 0x21, 0x21),
            },
        }
    }

    /// 窓の背景。
    #[must_use]
    pub const fn background(self) -> Rgb {
        self.background
    }

    /// 文字。
    #[must_use]
    pub const fn text(self) -> Rgb {
        self.text
    }

    /// 入力欄と未選択のボタンの地。
    #[must_use]
    pub const fn field(self) -> Rgb {
        self.field
    }

    /// 枠線。
    #[must_use]
    pub const fn border(self) -> Rgb {
        self.border
    }

    /// 選択中の印。
    #[must_use]
    pub const fn accent(self) -> Rgb {
        self.accent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_and_light_differ_in_background() {
        assert_ne!(
            ModalPalette::of(ModalTheme::Dark).background(),
            ModalPalette::of(ModalTheme::Light).background()
        );
        assert_eq!(
            ModalPalette::of(ModalTheme::Dark).background(),
            Rgb::new(0x00, 0x00, 0x00)
        );
    }
}
