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
use nene_ruler_core::unit::Unit;
use nene_ruler_core::window_layer::WindowLayer;

use crate::canvas::{Canvas, FONT_FAMILY, fill_rect, set_ink};
use crate::ui_failure::UiFailure;

/// 題の大きさ。
const TITLE_FONT_PX: f64 = 15.0;
/// 本文の大きさ。
const BODY_FONT_PX: f64 = 12.0;
/// 脚注の大きさ。
const FOOTER_FONT_PX: f64 = 10.0;
/// 選択の印とチェック箱の線の太さ。
const RING_PX: u16 = 2;

/// モーダルを描き切る。
///
/// # Errors
///
/// cairo が失敗した。
pub fn render_settings(frame: &SettingsFrame, format: Format) -> Result<Canvas, UiFailure> {
    let palette = frame.palette();
    Canvas::paint(SettingsLayout::window(), format, |context| {
        set_ink(context, palette.background());
        context.paint()?;
        draw_text(context, &palette, SettingsLayout::title(), TITLE_FONT_PX)?;
        for heading in SettingsLayout::headings() {
            draw_text(context, &palette, heading, BODY_FONT_PX)?;
        }
        draw_presets(context, &palette, frame.settings().ink())?;
        draw_hex_field(context, &palette, frame)?;
        draw_unit_buttons(context, &palette, frame.settings().unit())?;
        draw_theme_buttons(context, &palette, frame.settings().modal_theme())?;
        draw_layer_toggle(context, &palette, frame.settings().layer())?;
        if let Some(failure) = frame.store_failure() {
            set_ink(context, Rgb::INITIAL_INK);
            write_at(
                context,
                SettingsLayout::notice_baseline(),
                failure.describe(),
                FOOTER_FONT_PX,
            )?;
        }
        set_ink(context, palette.text());
        write_at(
            context,
            SettingsLayout::footer_baseline(),
            frame.footer(),
            FOOTER_FONT_PX,
        )
    })
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

/// 矩形の中央に文字を置く。
fn write_centred(context: &Context, rect: PixelRect, text: &str) -> Result<(), UiFailure> {
    context.select_font_face(FONT_FAMILY, FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(BODY_FONT_PX);
    let extents = context.text_extents(text)?;
    let x = f64::from(rect.x()) + (f64::from(rect.width()) - extents.width()) / 2.0_f64
        - extents.x_bearing();
    let y = f64::from(rect.y()) + f64::midpoint(f64::from(rect.height()), extents.height());
    context.move_to(x, y);
    context.show_text(text)?;
    Ok(())
}

/// 矩形の縁を `RING_PX` の太さで描く。
fn stroke_ring(context: &Context, rect: PixelRect, ink: Rgb) -> Result<(), UiFailure> {
    set_ink(context, ink);
    let outer = rect;
    let inner = PixelRect::new(
        rect.x() + RING_PX,
        rect.y() + RING_PX,
        rect.width().saturating_sub(2 * RING_PX),
        rect.height().saturating_sub(2 * RING_PX),
    );
    fill_rect(context, outer)?;
    context.set_operator(cairo::Operator::Clear);
    fill_rect(context, inner)?;
    context.set_operator(cairo::Operator::Over);
    Ok(())
}

/// 色見本。選ばれているものには選択の印の輪。
fn draw_presets(context: &Context, palette: &ModalPalette, ink: Rgb) -> Result<(), UiFailure> {
    for (index, preset) in Rgb::PRESETS.iter().enumerate() {
        let index = u8::try_from(index).map_err(|_error| UiFailure::GeometryOutOfRange)?;
        let rect = SettingsLayout::preset(index);
        set_ink(context, *preset);
        fill_rect(context, rect)?;
        if *preset == ink {
            // 輪は背景の上に見えるよう、見本の外側に 2 px 広げて描き、内側を見本の色で戻す。
            let ring = PixelRect::new(
                rect.x().saturating_sub(RING_PX),
                rect.y().saturating_sub(RING_PX),
                rect.width() + 2 * RING_PX,
                rect.height() + 2 * RING_PX,
            );
            stroke_ring(context, ring, palette.accent())?;
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
    let field = SettingsLayout::hex_field();
    set_ink(context, palette.field());
    fill_rect(context, field)?;
    stroke_ring(context, field, palette.border())?;
    set_ink(context, palette.field());
    fill_rect(
        context,
        PixelRect::new(
            field.x() + RING_PX,
            field.y() + RING_PX,
            field.width() - 2 * RING_PX,
            field.height() - 2 * RING_PX,
        ),
    )?;
    set_ink(context, palette.text());
    let session = frame.session();
    write_at(
        context,
        SettingsLayout::hex_text_baseline(),
        session.entry().text(),
        BODY_FONT_PX,
    )?;
    if session.rejection().is_some() {
        set_ink(context, Rgb::INITIAL_INK);
        write_at(
            context,
            SettingsLayout::hex_notice_baseline(),
            "not #RRGGBB",
            BODY_FONT_PX,
        )?;
    }
    Ok(())
}

/// 選ばれている・いないで地の色を変えたボタン。
fn draw_button(
    context: &Context,
    palette: &ModalPalette,
    rect: PixelRect,
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
    write_centred(context, rect, text)
}

/// 単位のボタン列。
fn draw_unit_buttons(
    context: &Context,
    palette: &ModalPalette,
    unit: Unit,
) -> Result<(), UiFailure> {
    for candidate in [Unit::Px, Unit::Mm, Unit::In] {
        draw_button(
            context,
            palette,
            SettingsLayout::unit_button(candidate),
            (candidate.suffix(), candidate == unit),
        )?;
    }
    Ok(())
}

/// 見えのボタン列。
fn draw_theme_buttons(
    context: &Context,
    palette: &ModalPalette,
    theme: ModalTheme,
) -> Result<(), UiFailure> {
    for candidate in ModalTheme::ALL {
        draw_button(
            context,
            palette,
            SettingsLayout::theme_button(candidate),
            (candidate.label(), candidate == theme),
        )?;
    }
    Ok(())
}

/// 最前面のチェック箱。
fn draw_layer_toggle(
    context: &Context,
    palette: &ModalPalette,
    layer: WindowLayer,
) -> Result<(), UiFailure> {
    let toggle = SettingsLayout::layer_toggle();
    set_ink(context, palette.field());
    fill_rect(context, toggle)?;
    stroke_ring(context, toggle, palette.border())?;
    set_ink(context, palette.field());
    fill_rect(
        context,
        PixelRect::new(
            toggle.x() + RING_PX,
            toggle.y() + RING_PX,
            toggle.width() - 2 * RING_PX,
            toggle.height() - 2 * RING_PX,
        ),
    )?;
    if layer == WindowLayer::AlwaysOnTop {
        set_ink(context, palette.accent());
        fill_rect(
            context,
            PixelRect::new(
                toggle.x() + 6,
                toggle.y() + 6,
                toggle.width() - 12,
                toggle.height() - 12,
            ),
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
        let mut controller = RulerController::new(
            Ports::new(&NoScreen, store, &NoClipboard),
            Transparency::Composited,
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
        let notice = SettingsLayout::notice_baseline();
        let row: Vec<[u8; 4]> = (notice.x()..notice.x() + 80)
            .filter_map(|x| failed.pixel(x, notice.y() - 3).unwrap())
            .collect();
        assert!(row.iter().any(|p| p[2] > p[1] && p[2] > 0x80));
        let mut clean = canvas(&[]);
        let clean_row: Vec<[u8; 4]> = (notice.x()..notice.x() + 80)
            .filter_map(|x| clean.pixel(x, notice.y() - 3).unwrap())
            .collect();
        assert!(clean_row.iter().all(|p| *p == [0x1E, 0x1E, 0x1E, 0xFF]));
    }

    #[test]
    fn dark_background_and_the_selected_preset_have_the_expected_pixels() {
        let mut canvas = canvas(&[]);
        assert_eq!((canvas.width(), canvas.height()), (320, 336));
        assert_eq!(
            canvas.pixel(300, 300).unwrap(),
            Some([0x1E, 0x1E, 0x1E, 0xFF])
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
            Some([0x1E, 0x1E, 0x1E, 0xFF])
        );
    }

    #[test]
    fn light_theme_changes_the_background() {
        let mut canvas = canvas(&[RulerIntent::ChooseModalTheme(ModalTheme::Light)]);
        assert_eq!(
            canvas.pixel(300, 300).unwrap(),
            Some([0xF5, 0xF5, 0xF5, 0xFF])
        );
    }

    #[test]
    fn selected_unit_button_is_filled_with_the_accent() {
        let mut canvas = canvas(&[RulerIntent::ChooseUnit(Unit::Mm)]);
        let mm = SettingsLayout::unit_button(Unit::Mm);
        let px = SettingsLayout::unit_button(Unit::Px);
        assert_eq!(
            canvas.pixel(mm.x() + 2, mm.y() + 2).unwrap(),
            Some([0xFF, 0xFF, 0xFF, 0xFF])
        );
        assert_eq!(
            canvas.pixel(px.x() + 2, px.y() + 2).unwrap(),
            Some([0x2C, 0x2C, 0x2C, 0xFF])
        );
    }

    #[test]
    fn the_toggle_shows_the_layer() {
        let toggle = SettingsLayout::layer_toggle();
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
        let notice = SettingsLayout::hex_notice_baseline();
        let row: Vec<[u8; 4]> = (notice.x()..notice.x() + 60)
            .filter_map(|x| canvas.pixel(x, notice.y() - 4).unwrap())
            .collect();
        assert!(row.iter().any(|p| p[2] > p[1] && p[2] > 0x80));
    }
}
