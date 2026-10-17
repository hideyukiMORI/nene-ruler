//! 設定モーダルの幾何。部品の置き場と当たり判定を物理画素で計算する（FR-009〜FR-011）。
//!
//! 値は 1x の初期値で、すべてに UI の倍率を掛ける（FR-016・ADR 0011）。

use alloc::vec::Vec;

use crate::modal_label::ModalLabel;
use crate::modal_theme::ModalTheme;
use crate::pixel_point::PixelPoint;
use crate::pixel_rect::PixelRect;
use crate::rgb::Rgb;
use crate::settings_control::SettingsControl;
use crate::settings_hit::SettingsHit;
use crate::ui_scale::UiScale;
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

/// 設定モーダルの置き場。大きさは倍率だけで決まる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsLayout {
    scale: UiScale,
}

impl SettingsLayout {
    /// UI の倍率から作る。
    #[must_use]
    pub const fn new(scale: UiScale) -> Self {
        Self { scale }
    }

    /// UI の倍率。字の大きさを決めるのに UI が読む。
    #[must_use]
    pub const fn scale(self) -> UiScale {
        self.scale
    }

    /// 1x の寸法に倍率を掛ける。
    const fn px(self, base: u16) -> u16 {
        base * self.scale.factor()
    }

    /// 1x の点に倍率を掛ける。
    const fn point(self, x: u16, y: u16) -> PixelPoint {
        PixelPoint::new(self.px(x), self.px(y))
    }

    /// 1x の矩形に倍率を掛ける。
    const fn scaled(self, rect: PixelRect) -> PixelRect {
        PixelRect::new(
            self.px(rect.x()),
            self.px(rect.y()),
            self.px(rect.width()),
            self.px(rect.height()),
        )
    }

    /// 窓全体。
    #[must_use]
    pub const fn window(self) -> PixelRect {
        self.scaled(PixelRect::new(0, 0, WIDTH_PX, HEIGHT_PX))
    }

    /// 題「NeNe Ruler」のベースライン。
    #[must_use]
    pub const fn title(self) -> ModalLabel {
        ModalLabel::new(self.point(MARGIN_PX, TITLE_BASELINE_PX), "NeNe Ruler")
    }

    /// 各段の見出し。
    #[must_use]
    pub const fn headings(self) -> [ModalLabel; 4] {
        [
            ModalLabel::new(self.point(MARGIN_PX, INK_ROW_PX - HEADING_GAP_PX), "Ink"),
            ModalLabel::new(self.point(MARGIN_PX, UNIT_ROW_PX - HEADING_GAP_PX), "Unit"),
            ModalLabel::new(
                self.point(MARGIN_PX, THEME_ROW_PX - HEADING_GAP_PX),
                "Settings window",
            ),
            ModalLabel::new(
                self.point(MARGIN_PX + SWATCH_PX + BUTTON_GAP_PX, LAYER_ROW_PX + 17),
                "Always on top",
            ),
        ]
    }

    /// 通知の行のベースライン。設定の読み書きの失敗をここに出す（FR-014）。
    #[must_use]
    pub const fn notice_baseline(self) -> PixelPoint {
        self.point(MARGIN_PX, NOTICE_BASELINE_PX)
    }

    /// 脚注（版とコピーライト）のベースライン。文字は application が版から作る。
    #[must_use]
    pub const fn footer_baseline(self) -> PixelPoint {
        self.point(MARGIN_PX, FOOTER_BASELINE_PX)
    }

    /// `index` 番目の色見本。
    #[must_use]
    pub fn preset(self, index: u8) -> PixelRect {
        self.scaled(PixelRect::new(
            MARGIN_PX + u16::from(index) * (SWATCH_PX + SWATCH_GAP_PX),
            INK_ROW_PX,
            SWATCH_PX,
            SWATCH_PX,
        ))
    }

    /// `#RRGGBB` の入力欄。
    #[must_use]
    pub const fn hex_field(self) -> PixelRect {
        self.scaled(PixelRect::new(
            MARGIN_PX,
            FIELD_ROW_PX,
            FIELD_WIDTH_PX,
            FIELD_HEIGHT_PX,
        ))
    }

    /// 入力欄の文字のベースライン。
    #[must_use]
    pub const fn hex_text_baseline(self) -> PixelPoint {
        self.point(MARGIN_PX + 8, FIELD_ROW_PX + 18)
    }

    /// 入力欄の右に出す拒否の文言のベースライン。
    #[must_use]
    pub const fn hex_notice_baseline(self) -> PixelPoint {
        self.point(
            MARGIN_PX + FIELD_WIDTH_PX + BUTTON_GAP_PX,
            FIELD_ROW_PX + 18,
        )
    }

    /// 単位のボタン。
    #[must_use]
    pub const fn unit_button(self, unit: Unit) -> PixelRect {
        let index: u16 = match unit {
            Unit::Px => 0,
            Unit::Mm => 1,
            Unit::In => 2,
        };
        self.scaled(PixelRect::new(
            MARGIN_PX + index * (UNIT_BUTTON_PX + BUTTON_GAP_PX),
            UNIT_ROW_PX,
            UNIT_BUTTON_PX,
            BUTTON_HEIGHT_PX,
        ))
    }

    /// 単位のボタンの右に出す通知（物理寸法が無く mm / in が効かない）のベースライン。
    #[must_use]
    pub const fn unit_notice_baseline(self) -> PixelPoint {
        self.point(
            MARGIN_PX + 3 * (UNIT_BUTTON_PX + BUTTON_GAP_PX),
            UNIT_ROW_PX + 16,
        )
    }

