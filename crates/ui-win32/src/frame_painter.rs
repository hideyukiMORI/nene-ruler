//! 枠の 1 フレームを描く（ADR 0021）。X11 版の `canvas.rs` と同じ順に、同じ core の置き場から描く。
//!
//! 背景はアルファ 0、当たり判定の矩形はアルファ 1/255 の下塗り（ADR 0008）。Windows の layered 窓は
//! アルファ 0 の画素をクリックが抜ける所として扱うので、WSLg と同じ形で当たり判定が効く。

use core::f32::consts::PI;

use nene_ruler_application::copy_mark::CopyMark;
use nene_ruler_application::ruler_frame::RulerFrame;
use nene_ruler_core::edge::Edge;
use nene_ruler_core::length_label::LengthLabel;
use nene_ruler_core::pixel_rect::PixelRect;
use windows::Win32::Graphics::Direct2D::ID2D1Brush;
use windows_numerics::Vector2;

use crate::layered_bitmap::LayeredBitmap;
use crate::pen::Pen;
use crate::text_align::TextAlign;
use crate::win32_failure::Win32Failure;

/// 主目盛の数字（ADR 0013）と幅・高さの数字（ADR 0015）の大きさ。倍率に関わらずこの大きさ。
const TICK_FONT_PX: f32 = 9.0;
const LABEL_FONT_PX: f32 = 11.0;
/// 目に見えないが当たり判定には乗るアルファ（ADR 0008）。
const TOUCHABLE_ALPHA: f32 = 1.0 / 255.0;
/// ギアの歯の数と、1 つの円弧を割る線分の数。
const GEAR_TEETH: u16 = 8;
const ARC_STEPS: u16 = 4;

/// フレームを描き切った画素。
///
/// # Errors
///
/// DIB か Direct2D の描画が失敗した。
pub fn paint_frame(pen: &Pen, frame: &RulerFrame) -> Result<LayeredBitmap, Win32Failure> {
    pen.paint(frame.layout().window(), |pen| draw(pen, frame))
}

/// 下塗り・線と目盛と鉤括弧・目盛の数字・幅と高さの数字・通知・ギア。
fn draw(pen: &Pen, frame: &RulerFrame) -> Result<(), Win32Failure> {
    let ink = pen.brush(frame.ink(), 1.0)?;
    let touchable = pen.brush(frame.ink(), TOUCHABLE_ALPHA)?;
    for region in frame.hit_regions() {
        pen.fill(region.rect(), &touchable);
    }
    for rect in frame.layout().ink_rects() {
        pen.fill(rect, &ink);
    }
    for edge in [Edge::Top, Edge::Left] {
        for label in frame.layout().tick_labels(edge) {
            let value = label.value().to_string();
            pen.write(
                (&value, label.baseline()),
                (TICK_FONT_PX, TextAlign::Left),
                &ink,
            )?;
        }
    }
    for label in frame.layout().labels() {
        draw_label(pen, frame, *label, (&ink, &touchable))?;
    }
    let notice_px = f32::from(frame.layout().notice_px());
    for (index, notice) in frame.notices().list().into_iter().enumerate() {
        let index = u16::try_from(index).map_err(|_error| Win32Failure::GeometryOutOfRange)?;
        let baseline = frame.layout().notice_baseline(index);
        pen.write(
            (notice.describe(), baseline),
            (notice_px, TextAlign::Left),
            &ink,
        )?;
    }
    draw_gear(pen, frame.layout().gear(), (&ink, &touchable))
}

/// 数字。コピーが成功していれば、インクの箱に「見えないが触れる」字（反転）、失敗していれば取り消し線。
fn draw_label(
    pen: &Pen,
    frame: &RulerFrame,
    label: LengthLabel,
    (ink, touchable): (&ID2D1Brush, &ID2D1Brush),
) -> Result<(), Win32Failure> {
    let text = frame.shown(label).as_str();
    let baseline = frame.layout().label_baseline(label);
    let hit = frame.layout().label_hit(label, frame.shown(label).chars());
    let mark = frame.copy_mark().filter(|mark| mark.label() == label);
    let inverted = mark.is_some_and(CopyMark::succeeded);
    if inverted {
        pen.fill(hit, ink);
    }
    pen.rotate_about(baseline, label == LengthLabel::Height);
    let style = (LABEL_FONT_PX, TextAlign::Centre);
    let written = if inverted {
        pen.copying(|| pen.write((text, baseline), style, touchable).map(|_end| ()))
    } else {
        pen.write((text, baseline), style, ink).map(|_end| ())
    };
    pen.rotate_about(baseline, false);
    written?;
    if mark.is_some_and(|mark| !mark.succeeded()) {
        let thickness = frame.layout().scale().factor();
        let line = match label {
            LengthLabel::Width | LengthLabel::Size => {
                PixelRect::new(hit.x(), hit.y() + hit.height() / 2, hit.width(), thickness)
            }
            LengthLabel::Height => {
                PixelRect::new(hit.x() + hit.width() / 2, hit.y(), thickness, hit.height())
            }
        };
        pen.fill(line, ink);
    }
    Ok(())
}

/// ギア。丸をインクで塗り、歯車の形を「見えないが触れる」アルファで抜き、軸だけインクで戻す（FR-009 / ADR 0008）。
fn draw_gear(
    pen: &Pen,
    bounds: PixelRect,
    (ink, touchable): (&ID2D1Brush, &ID2D1Brush),
) -> Result<(), Win32Failure> {
    let radius = f32::from(bounds.width()) / 2.0;
    let centre = Vector2 {
        X: f32::from(bounds.x()) + radius,
        Y: f32::from(bounds.y()) + radius,
    };
    pen.fill_disc(centre, radius, ink);
    let teeth = gear_outline(centre, radius);
    pen.copying(|| pen.fill_polygon(&teeth, touchable))?;
    pen.fill_disc(centre, radius * 0.2, ink);
    Ok(())
}

/// 歯車の輪郭。内径と外径を歯ごとに行き来する（X11 版の `gear_path` と同じ形）。
fn gear_outline(centre: Vector2, radius: f32) -> Vec<Vector2> {
    let (inner, outer) = (radius * 0.48, radius * 0.72);
    let pitch = 2.0 * PI / f32::from(GEAR_TEETH);
    let half = pitch / 4.0;
    let at = |angle: f32, length: f32| Vector2 {
        X: centre.X + length * angle.cos(),
        Y: centre.Y + length * angle.sin(),
    };
    let mut points = Vec::new();
    for tooth in 0..GEAR_TEETH {
        let middle = f32::from(tooth) * pitch;
        points.extend(arc_points((middle - 2.0 * half, middle - half), inner, &at));
        points.extend(arc_points((middle - half, middle + half), outer, &at));
    }
    points
}

/// 円弧を `ARC_STEPS` 本の線分に割った点（両端を含む）。
fn arc_points(
    (from, to): (f32, f32),
    length: f32,
    at: &impl Fn(f32, f32) -> Vector2,
) -> impl Iterator<Item = Vector2> {
    (0..=ARC_STEPS).map(move |step| {
        let angle = from + (to - from) * f32::from(step) / f32::from(ARC_STEPS);
        at(angle, length)
    })
}
