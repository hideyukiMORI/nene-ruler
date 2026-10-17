//! `SettingsFrame` を cairo の `ImageSurface` に描く。不透明（FR-010 の背景）。

use cairo::{Context, FontSlant, FontWeight, Format};
use nene_ruler_application::settings_frame::SettingsFrame;
use nene_ruler_core::modal_label::ModalLabel;
use nene_ruler_core::modal_palette::ModalPalette;
use nene_ruler_core::modal_theme::ModalTheme;
use nene_ruler_core::pixel_point::PixelPoint;
use nene_ruler_core::pixel_rect::PixelRect;
use nene_ruler_core::rgb::Rgb;
use nene_ruler_core::settings_layout::SettingsLayout;
use nene_ruler_core::window_layer::WindowLayer;

use crate::canvas::{Canvas, FONT_FAMILY, fill_rect, font_px, set_ink};
use crate::ui_failure::UiFailure;

/// 題の大きさ（1x）。
const TITLE_FONT_PX: f64 = 15.0;
/// 本文の大きさ（1x）。
const BODY_FONT_PX: f64 = 12.0;
/// 脚注の大きさ（1x）。
const FOOTER_FONT_PX: f64 = 10.0;
/// 選択の印とチェック箱の線の太さ（1x）。
const RING_PX: u16 = 2;
/// チェック箱の印の、箱の縁からの距離（1x）。
const CHECK_INSET_PX: u16 = 6;
/// 閉じるボタンの × の、ボタンの縁からの距離と線の太さ（1x）。
const CROSS_INSET_PX: u16 = 7;
const CROSS_LINE_PX: f64 = 1.5;

/// モーダルを描き切る。
///
/// # Errors
///
/// cairo が失敗した。
pub fn render_settings(frame: &SettingsFrame, format: Format) -> Result<Canvas, UiFailure> {
    let palette = frame.palette();
    let layout = frame.layout();
    let (title_px, body_px, footer_px) = (
        font_px(TITLE_FONT_PX, layout.scale()),
        font_px(BODY_FONT_PX, layout.scale()),
        font_px(FOOTER_FONT_PX, layout.scale()),
    );
    Canvas::paint(layout.window(), format, |context| {
        set_ink(context, palette.background());
        context.paint()?;
        draw_outline(context, palette.border(), layout.window())?;
        draw_close_button(context, &palette, layout)?;
        draw_text(context, &palette, layout.title(), title_px)?;
        for heading in layout.headings() {
            draw_text(context, &palette, heading, body_px)?;
        }
        draw_presets(context, &palette, layout, frame.settings().ink())?;
        draw_hex_field(context, &palette, frame)?;
        draw_theme_buttons(context, &palette, layout, frame.settings().modal_theme())?;
        draw_layer_toggle(context, &palette, layout, frame.settings().layer())?;
        if let Some(failure) = frame.store_failure() {
            set_ink(context, Rgb::INITIAL_INK);
            write_at(
                context,
                layout.notice_baseline(),
                failure.describe(),
                footer_px,
            )?;
        }
        set_ink(context, palette.text());
        write_at(context, layout.footer_baseline(), frame.footer(), footer_px)
    })
}

/// 窓の外周の 1 px の縁線。窓に飾りが無いので、黒い背景がデスクトップに溶けないようにする（ADR 0014）。
fn draw_outline(context: &Context, ink: Rgb, window: PixelRect) -> Result<(), UiFailure> {
    set_ink(context, ink);
    let (width, height) = (window.width(), window.height());
    for edge in [
        PixelRect::new(0, 0, width, 1),
        PixelRect::new(0, height.saturating_sub(1), width, 1),
        PixelRect::new(0, 0, 1, height),
        PixelRect::new(width.saturating_sub(1), 0, 1, height),
    ] {
        fill_rect(context, edge)?;
    }
    Ok(())
}

/// 右上の閉じるボタン。文字色の ×（ADR 0014）。
fn draw_close_button(
    context: &Context,
    palette: &ModalPalette,
    layout: SettingsLayout,
) -> Result<(), UiFailure> {
    let button = inset(
        layout.close_button(),
        CROSS_INSET_PX * layout.scale().factor(),
    );
    let (left, top) = (f64::from(button.x()), f64::from(button.y()));
    let (right, bottom) = (
        left + f64::from(button.width()),
        top + f64::from(button.height()),
    );
    set_ink(context, palette.text());
    context.set_line_width(CROSS_LINE_PX * f64::from(layout.scale().factor()));
    context.move_to(left, top);
    context.line_to(right, bottom);
    context.move_to(right, top);
    context.line_to(left, bottom);
    context.stroke()?;
    Ok(())
}