    /// 「Always on top」の右に出す通知（WM が最前面に対応しない）のベースライン。
    #[must_use]
    pub const fn layer_notice_baseline(self) -> PixelPoint {
        self.point(
            MARGIN_PX + SWATCH_PX + BUTTON_GAP_PX + 104,
            LAYER_ROW_PX + 17,
        )
    }

    /// 見えのボタン。
    #[must_use]
    pub const fn theme_button(self, theme: ModalTheme) -> PixelRect {
        let index: u16 = match theme {
            ModalTheme::Dark => 0,
            ModalTheme::Light => 1,
        };
        self.scaled(PixelRect::new(
            MARGIN_PX + index * (THEME_BUTTON_PX + BUTTON_GAP_PX),
            THEME_ROW_PX,
            THEME_BUTTON_PX,
            BUTTON_HEIGHT_PX,
        ))
    }

    /// 最前面の切替（チェック箱）。
    #[must_use]
    pub const fn layer_toggle(self) -> PixelRect {
        self.scaled(PixelRect::new(
            MARGIN_PX,
            LAYER_ROW_PX,
            SWATCH_PX,
            SWATCH_PX,
        ))
    }

    /// 当たり判定を優先順に。`layer` はいまの重なり順で、押すと反転する。
    #[must_use]
    pub fn hits(self, layer: WindowLayer) -> Vec<SettingsHit> {
        let mut hits = Vec::new();
        for index in 0..Rgb::PRESETS.len() {
            let index = u8::try_from(index).unwrap_or(u8::MAX);
            hits.push(SettingsHit::new(
                self.preset(index),
                SettingsControl::Preset(index),
            ));
        }
        hits.push(SettingsHit::new(
            self.hex_field(),
            SettingsControl::HexField,
        ));
        for unit in [Unit::Px, Unit::Mm, Unit::In] {
            hits.push(SettingsHit::new(
                self.unit_button(unit),
                SettingsControl::Unit(unit),
            ));
        }
        for theme in ModalTheme::ALL {
            hits.push(SettingsHit::new(
                self.theme_button(theme),
                SettingsControl::Theme(theme),
            ));
        }
        hits.push(SettingsHit::new(
            self.layer_toggle(),
            SettingsControl::Layer(layer.toggled()),
        ));
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_hit::control_at;

    const SINGLE: SettingsLayout = SettingsLayout::new(UiScale::Single);

    #[test]
    fn everything_fits_inside_the_window() {
        let window = SINGLE.window();
        let hits = SINGLE.hits(WindowLayer::AlwaysOnTop);
        for hit in &hits {
            assert!(hit.rect().right() <= window.right(), "{hit:?}");
            assert!(hit.rect().bottom() <= window.bottom(), "{hit:?}");
        }
        assert_eq!(hits.len(), 8 + 1 + 3 + 2 + 1);
    }

    #[test]
    fn notices_sit_inside_the_window_and_clear_of_the_controls() {
        let window = SINGLE.window();
        assert!(SINGLE.footer_baseline().y() < window.height());
        assert!(SINGLE.notice_baseline().y() > SINGLE.layer_toggle().bottom());
        assert!(SINGLE.unit_notice_baseline().x() > SINGLE.unit_button(Unit::In).right());
        assert!(SINGLE.unit_notice_baseline().x() < window.width());
        assert!(SINGLE.layer_notice_baseline().x() < window.width());
        assert!(SINGLE.notice_baseline().y() < SINGLE.footer_baseline().y());
    }

    #[test]
    fn presets_line_up_without_overlap() {
        assert_eq!(SINGLE.preset(0), PixelRect::new(16, 72, 24, 24));
        assert_eq!(SINGLE.preset(7), PixelRect::new(240, 72, 24, 24));
        assert!(SINGLE.preset(0).right() < SINGLE.preset(1).x());
    }

    #[test]
    fn clicks_map_to_controls() {
        let hits = SINGLE.hits(WindowLayer::AlwaysOnTop);
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
        let [ink, unit, theme, layer] = SINGLE.headings();
        assert!(ink.baseline().y() < SINGLE.preset(0).y());
        assert!(unit.baseline().y() < SINGLE.unit_button(Unit::Px).y());
        assert!(theme.baseline().y() < SINGLE.theme_button(ModalTheme::Dark).y());
        assert_eq!(layer.text(), "Always on top");
        assert_eq!(SINGLE.title().text(), "NeNe Ruler");
    }

    #[test]
    fn double_is_exactly_twice_the_single() {
        let double = SettingsLayout::new(UiScale::Double);
        assert_eq!(double.window(), PixelRect::new(0, 0, 640, 672));
        assert_eq!(double.preset(7), PixelRect::new(480, 144, 48, 48));
        assert_eq!(
            double.layer_notice_baseline(),
            PixelPoint::new(
                2 * SINGLE.layer_notice_baseline().x(),
                2 * SINGLE.layer_notice_baseline().y()
            )
        );
        let hits = double.hits(WindowLayer::AlwaysOnTop);
        assert_eq!(
            control_at(&hits, PixelPoint::new(180, 340)),
            Some(SettingsControl::Unit(Unit::Mm))
        );
        assert_eq!(control_at(&hits, PixelPoint::new(90, 170)), None);
    }
}
