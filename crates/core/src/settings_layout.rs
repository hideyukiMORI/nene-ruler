//! 設定モーダルの幾何。部品の置き場と当たり判定を物理画素で計算する（FR-009〜FR-011）。
//!
//! 値は 1x の初期値で、すべてに UI の倍率を掛ける（FR-016・ADR 0011）。

use alloc::vec::Vec;

use crate::hsv::Hsv;
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
const HEIGHT_PX: u16 = 448;
/// 色見本の一辺。
const SWATCH_PX: u16 = 24;
/// 色見本と色見本の隙間。
const SWATCH_GAP_PX: u16 = 8;
/// ボタンの高さ。
const BUTTON_HEIGHT_PX: u16 = 24;
/// ボタンとボタンの隙間。
const BUTTON_GAP_PX: u16 = 8;
/// 単位のボタンの幅（`Physical px` が入る）。
const UNIT_BUTTON_PX: u16 = 96;
/// 見えのボタンの幅。
const THEME_BUTTON_PX: u16 = 72;
/// 入力欄の大きさ。
const FIELD_WIDTH_PX: u16 = 120;
const FIELD_HEIGHT_PX: u16 = 26;
/// 各段の上端。見出しのベースラインはその 8 px 上。
const INK_ROW_PX: u16 = 72;
const FIELD_ROW_PX: u16 = 108;
const PICKER_ROW_PX: u16 = 144;
const UNIT_ROW_PX: u16 = 264;
/// 換算の説明のベースライン。単位のボタンの下の行。
const RATIO_BASELINE_PX: u16 = 306;
const THEME_ROW_PX: u16 = 336;
const LAYER_ROW_PX: u16 = 384;
/// カラーピッカーの高さ・四角の幅・帯の幅（ADR 0016）。
const PICKER_HEIGHT_PX: u16 = 96;
const SHADE_WIDTH_PX: u16 = 200;
const HUE_BAR_WIDTH_PX: u16 = 20;
/// 色相の 1 周（0.1 度刻み）の最後の値。
const LAST_HUE: u32 = 3599;
/// 彩度・明度の最大（千分率）。
const PER_MILLE: u32 = 1000;
/// 見出しと段の間。
const HEADING_GAP_PX: u16 = 8;
/// 終了ボタンの幅。
const QUIT_BUTTON_PX: u16 = 56;
/// 閉じるボタンの一辺と、窓の右上の角からの距離。
const CLOSE_PX: u16 = 24;
const CLOSE_INSET_PX: u16 = 8;
/// 題のベースライン。
const TITLE_BASELINE_PX: u16 = 28;
/// 通知（設定の読み書きの失敗）のベースライン。
const NOTICE_BASELINE_PX: u16 = 416;
/// 脚注のベースライン。
const FOOTER_BASELINE_PX: u16 = 434;

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

    /// カラーピッカーの彩度 × 明度の四角。左が彩度 0、上が明度 1000。
    #[must_use]
    pub const fn shade_area(self) -> PixelRect {
        self.scaled(PixelRect::new(
            MARGIN_PX,
            PICKER_ROW_PX,
            SHADE_WIDTH_PX,
            PICKER_HEIGHT_PX,
        ))
    }

    /// カラーピッカーの色相の帯。上が 0 度（赤）で、下へ 1 周する。
    #[must_use]
    pub const fn hue_bar(self) -> PixelRect {
        self.scaled(PixelRect::new(
            MARGIN_PX + SHADE_WIDTH_PX + BUTTON_GAP_PX,
            PICKER_ROW_PX,
            HUE_BAR_WIDTH_PX,
            PICKER_HEIGHT_PX,
        ))
    }

    /// 四角の中の位置の彩度と明度（千分率）。四角の外は縁へ寄せる（引いて外へ出たとき）。
    #[must_use]
    pub fn shade_at(self, point: PixelPoint) -> (u16, u16) {
        let area = self.shade_area();
        let across = fraction(point.x(), area.x(), area.width());
        let down = fraction(point.y(), area.y(), area.height());
        (narrow(across), narrow(PER_MILLE - down))
    }

    /// 帯の中の位置の色相（0.1 度）。帯の外は端へ寄せる。
    #[must_use]
    pub fn hue_at(self, point: PixelPoint) -> u16 {
        let strip = self.hue_bar();
        let down = fraction(point.y(), strip.y(), strip.height());
        narrow(down * LAST_HUE / PER_MILLE)
    }

    /// 色の、四角の中の印の位置。
    #[must_use]
    pub fn shade_point(self, colour: Hsv) -> PixelPoint {
        let area = self.shade_area();
        let span_x = u32::from(area.width().saturating_sub(1));
        let span_y = u32::from(area.height().saturating_sub(1));
        PixelPoint::new(
            area.x() + narrow(u32::from(colour.saturation()) * span_x / PER_MILLE),
            area.y() + narrow((PER_MILLE - u32::from(colour.value())) * span_y / PER_MILLE),
        )
    }

    /// 色相の、帯の中の印の高さ。
    #[must_use]
    pub fn hue_y(self, colour: Hsv) -> u16 {
        let strip = self.hue_bar();
        let span = u32::from(strip.height().saturating_sub(1));
        strip.y() + narrow(u32::from(colour.hue()) * span / LAST_HUE)
    }

    /// 単位のボタン（FR-008・ADR 0027）。
    #[must_use]
    pub const fn unit_button(self, unit: Unit) -> PixelRect {
        let index: u16 = match unit {
            Unit::Physical => 0,
            Unit::Logical => 1,
        };
        self.scaled(PixelRect::new(
            MARGIN_PX + index * (UNIT_BUTTON_PX + BUTTON_GAP_PX),
            UNIT_ROW_PX,
            UNIT_BUTTON_PX,
            BUTTON_HEIGHT_PX,
        ))
    }

    /// 単位のボタンの下に出す換算（`1 logical px = 1.5 physical px`）のベースライン。文字は application が作る。
    #[must_use]
    pub const fn ratio_baseline(self) -> PixelPoint {
        self.point(MARGIN_PX, RATIO_BASELINE_PX)
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

    /// 右上の閉じるボタン。窓に飾りが無いので自前で持つ（ADR 0014）。
    #[must_use]
    pub const fn close_button(self) -> PixelRect {
        self.scaled(PixelRect::new(
            WIDTH_PX - CLOSE_INSET_PX - CLOSE_PX,
            CLOSE_INSET_PX,
            CLOSE_PX,
            CLOSE_PX,
        ))
    }

    /// 終了ボタン。「Always on top」の段の右端（Issue #42）。
    #[must_use]
    pub const fn quit_button(self) -> PixelRect {
        self.scaled(PixelRect::new(
            WIDTH_PX - MARGIN_PX - QUIT_BUTTON_PX,
            LAYER_ROW_PX,
            QUIT_BUTTON_PX,
            BUTTON_HEIGHT_PX,
        ))
    }

    /// 当たり判定を優先順に。`layer` はいまの重なり順で、押すと反転する。
    /// どれにも当たらない所は、窓を動かす掴み所になる（ADR 0014）。
    #[must_use]
    pub fn hits(self, layer: WindowLayer) -> Vec<SettingsHit> {
        let mut hits = Vec::new();
        hits.push(SettingsHit::new(
            self.close_button(),
            SettingsControl::Close,
        ));
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
        hits.push(SettingsHit::new(self.shade_area(), SettingsControl::Shade));
        hits.push(SettingsHit::new(self.hue_bar(), SettingsControl::Hue));
        for unit in Unit::ALL {
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
        hits.push(SettingsHit::new(self.quit_button(), SettingsControl::Quit));
        hits
    }
}

/// 始まり `origin` から長さ `span` の範囲での `at` の位置を千分率で。範囲の外は 0 か 1000 に寄せる。
fn fraction(at: u16, origin: u16, span: u16) -> u32 {
    let last = u32::from(span.saturating_sub(1)).max(1);
    let offset = u32::from(at.saturating_sub(origin)).min(last);
    offset * PER_MILLE / last
}

/// 範囲が分かっている値を `u16` に。
fn narrow(value: u32) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
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
        assert_eq!(hits.len(), 1 + 8 + 1 + 2 + 2 + 2 + 1 + 1);
    }

    #[test]
    fn notices_sit_inside_the_window_and_clear_of_the_controls() {
        let window = SINGLE.window();
        assert!(SINGLE.footer_baseline().y() < window.height());
        assert!(SINGLE.notice_baseline().y() > SINGLE.layer_toggle().bottom());
        assert!(SINGLE.notice_baseline().y() < SINGLE.footer_baseline().y());
        assert!(SINGLE.ratio_baseline().y() > SINGLE.unit_button(Unit::Logical).bottom());
        assert!(SINGLE.ratio_baseline().y() < SINGLE.theme_button(ModalTheme::Dark).y());
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
        let cases = [
            ((50, 80), Some(SettingsControl::Preset(1))),
            ((20, 110), Some(SettingsControl::HexField)),
            ((100, 150), Some(SettingsControl::Shade)),
            ((230, 150), Some(SettingsControl::Hue)),
            ((20, 270), Some(SettingsControl::Unit(Unit::Physical))),
            ((140, 270), Some(SettingsControl::Unit(Unit::Logical))),
            ((250, 270), None),
            ((100, 300), None),
            ((100, 342), Some(SettingsControl::Theme(ModalTheme::Light))),
            ((20, 392), Some(SettingsControl::Layer(WindowLayer::Normal))),
            ((300, 418), None),
            ((300, 20), Some(SettingsControl::Close)),
            ((270, 396), Some(SettingsControl::Quit)),
            // 題の上は掴み所（部品ではない）。
            ((40, 20), None),
        ];
        for ((x, y), expected) in cases {
            assert_eq!(
                control_at(&hits, PixelPoint::new(x, y)),
                expected,
                "({x}, {y})"
            );
        }
    }

    #[test]
    fn headings_sit_above_their_rows() {
        let [ink, unit, theme, layer] = SINGLE.headings();
        assert!(ink.baseline().y() < SINGLE.preset(0).y());
        assert!(unit.baseline().y() < SINGLE.unit_button(Unit::Physical).y());
        assert!(unit.baseline().y() > SINGLE.shade_area().bottom());
        assert!(theme.baseline().y() < SINGLE.theme_button(ModalTheme::Dark).y());
        assert!(theme.baseline().y() > SINGLE.unit_button(Unit::Physical).bottom());
        assert_eq!(layer.text(), "Always on top");
        assert_eq!(SINGLE.title().text(), "NeNe Ruler");
    }

    #[test]
    fn double_is_exactly_twice_the_single() {
        let double = SettingsLayout::new(UiScale::Double);
        assert_eq!(double.window(), PixelRect::new(0, 0, 640, 896));
        assert_eq!(double.preset(7), PixelRect::new(480, 144, 48, 48));
        assert_eq!(
            double.notice_baseline(),
            PixelPoint::new(
                2 * SINGLE.notice_baseline().x(),
                2 * SINGLE.notice_baseline().y()
            )
        );
        let hits = double.hits(WindowLayer::AlwaysOnTop);
        assert_eq!(
            control_at(&hits, PixelPoint::new(200, 684)),
            Some(SettingsControl::Theme(ModalTheme::Light))
        );
        assert_eq!(control_at(&hits, PixelPoint::new(88, 174)), None);
    }

    #[test]
    fn the_picker_maps_positions_to_colours_and_back() {
        assert_eq!(SINGLE.shade_area(), PixelRect::new(16, 144, 200, 96));
        assert_eq!(SINGLE.hue_bar(), PixelRect::new(224, 144, 20, 96));
        // 四角: 左上は彩度 0・明度 1000、右下は彩度 1000・明度 0。外は縁へ寄せる。
        let shades = [
            ((16, 144), (0, 1000)),
            ((215, 239), (1000, 0)),
            ((0, 0), (0, 1000)),
            ((400, 400), (1000, 0)),
        ];
        for ((x, y), expected) in shades {
            assert_eq!(SINGLE.shade_at(PixelPoint::new(x, y)), expected);
        }
        // 帯: 上端が 0 度、下端が 359.9 度。
        assert_eq!(SINGLE.hue_at(PixelPoint::new(230, 144)), 0);
        assert_eq!(SINGLE.hue_at(PixelPoint::new(230, 239)), 3599);
        // 印は逆写像。
        let colour = Hsv::new(1800, 1000, 0);
        assert_eq!(SINGLE.shade_point(colour), PixelPoint::new(215, 239));
        assert_eq!(SINGLE.hue_y(colour), 144 + 47);
    }
}
