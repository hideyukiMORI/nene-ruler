//! 設定モーダルを描く（ADR 0014 / 0016 / 0021）。X11 版の `settings_canvas.rs` と同じ順に、同じ core の置き場から描く。

use nene_ruler_application::settings_frame::SettingsFrame;
use nene_ruler_core::hsv::Hsv;
use nene_ruler_core::modal_label::ModalLabel;
use nene_ruler_core::modal_palette::ModalPalette;
use nene_ruler_core::modal_theme::ModalTheme;
use nene_ruler_core::pixel_rect::PixelRect;
use nene_ruler_core::rgb::Rgb;
use nene_ruler_core::settings_layout::SettingsLayout;
use nene_ruler_core::window_layer::WindowLayer;
use windows::Win32::Graphics::Direct2D::ID2D1Brush;
use windows_numerics::Vector2;

use crate::layered_bitmap::LayeredBitmap;
use crate::modal_focus::ModalFocus;
use crate::pen::Pen;
use crate::text_align::TextAlign;
use crate::win32_failure::Win32Failure;

/// 題・本文・脚注の大きさ（1x。モーダルは倍率を掛けない・ADR 0014）。
const TITLE_FONT_PX: f32 = 15.0;
const BODY_FONT_PX: f32 = 12.0;
const FOOTER_FONT_PX: f32 = 10.0;
/// 選択の印とチェック箱の線の太さ、チェックの印の内側への距離、× の内側への距離と線の太さ。
const RING_PX: u16 = 2;
const CHECK_INSET_PX: u16 = 6;
const CROSS_INSET_PX: u16 = 7;
const CROSS_LINE_PX: f32 = 1.5;
/// 入力欄の棒の、欄の上下の縁からの距離。
const CARET_MARGIN_PX: u16 = 6;

/// モーダルを描き切った画素（不透明）。
///
/// # Errors
///
/// DIB か Direct2D の描画が失敗した。
pub fn paint_settings(
    pen: &Pen,
    frame: &SettingsFrame,
    focus: ModalFocus,
) -> Result<LayeredBitmap, Win32Failure> {
    let layout = frame.layout();
    pen.paint(layout.window(), |pen| draw(pen, frame, focus))
}

/// 背景・縁線・×・題と見出し・色見本・入力欄・ピッカー・見え・最前面・Quit・通知・脚注。
fn draw(pen: &Pen, frame: &SettingsFrame, focus: ModalFocus) -> Result<(), Win32Failure> {
    let palette = frame.palette();
    let layout = frame.layout();
    let text = pen.brush(palette.text(), 1.0)?;
    pen.fill(layout.window(), &pen.brush(palette.background(), 1.0)?);
    draw_outline(pen, layout.window(), &pen.brush(palette.border(), 1.0)?);
    draw_close_button(pen, layout, &text);
    write_label(pen, layout.title(), TITLE_FONT_PX, &text)?;
    for heading in layout.headings() {
        write_label(pen, heading, BODY_FONT_PX, &text)?;
    }
    draw_presets(pen, &palette, layout, frame.settings().ink())?;
    draw_hex_field(pen, &palette, frame, focus)?;
    draw_picker(pen, layout, frame.session().picker())?;
    for theme in ModalTheme::ALL {
        let selected = theme == frame.settings().modal_theme();
        draw_button(
            pen,
            &palette,
            (layout.theme_button(theme), theme.label()),
            selected,
        )?;
    }
    draw_layer_toggle(pen, &palette, layout, frame.settings().layer())?;
    draw_button(pen, &palette, (layout.quit_button(), "Quit"), false)?;
    if let Some(failure) = frame.store_failure() {
        let warning = pen.brush(Rgb::INITIAL_INK, 1.0)?;
        pen.write(
            (failure.describe(), layout.notice_baseline()),
            (FOOTER_FONT_PX, TextAlign::Left),
            &warning,
        )?;
    }
    pen.write(
        (frame.footer(), layout.footer_baseline()),
        (FOOTER_FONT_PX, TextAlign::Left),
        &text,
    )?;
    Ok(())
}