/// 文字色で 1 行書く。
fn draw_text(
    context: &Context,
    palette: &ModalPalette,
    label: ModalLabel,
    size: f64,
) -> Result<(), UiFailure> {
    set_ink(context, palette.text());
    write_at(context, label.baseline(), label.text(), size)
}

/// いまの色で、点をベースラインの左端として書く。
fn write_at(
    context: &Context,
    baseline: PixelPoint,
    text: &str,
    size: f64,
) -> Result<(), UiFailure> {
    context.select_font_face(FONT_FAMILY, FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(size);
    context.move_to(f64::from(baseline.x()), f64::from(baseline.y()));
    context.show_text(text)?;
    Ok(())
}

/// 矩形の中央に本文の大きさで文字を置く。
fn write_centred(
    context: &Context,
    rect: PixelRect,
    text: &str,
    layout: SettingsLayout,
) -> Result<(), UiFailure> {
    context.select_font_face(FONT_FAMILY, FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(font_px(BODY_FONT_PX, layout.scale()));
    let extents = context.text_extents(text)?;
    let x = f64::from(rect.x()) + (f64::from(rect.width()) - extents.width()) / 2.0_f64
        - extents.x_bearing();
    let y = f64::from(rect.y()) + f64::midpoint(f64::from(rect.height()), extents.height());
    context.move_to(x, y);
    context.show_text(text)?;
    Ok(())
}

/// 輪の太さ。
const fn ring_px(layout: SettingsLayout) -> u16 {
    RING_PX * layout.scale().factor()
}

/// 矩形を四辺から `by` だけ縮める。
fn inset(rect: PixelRect, by: u16) -> PixelRect {
    PixelRect::new(
        rect.x() + by,
        rect.y() + by,
        rect.width().saturating_sub(2 * by),
        rect.height().saturating_sub(2 * by),
    )
}

/// 矩形の縁を輪の太さで描く。
fn stroke_ring(
    context: &Context,
    rect: PixelRect,
    ink: Rgb,
    layout: SettingsLayout,
) -> Result<(), UiFailure> {
    set_ink(context, ink);
    let outer = rect;
    let inner = inset(rect, ring_px(layout));
    fill_rect(context, outer)?;
    context.set_operator(cairo::Operator::Clear);
    fill_rect(context, inner)?;
    context.set_operator(cairo::Operator::Over);
    Ok(())
}

/// 色見本。選ばれているものには選択の印の輪。
fn draw_presets(
    context: &Context,
    palette: &ModalPalette,
    layout: SettingsLayout,
    ink: Rgb,
) -> Result<(), UiFailure> {
    let ring_px = ring_px(layout);
    for (index, preset) in Rgb::PRESETS.iter().enumerate() {
        let index = u8::try_from(index).map_err(|_error| UiFailure::GeometryOutOfRange)?;
        let rect = layout.preset(index);
        set_ink(context, *preset);
        fill_rect(context, rect)?;
        if *preset == ink {
            // 輪は背景の上に見えるよう、見本の外側に輪の太さだけ広げて描き、内側を見本の色で戻す。
            let ring = PixelRect::new(
                rect.x().saturating_sub(ring_px),
                rect.y().saturating_sub(ring_px),
                rect.width() + 2 * ring_px,
                rect.height() + 2 * ring_px,
            );
            stroke_ring(context, ring, palette.accent(), layout)?;
            set_ink(context, *preset);
            fill_rect(context, rect)?;
        }
    }
    Ok(())
}

/// `#RRGGBB` の入力欄。拒否されていれば右に短く示す（FR-014）。
fn draw_hex_field(
    context: &Context,
    palette: &ModalPalette,
    frame: &SettingsFrame,
) -> Result<(), UiFailure> {
    let layout = frame.layout();
    let body_px = font_px(BODY_FONT_PX, layout.scale());
    let field = layout.hex_field();
    set_ink(context, palette.field());
    fill_rect(context, field)?;
    stroke_ring(context, field, palette.border(), layout)?;
    set_ink(context, palette.field());
    fill_rect(context, inset(field, ring_px(layout)))?;
    set_ink(context, palette.text());
    let session = frame.session();
    write_at(
        context,
        layout.hex_text_baseline(),
        session.entry().text(),
        body_px,
    )?;
    if session.rejection().is_some() {
        set_ink(context, Rgb::INITIAL_INK);
        write_at(
            context,
            layout.hex_notice_baseline(),
            "not #RRGGBB",
            body_px,
        )?;
    }
    Ok(())
}

/// 選ばれている・いないで地の色を変えたボタン。
fn draw_button(
    context: &Context,
    palette: &ModalPalette,
    (rect, layout): (PixelRect, SettingsLayout),
    (text, selected): (&str, bool),
) -> Result<(), UiFailure> {
    set_ink(
        context,
        if selected {
            palette.accent()
        } else {
            palette.field()
        },
    );
    fill_rect(context, rect)?;
    set_ink(
        context,
        if selected {
            palette.background()
        } else {
            palette.text()
        },
    );
    write_centred(context, rect, text, layout)
}

/// 見えのボタン列。
fn draw_theme_buttons(
    context: &Context,
    palette: &ModalPalette,
    layout: SettingsLayout,
    theme: ModalTheme,
) -> Result<(), UiFailure> {
    for candidate in ModalTheme::ALL {
        draw_button(
            context,
            palette,
            (layout.theme_button(candidate), layout),
            (candidate.label(), candidate == theme),
        )?;
    }
    Ok(())
}

/// 最前面のチェック箱。
fn draw_layer_toggle(
    context: &Context,
    palette: &ModalPalette,
    layout: SettingsLayout,
    layer: WindowLayer,
) -> Result<(), UiFailure> {
    let toggle = layout.layer_toggle();
    set_ink(context, palette.field());
    fill_rect(context, toggle)?;
    stroke_ring(context, toggle, palette.border(), layout)?;
    set_ink(context, palette.field());
    fill_rect(context, inset(toggle, ring_px(layout)))?;
    if layer == WindowLayer::AlwaysOnTop {
        set_ink(context, palette.accent());
        fill_rect(
            context,
            inset(toggle, CHECK_INSET_PX * layout.scale().factor()),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ruler_application::ports::Ports;
    use nene_ruler_application::product_version::ProductVersion;
    use nene_ruler_application::ruler_controller::RulerController;
    use nene_ruler_application::ruler_intent::RulerIntent;
    use nene_ruler_application::screen_metrics_port::ScreenMetricsPort;
    use nene_ruler_application::settings_store_failure::SettingsStoreFailure;
    use nene_ruler_application::settings_store_port::SettingsStorePort;
    use nene_ruler_application::transparency::Transparency;
    use nene_ruler_core::ruler_settings::RulerSettings;
    use nene_ruler_core::screen_metrics::ScreenMetrics;
    use nene_ruler_core::screen_metrics_failure::ScreenMetricsFailure;
    use nene_ruler_core::ui_scale::UiScale;

    const SINGLE: SettingsLayout = SettingsLayout::new(UiScale::Single);

    struct NoScreen;

    impl ScreenMetricsPort for NoScreen {
        fn screen_metrics(&self) -> Result<ScreenMetrics, ScreenMetricsFailure> {
            Err(ScreenMetricsFailure::Unavailable)
        }
    }

    struct NoStore(Result<Option<RulerSettings>, SettingsStoreFailure>);

    impl SettingsStorePort for NoStore {
        fn load(&self) -> Result<Option<RulerSettings>, SettingsStoreFailure> {
            self.0
        }

        fn save(&self, _settings: RulerSettings) -> Result<(), SettingsStoreFailure> {
            Ok(())
        }
    }

    struct NoClipboard;

    impl nene_ruler_application::clipboard_port::ClipboardPort for NoClipboard {
        fn copy(
            &self,
            _text: &str,
        ) -> Result<(), nene_ruler_application::clipboard_failure::ClipboardFailure> {
            Ok(())
        }
    }

    fn canvas_with(store: &NoStore, intents: &[RulerIntent]) -> Canvas {
        canvas_at(store, None, intents)
    }

    fn canvas_at(store: &NoStore, xft_dpi: Option<u32>, intents: &[RulerIntent]) -> Canvas {
        let mut controller = RulerController::new(
            Ports::new(&NoScreen, store, &NoClipboard),
            nene_ruler_application::environment::Environment::new(Transparency::Composited)
                .with_xft_dpi(xft_dpi),
            ProductVersion::new("0.1.0"),
        );
        controller.apply(RulerIntent::OpenSettings);
        for intent in intents {
            controller.apply(*intent);
        }
        render_settings(&controller.settings_frame().unwrap(), Format::ARgb32).unwrap()
    }

    fn canvas(intents: &[RulerIntent]) -> Canvas {
        canvas_with(&NoStore(Ok(None)), intents)
    }

    #[test]
    fn a_store_failure_is_written_above_the_footer() {
        let store = NoStore(Err(SettingsStoreFailure::Malformed));
        let mut failed = canvas_with(&store, &[]);
        let notice = SINGLE.notice_baseline();
        let row: Vec<[u8; 4]> = (notice.x()..notice.x() + 80)
            .filter_map(|x| failed.pixel(x, notice.y() - 3).unwrap())
            .collect();
        assert!(row.iter().any(|p| p[2] > p[1] && p[2] > 0x80));
        let mut clean = canvas(&[]);
        let clean_row: Vec<[u8; 4]> = (notice.x()..notice.x() + 80)
            .filter_map(|x| clean.pixel(x, notice.y() - 3).unwrap())
            .collect();
        assert!(clean_row.iter().all(|p| *p == [0x00, 0x00, 0x00, 0xFF]));
    }

    #[test]
    fn dark_background_and_the_selected_preset_have_the_expected_pixels() {
        let mut canvas = canvas(&[]);
        assert_eq!((canvas.width(), canvas.height()), (320, 280));
        assert_eq!(
            canvas.pixel(305, 150).unwrap(),
            Some([0x00, 0x00, 0x00, 0xFF])
        );
        assert_eq!(
            canvas.pixel(28, 84).unwrap(),
            Some([0x35, 0x39, 0xE5, 0xFF])
        );
        assert_eq!(
            canvas.pixel(15, 71).unwrap(),
            Some([0xFF, 0xFF, 0xFF, 0xFF])
        );
        assert_eq!(
            canvas.pixel(47, 71).unwrap(),
            Some([0x00, 0x00, 0x00, 0xFF])
        );
    }

    #[test]
    fn light_theme_changes_the_background() {
        let mut canvas = canvas(&[RulerIntent::ChooseModalTheme(ModalTheme::Light)]);
        assert_eq!(
            canvas.pixel(305, 150).unwrap(),
            Some([0xF5, 0xF5, 0xF5, 0xFF])
        );
    }

    #[test]
    fn the_toggle_shows_the_layer() {
        let toggle = SINGLE.layer_toggle();
        let centre = (toggle.x() + 12, toggle.y() + 12);
        let mut on = canvas(&[]);
        assert_eq!(
            on.pixel(centre.0, centre.1).unwrap(),
            Some([0xFF, 0xFF, 0xFF, 0xFF])
        );
        let mut off = canvas(&[RulerIntent::ChooseLayer(WindowLayer::Normal)]);
        assert_eq!(
            off.pixel(centre.0, centre.1).unwrap(),
            Some([0x2C, 0x2C, 0x2C, 0xFF])
        );
    }

    #[test]
    fn a_rejected_entry_leaves_ink_beside_the_field() {
        let mut canvas = canvas(&[RulerIntent::HexBackspace, RulerIntent::HexSubmit]);
        let notice = SINGLE.hex_notice_baseline();
        let row: Vec<[u8; 4]> = (notice.x()..notice.x() + 60)
            .filter_map(|x| canvas.pixel(x, notice.y() - 4).unwrap())
            .collect();
        assert!(row.iter().any(|p| p[2] > p[1] && p[2] > 0x80));
    }

    #[test]
    fn the_modal_stays_single_even_at_double() {
        // モーダルは倍率を掛けない（ADR 0014）。
        let double = canvas_at(&NoStore(Ok(None)), Some(192), &[]);
        assert_eq!((double.width(), double.height()), (320, 280));
        let single = canvas(&[]);
        assert_eq!((single.width(), single.height()), (320, 280));
    }

    #[test]
    fn nothing_is_noted_beside_always_on_top_or_for_a_missing_screen_size() {
        // 最前面が効かないことも、物理寸法が無いことも示さない（ADR 0012）。NoScreen で描く。
        let mut canvas = canvas(&[]);
        let toggle = SINGLE.layer_toggle();
        let red = (toggle.y()..toggle.bottom())
            .flat_map(|y| (toggle.right() + 100..SINGLE.window().width()).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                canvas
                    .pixel(x, y)
                    .unwrap()
                    .is_some_and(|p| p[2] > 0x80 && p[2] > p[1])
            })
            .count();
        assert_eq!(red, 0);
    }

    #[test]
    fn the_outline_and_the_close_cross_are_drawn() {
        let mut canvas = canvas(&[]);
        let border = [0x55, 0x55, 0x55, 0xFF];
        assert_eq!(canvas.pixel(0, 140).unwrap(), Some(border));
        assert_eq!(canvas.pixel(319, 140).unwrap(), Some(border));
        assert_eq!(canvas.pixel(160, 279).unwrap(), Some(border));
        // × の交点（ボタンの中央）は文字色に近い明るさ。
        let close = SINGLE.close_button();
        let centre = canvas
            .pixel(
                close.x() + close.width() / 2,
                close.y() + close.height() / 2,
            )
            .unwrap()
            .unwrap();
        assert!(centre[0] > 0x80, "{centre:?}");
    }
}
