//! 設定モーダルの幾何。部品の置き場と当たり判定を物理画素で計算する（FR-009〜FR-011）。
//!
//! 値は 1x の初期値。HiDPI の倍率（FR-016）は後続の Issue で掛ける。

use alloc::vec::Vec;

use crate::modal_label::ModalLabel;
use crate::modal_theme::ModalTheme;
use crate::pixel_point::PixelPoint;
use crate::pixel_rect::PixelRect;
use crate::rgb::Rgb;
use crate::settings_control::SettingsControl;
use crate::settings_hit::SettingsHit;
use crate::unit::Unit;
use crate::window_layer::WindowLayer;

/// 左右と上の余白。
const MARGIN_PX: u16 = 16;
/// 窓の幅。
const WIDTH_PX: u16 = 320;
/// 窓の高さ。
const HEIGHT_PX: u16 = 336;
/// 色見本の一辺。
const SWATCH_PX: u16 = 24;
/// 色見本と色見本の隙間。
const SWATCH_GAP_PX: u16 = 8;
/// ボタンの高さ。
const BUTTON_HEIGHT_PX: u16 = 24;
/// ボタンとボタンの隙間。
const BUTTON_GAP_PX: u16 = 8;
/// 単位のボタンの幅。
const UNIT_BUTTON_PX: u16 = 56;
/// 見えのボタンの幅。
const THEME_BUTTON_PX: u16 = 72;
/// 入力欄の大きさ。
const FIELD_WIDTH_PX: u16 = 120;
const FIELD_HEIGHT_PX: u16 = 26;
/// 各段の上端。見出しのベースラインはその 8 px 上。
const INK_ROW_PX: u16 = 72;
const FIELD_ROW_PX: u16 = 108;
const UNIT_ROW_PX: u16 = 168;
const THEME_ROW_PX: u16 = 224;
const LAYER_ROW_PX: u16 = 272;
/// 見出しと段の間。
const HEADING_GAP_PX: u16 = 8;
/// 題のベースライン。
const TITLE_BASELINE_PX: u16 = 28;
/// 通知（設定の読み書きの失敗）のベースライン。
const NOTICE_BASELINE_PX: u16 = 304;
/// 脚注のベースライン。
const FOOTER_BASELINE_PX: u16 = 322;

/// 設定モーダルの置き場。大きさは固定なのでフィールドを持たない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsLayout;

impl SettingsLayout {
    /// 窓全体。
    #[must_use]
    pub const fn window() -> PixelRect {
        PixelRect::new(0, 0, WIDTH_PX, HEIGHT_PX)
    }

    /// 題「NeNe Ruler」のベースライン。
    #[must_use]
    pub const fn title() -> ModalLabel {
        ModalLabel::new(PixelPoint::new(MARGIN_PX, TITLE_BASELINE_PX), "NeNe Ruler")
    }

    /// 各段の見出し。
    #[must_use]
    pub const fn headings() -> [ModalLabel; 4] {
        [
            ModalLabel::new(
                PixelPoint::new(MARGIN_PX, INK_ROW_PX - HEADING_GAP_PX),
                "Ink",
            ),
            ModalLabel::new(
                PixelPoint::new(MARGIN_PX, UNIT_ROW_PX - HEADING_GAP_PX),
                "Unit",
            ),
            ModalLabel::new(
                PixelPoint::new(MARGIN_PX, THEME_ROW_PX - HEADING_GAP_PX),
                "Settings window",
            ),
            ModalLabel::new(
                PixelPoint::new(MARGIN_PX + SWATCH_PX + BUTTON_GAP_PX, LAYER_ROW_PX + 17),
                "Always on top",
            ),
        ]
    }

    /// 通知の行のベースライン。設定の読み書きの失敗をここに出す（FR-014）。
    #[must_use]
    pub const fn notice_baseline() -> PixelPoint {
        PixelPoint::new(MARGIN_PX, NOTICE_BASELINE_PX)
    }

    /// 脚注（版とコピーライト）のベースライン。文字は application が版から作る。
    #[must_use]
    pub const fn footer_baseline() -> PixelPoint {
        PixelPoint::new(MARGIN_PX, FOOTER_BASELINE_PX)
    }

    /// `index` 番目の色見本。
    #[must_use]
    pub fn preset(index: u8) -> PixelRect {
        PixelRect::new(
            MARGIN_PX + u16::from(index) * (SWATCH_PX + SWATCH_GAP_PX),
            INK_ROW_PX,
            SWATCH_PX,
            SWATCH_PX,
        )
    }

    /// `#RRGGBB` の入力欄。
    #[must_use]
    pub const fn hex_field() -> PixelRect {
        PixelRect::new(MARGIN_PX, FIELD_ROW_PX, FIELD_WIDTH_PX, FIELD_HEIGHT_PX)
    }

    /// 入力欄の文字のベースライン。
    #[must_use]
    pub const fn hex_text_baseline() -> PixelPoint {
        PixelPoint::new(MARGIN_PX + 8, FIELD_ROW_PX + 18)
    }

    /// 入力欄の右に出す拒否の文言のベースライン。
    #[must_use]
    pub const fn hex_notice_baseline() -> PixelPoint {
        PixelPoint::new(
            MARGIN_PX + FIELD_WIDTH_PX + BUTTON_GAP_PX,
            FIELD_ROW_PX + 18,
        )
    }