/// 見出しや題を 1 行。
fn write_label(
    pen: &Pen,
    label: ModalLabel,
    size: f32,
    brush: &ID2D1Brush,
) -> Result<(), Win32Failure> {
    pen.write(
        (label.text(), label.baseline()),
        (size, TextAlign::Left),
        brush,
    )
    .map(|_end| ())
}

/// 窓の外周の 1 px の縁線（ADR 0014）。
fn draw_outline(pen: &Pen, window: PixelRect, brush: &ID2D1Brush) {
    let (width, height) = (window.width(), window.height());
    for edge in [
        PixelRect::new(0, 0, width, 1),
        PixelRect::new(0, height.saturating_sub(1), width, 1),
        PixelRect::new(0, 0, 1, height),
        PixelRect::new(width.saturating_sub(1), 0, 1, height),
    ] {
        pen.fill(edge, brush);
    }
}

/// 右上の ×（ADR 0014）。
fn draw_close_button(pen: &Pen, layout: SettingsLayout, brush: &ID2D1Brush) {
    let button = inset(layout.close_button(), CROSS_INSET_PX);
    let (left, top) = (f32::from(button.x()), f32::from(button.y()));
    let (right, bottom) = (
        left + f32::from(button.width()),
        top + f32::from(button.height()),
    );
    let point = |x: f32, y: f32| Vector2 { X: x, Y: y };
    pen.stroke_line(
        (point(left, top), point(right, bottom)),
        (CROSS_LINE_PX, brush),
    );
    pen.stroke_line(
        (point(right, top), point(left, bottom)),
        (CROSS_LINE_PX, brush),
    );
}

/// 矩形の縁を `RING_PX` の太さで描く（内側は塗らない）。
fn stroke_ring(pen: &Pen, rect: PixelRect, brush: &ID2D1Brush) {
    let (x, y, width, height) = (rect.x(), rect.y(), rect.width(), rect.height());
    for edge in [
        PixelRect::new(x, y, width, RING_PX),
        PixelRect::new(x, (y + height).saturating_sub(RING_PX), width, RING_PX),
        PixelRect::new(x, y, RING_PX, height),
        PixelRect::new((x + width).saturating_sub(RING_PX), y, RING_PX, height),
    ] {
        pen.fill(edge, brush);
    }
}

/// 色見本。選ばれているものには外側に選択の輪。
fn draw_presets(
    pen: &Pen,
    palette: &ModalPalette,
    layout: SettingsLayout,
    ink: Rgb,
) -> Result<(), Win32Failure> {
    for (index, preset) in Rgb::PRESETS.iter().enumerate() {
        let index = u8::try_from(index).map_err(|_error| Win32Failure::GeometryOutOfRange)?;
        let rect = layout.preset(index);
        pen.fill(rect, &pen.brush(*preset, 1.0)?);
        if *preset == ink {
            let ring = PixelRect::new(
                rect.x().saturating_sub(RING_PX),
                rect.y().saturating_sub(RING_PX),
                rect.width() + 2 * RING_PX,
                rect.height() + 2 * RING_PX,
            );
            stroke_ring(pen, ring, &pen.brush(palette.accent(), 1.0)?);
        }
    }
    Ok(())
}

/// `#RRGGBB` の入力欄。焦点があれば選択色の縁と文字の後ろの棒。拒否されていれば右に短く示す（FR-014）。
fn draw_hex_field(
    pen: &Pen,
    palette: &ModalPalette,
    frame: &SettingsFrame,
    focus: ModalFocus,
) -> Result<(), Win32Failure> {
    let layout = frame.layout();
    let field = layout.hex_field();
    let edge = match focus {
        ModalFocus::Focused => palette.accent(),
        ModalFocus::Unfocused => palette.border(),
    };
    pen.fill(field, &pen.brush(palette.field(), 1.0)?);
    stroke_ring(pen, field, &pen.brush(edge, 1.0)?);
    let text = pen.brush(palette.text(), 1.0)?;
    let session = frame.session();
    let end = pen.write(
        (session.entry().text(), layout.hex_text_baseline()),
        (BODY_FONT_PX, TextAlign::Left),
        &text,
    )?;
    if focus == ModalFocus::Focused {
        let top = f32::from(field.y() + CARET_MARGIN_PX);
        let bottom = f32::from((field.y() + field.height()).saturating_sub(CARET_MARGIN_PX));
        let x = end.ceil() + 1.5;
        pen.stroke_line(
            (Vector2 { X: x, Y: top }, Vector2 { X: x, Y: bottom }),
            (1.0, &text),
        );
    }
    if session.rejection().is_some() {
        let warning = pen.brush(Rgb::INITIAL_INK, 1.0)?;
        pen.write(
            ("not #RRGGBB", layout.hex_notice_baseline()),
            (BODY_FONT_PX, TextAlign::Left),
            &warning,
        )?;
    }
    Ok(())
}