    /// 単位のボタン。
    #[must_use]
    pub const fn unit_button(unit: Unit) -> PixelRect {
        let index: u16 = match unit {
            Unit::Px => 0,
            Unit::Mm => 1,
            Unit::In => 2,
        };
        PixelRect::new(
            MARGIN_PX + index * (UNIT_BUTTON_PX + BUTTON_GAP_PX),
            UNIT_ROW_PX,
            UNIT_BUTTON_PX,
            BUTTON_HEIGHT_PX,
        )
    }

    /// 単位のボタンの右に出す通知（物理寸法が無く mm / in が効かない）のベースライン。
    #[must_use]
    pub const fn unit_notice_baseline() -> PixelPoint {
        PixelPoint::new(
            MARGIN_PX + 3 * (UNIT_BUTTON_PX + BUTTON_GAP_PX),
            UNIT_ROW_PX + 16,
        )
    }

    /// 見えのボタン。
    #[must_use]
    pub const fn theme_button(theme: ModalTheme) -> PixelRect {
        let index: u16 = match theme {
            ModalTheme::Dark => 0,
            ModalTheme::Light => 1,
        };
        PixelRect::new(
            MARGIN_PX + index * (THEME_BUTTON_PX + BUTTON_GAP_PX),
            THEME_ROW_PX,
            THEME_BUTTON_PX,
            BUTTON_HEIGHT_PX,
        )
    }

    /// 最前面の切替（チェック箱）。
    #[must_use]
    pub const fn layer_toggle() -> PixelRect {
        PixelRect::new(MARGIN_PX, LAYER_ROW_PX, SWATCH_PX, SWATCH_PX)
    }

    /// 当たり判定を優先順に。`layer` はいまの重なり順で、押すと反転する。
    #[must_use]
    pub fn hits(layer: WindowLayer) -> Vec<SettingsHit> {
        let mut hits = Vec::new();
        for index in 0..Rgb::PRESETS.len() {
            let index = u8::try_from(index).unwrap_or(u8::MAX);
            hits.push(SettingsHit::new(
                Self::preset(index),
                SettingsControl::Preset(index),
            ));
        }
        hits.push(SettingsHit::new(
            Self::hex_field(),
            SettingsControl::HexField,
        ));
        for unit in [Unit::Px, Unit::Mm, Unit::In] {
            hits.push(SettingsHit::new(
                Self::unit_button(unit),
                SettingsControl::Unit(unit),
            ));
        }
        for theme in ModalTheme::ALL {
            hits.push(SettingsHit::new(
                Self::theme_button(theme),
                SettingsControl::Theme(theme),
            ));
        }
        hits.push(SettingsHit::new(
            Self::layer_toggle(),
            SettingsControl::Layer(layer.toggled()),
        ));
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_hit::control_at;

    #[test]
    fn everything_fits_inside_the_window() {
        let window = SettingsLayout::window();
        let hits = SettingsLayout::hits(WindowLayer::AlwaysOnTop);
        for hit in &hits {
            assert!(hit.rect().right() <= window.right(), "{hit:?}");
            assert!(hit.rect().bottom() <= window.bottom(), "{hit:?}");
        }
        assert_eq!(hits.len(), 8 + 1 + 3 + 2 + 1);
        assert!(SettingsLayout::footer_baseline().y() < window.height());
        assert!(SettingsLayout::notice_baseline().y() > SettingsLayout::layer_toggle().bottom());
        assert!(
            SettingsLayout::unit_notice_baseline().x()
                > SettingsLayout::unit_button(Unit::In).right()
        );
        assert!(SettingsLayout::unit_notice_baseline().x() < window.width());
        assert!(SettingsLayout::notice_baseline().y() < SettingsLayout::footer_baseline().y());
    }

    #[test]
    fn presets_line_up_without_overlap() {
        assert_eq!(SettingsLayout::preset(0), PixelRect::new(16, 72, 24, 24));
        assert_eq!(SettingsLayout::preset(7), PixelRect::new(240, 72, 24, 24));
        assert!(SettingsLayout::preset(0).right() < SettingsLayout::preset(1).x());
    }

    #[test]
    fn clicks_map_to_controls() {
        let hits = SettingsLayout::hits(WindowLayer::AlwaysOnTop);
        assert_eq!(
            control_at(&hits, PixelPoint::new(50, 80)),
            Some(SettingsControl::Preset(1))
        );
        assert_eq!(
            control_at(&hits, PixelPoint::new(20, 110)),
            Some(SettingsControl::HexField)
        );
        assert_eq!(
            control_at(&hits, PixelPoint::new(90, 170)),
            Some(SettingsControl::Unit(Unit::Mm))
        );
        assert_eq!(
            control_at(&hits, PixelPoint::new(100, 230)),
            Some(SettingsControl::Theme(ModalTheme::Light))
        );
        assert_eq!(
            control_at(&hits, PixelPoint::new(20, 280)),
            Some(SettingsControl::Layer(WindowLayer::Normal))
        );
        assert_eq!(control_at(&hits, PixelPoint::new(300, 300)), None);
    }

    #[test]
    fn headings_sit_above_their_rows() {
        let [ink, unit, theme, layer] = SettingsLayout::headings();
        assert!(ink.baseline().y() < SettingsLayout::preset(0).y());
        assert!(unit.baseline().y() < SettingsLayout::unit_button(Unit::Px).y());
        assert!(theme.baseline().y() < SettingsLayout::theme_button(ModalTheme::Dark).y());
        assert_eq!(layer.text(), "Always on top");
        assert_eq!(SettingsLayout::title().text(), "NeNe Ruler");
    }
}