/// ピッカー。四角は色相の純色に左から白・下から黒を重ね、帯は 6 区間の色相（ADR 0016）。
fn draw_picker(pen: &Pen, layout: SettingsLayout, picker: Hsv) -> Result<(), Win32Failure> {
    let area = layout.shade_area();
    pen.fill(
        area,
        &pen.brush(Hsv::new(picker.hue(), 1000, 1000).to_rgb(), 1.0)?,
    );
    let white = Rgb::new(0xFF, 0xFF, 0xFF);
    let black = Rgb::new(0, 0, 0);
    pen.fill_gradient(area, &[(0.0, white, 1.0), (1.0, white, 0.0)], false)?;
    pen.fill_gradient(area, &[(0.0, black, 0.0), (1.0, black, 1.0)], true)?;
    let hues: Vec<(f32, Rgb, f32)> = (0..=6_u16)
        .map(|sixth| {
            (
                f32::from(sixth) / 6.0,
                Hsv::new(sixth * 600, 1000, 1000).to_rgb(),
                1.0,
            )
        })
        .collect();
    pen.fill_gradient(layout.hue_bar(), &hues, true)?;
    let spot = layout.shade_point(picker);
    let centre = Vector2 {
        X: f32::from(spot.x()) + 0.5,
        Y: f32::from(spot.y()) + 0.5,
    };
    pen.stroke_circle(centre, 5.0, (3.0, &pen.brush(black, 1.0)?));
    pen.stroke_circle(centre, 5.0, (1.5, &pen.brush(white, 1.0)?));
    let strip = layout.hue_bar();
    let line_y = layout.hue_y(picker);
    pen.fill(
        PixelRect::new(
            strip.x().saturating_sub(2),
            line_y.saturating_sub(2),
            strip.width() + 4,
            5,
        ),
        &pen.brush(black, 1.0)?,
    );
    pen.fill(
        PixelRect::new(
            strip.x().saturating_sub(1),
            line_y.saturating_sub(1),
            strip.width() + 2,
            3,
        ),
        &pen.brush(white, 1.0)?,
    );
    Ok(())
}

/// 選ばれている・いないで地の色を変えたボタン。
fn draw_button(
    pen: &Pen,
    palette: &ModalPalette,
    (rect, label): (PixelRect, &str),
    selected: bool,
) -> Result<(), Win32Failure> {
    let (ground, ink) = if selected {
        (palette.accent(), palette.background())
    } else {
        (palette.field(), palette.text())
    };
    pen.fill(rect, &pen.brush(ground, 1.0)?);
    pen.write_in((label, rect), BODY_FONT_PX, &pen.brush(ink, 1.0)?)
}

/// 最前面のチェック箱。
fn draw_layer_toggle(
    pen: &Pen,
    palette: &ModalPalette,
    layout: SettingsLayout,
    layer: WindowLayer,
) -> Result<(), Win32Failure> {
    let toggle = layout.layer_toggle();
    pen.fill(toggle, &pen.brush(palette.field(), 1.0)?);
    stroke_ring(pen, toggle, &pen.brush(palette.border(), 1.0)?);
    if layer == WindowLayer::AlwaysOnTop {
        pen.fill(
            inset(toggle, CHECK_INSET_PX),
            &pen.brush(palette.accent(), 1.0)?,
        );
    }
    Ok(())
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
