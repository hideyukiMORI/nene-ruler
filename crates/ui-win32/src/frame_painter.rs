//! 枠の 1 フレームを Direct2D / DirectWrite で描く（ADR 0021）。X11 版の `canvas.rs` と同じ順に、同じ core の置き場から描く。
//!
//! 背景はアルファ 0、当たり判定の矩形はアルファ 1/255 の下塗り（ADR 0008）。Windows の layered 窓は
//! アルファ 0 の画素をクリックが抜ける所として扱うので、WSLg と同じ形で当たり判定が効く。

use core::f32::consts::PI;

use nene_ruler_application::copy_mark::CopyMark;
use nene_ruler_application::ruler_frame::RulerFrame;
use nene_ruler_core::edge::Edge;
use nene_ruler_core::length_label::LengthLabel;
use nene_ruler_core::pixel_point::PixelPoint;
use nene_ruler_core::pixel_rect::PixelRect;
use nene_ruler_core::rgb::Rgb;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D_RECT_F, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_FIGURE_BEGIN_FILLED,
    D2D1_FIGURE_END_CLOSED, D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE, D2D1_FACTORY_TYPE_SINGLE_THREADED,
    D2D1_FEATURE_LEVEL_DEFAULT, D2D1_PRIMITIVE_BLEND_COPY, D2D1_PRIMITIVE_BLEND_SOURCE_OVER,
    D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE,
    D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE, D2D1CreateFactory, ID2D1DCRenderTarget, ID2D1DeviceContext,
    ID2D1Factory, ID2D1SolidColorBrush,
};
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
    DWRITE_FONT_WEIGHT_NORMAL, DWRITE_LINE_METRICS, DWRITE_TEXT_METRICS, DWriteCreateFactory,
    IDWriteFactory, IDWriteTextLayout,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::core::{Interface, w};
use windows_numerics::{Matrix3x2, Vector2};

use crate::layered_bitmap::LayeredBitmap;
use crate::win32_failure::Win32Failure;

/// 字の家族。Windows の UI の字。
const FONT_FAMILY: windows::core::PCWSTR = w!("Segoe UI");
/// 主目盛の数字（ADR 0013）と幅・高さの数字（ADR 0015）の大きさ。倍率に関わらずこの大きさ。
const TICK_FONT_PX: f32 = 9.0;
const LABEL_FONT_PX: f32 = 11.0;
/// 目に見えないが当たり判定には乗るアルファ（ADR 0008）。
const TOUCHABLE_ALPHA: f32 = 1.0 / 255.0;
/// ギアの歯の数と、1 つの円弧を割る線分の数。
const GEAR_TEETH: u16 = 8;
const ARC_STEPS: u16 = 4;

/// Direct2D と DirectWrite の工場と、DC に結ぶレンダーターゲット。窓 1 つにつき 1 つ持つ。
#[derive(Debug)]
pub struct FramePainter {
    target: ID2D1DCRenderTarget,
    factory: ID2D1Factory,
    words: IDWriteFactory,
}

impl FramePainter {
    /// 工場とレンダーターゲットを作る。
    ///
    /// # Errors
    ///
    /// Direct2D か DirectWrite を作れなかった。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D / DirectWrite の工場を作る（ADR 0021）"
    )]
    pub fn new() -> Result<Self, Win32Failure> {
        // SAFETY: 単一スレッドの工場。既定の生成の引数だけを渡す。
        let factory: ID2D1Factory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None) }?;
        let properties = D2D1_RENDER_TARGET_PROPERTIES {
            r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            // 1 DIP = 1 物理画素。測る値は物理画素のまま（FR-016）。
            dpiX: 96.0,
            dpiY: 96.0,
            usage: D2D1_RENDER_TARGET_USAGE_NONE,
            minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
        };
        // SAFETY: properties はこの関数の局所で、呼び出しの間だけ読まれる。
        let target = unsafe { factory.CreateDCRenderTarget(&raw const properties) }?;
        // SAFETY: 共有の DirectWrite の工場。
        let words: IDWriteFactory = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) }?;
        Ok(Self {
            target,
            factory,
            words,
        })
    }

    /// フレームを描き切った画素を返す。
    ///
    /// # Errors
    ///
    /// DIB か Direct2D の描画が失敗した。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の BindDC / BeginDraw / EndDraw を呼ぶ（ADR 0021）"
    )]
    pub fn paint(&self, frame: &RulerFrame) -> Result<LayeredBitmap, Win32Failure> {
        let window = frame.layout().window();
        let bitmap = LayeredBitmap::new(window.width(), window.height())?;
        let area = RECT {
            left: 0,
            top: 0,
            right: i32::from(window.width()),
            bottom: i32::from(window.height()),
        };
        // SAFETY: bitmap の DC はこの関数の間生きている。描画は BeginDraw と EndDraw の間だけ。
        unsafe {
            self.target.BindDC(bitmap.dc(), &raw const area)?;
            self.target.BeginDraw();
            let clear = TRANSPARENT;
            self.target.Clear(Some(&raw const clear));
            self.target
                .SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
        }
        let drawn = self.draw(frame);
        // SAFETY: 上の BeginDraw と対になる。
        unsafe { self.target.EndDraw(None, None) }?;
        drawn.map(|()| bitmap)
    }

    /// 下塗り・線と目盛と鉤括弧・目盛の数字・幅と高さの数字・通知・ギア。
    fn draw(&self, frame: &RulerFrame) -> Result<(), Win32Failure> {
        let ink = self.brush(frame.ink(), 1.0)?;
        let touchable = self.brush(frame.ink(), TOUCHABLE_ALPHA)?;
        for region in frame.hit_regions() {
            self.fill(region.rect(), &touchable);
        }
        for rect in frame.layout().ink_rects() {
            self.fill(rect, &ink);
        }
        for edge in [Edge::Top, Edge::Left] {
            for label in frame.layout().tick_labels(edge) {
                let value = label.value().to_string();
                self.write((&value, label.baseline()), (TICK_FONT_PX, false), &ink)?;
            }
        }
        for label in frame.layout().labels() {
            self.draw_label(frame, *label, (&ink, &touchable))?;
        }
        let notice_px = f32::from(frame.layout().notice_px());
        for (index, notice) in frame.notices().list().into_iter().enumerate() {
            let index = u16::try_from(index).map_err(|_error| Win32Failure::GeometryOutOfRange)?;
            let baseline = frame.layout().notice_baseline(index);
            self.write((notice.describe(), baseline), (notice_px, false), &ink)?;
        }
        self.draw_gear(frame.layout().gear(), (&ink, &touchable))
    }

    /// 数字。コピーが成功していれば、インクの箱に「見えないが触れる」字（反転）、失敗していれば取り消し線。
    fn draw_label(
        &self,
        frame: &RulerFrame,
        label: LengthLabel,
        (ink, touchable): (&ID2D1SolidColorBrush, &ID2D1SolidColorBrush),
    ) -> Result<(), Win32Failure> {
        let text = frame.shown(label).as_str();
        let baseline = frame.layout().label_baseline(label);
        let hit = frame.layout().label_hit(label, frame.shown(label).chars());
        let mark = frame.copy_mark().filter(|mark| mark.label() == label);
        let inverted = mark.is_some_and(CopyMark::succeeded);
        if inverted {
            self.fill(hit, ink);
        }
        let rotated = label == LengthLabel::Height;
        self.rotate_about(baseline, rotated);
        let written = if inverted {
            self.copying(|| self.write((text, baseline), (LABEL_FONT_PX, true), touchable))
        } else {
            self.write((text, baseline), (LABEL_FONT_PX, true), ink)
        };
        self.rotate_about(baseline, false);
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
            self.fill(line, ink);
        }
        Ok(())
    }

    /// ギア。丸をインクで塗り、歯車の形を「見えないが触れる」アルファで抜き、軸だけインクで戻す（FR-009 / ADR 0008）。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の FillEllipse / FillGeometry を呼ぶ（ADR 0021）"
    )]
    fn draw_gear(
        &self,
        bounds: PixelRect,
        (ink, touchable): (&ID2D1SolidColorBrush, &ID2D1SolidColorBrush),
    ) -> Result<(), Win32Failure> {
        let radius = f32::from(bounds.width()) / 2.0;
        let centre = Vector2 {
            X: f32::from(bounds.x()) + radius,
            Y: f32::from(bounds.y()) + radius,
        };
        let disc = |size: f32| D2D1_ELLIPSE {
            point: centre,
            radiusX: size,
            radiusY: size,
        };
        let teeth = self.gear_outline(centre, radius)?;
        let (outside, axle) = (disc(radius), disc(radius * 0.2));
        // SAFETY: 描画は paint の BeginDraw と EndDraw の間で呼ばれる。引数はこの関数の局所。
        unsafe {
            self.target.FillEllipse(&raw const outside, ink);
        }
        self.copying(|| {
            // SAFETY: 同上。
            unsafe {
                self.target.FillGeometry(&teeth, touchable, None);
            }
            Ok(())
        })?;
        // SAFETY: 同上。
        unsafe {
            self.target.FillEllipse(&raw const axle, ink);
        }
        Ok(())
    }

    /// 歯車の輪郭。内径と外径を歯ごとに行き来する（X11 版の `gear_path` と同じ形）。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D のパスの幾何を組み立てる（ADR 0021）"
    )]
    fn gear_outline(
        &self,
        centre: Vector2,
        radius: f32,
    ) -> Result<windows::Win32::Graphics::Direct2D::ID2D1PathGeometry, Win32Failure> {
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
        // SAFETY: 幾何は工場から作り、シンクは Close まで、この関数の中だけで使う。
        unsafe {
            let geometry = self.factory.CreatePathGeometry()?;
            let sink = geometry.Open()?;
            if let Some(first) = points.first() {
                sink.BeginFigure(*first, D2D1_FIGURE_BEGIN_FILLED);
                sink.AddLines(&points);
                sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            }
            sink.Close()?;
            Ok(geometry)
        }
    }

    /// 1 色の筆。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の CreateSolidColorBrush を呼ぶ（ADR 0021）"
    )]
    fn brush(&self, ink: Rgb, alpha: f32) -> Result<ID2D1SolidColorBrush, Win32Failure> {
        let colour = D2D1_COLOR_F {
            r: f32::from(ink.red()) / 255.0,
            g: f32::from(ink.green()) / 255.0,
            b: f32::from(ink.blue()) / 255.0,
            a: alpha,
        };
        // SAFETY: colour はこの関数の局所で、呼び出しの間だけ読まれる。
        Ok(unsafe { self.target.CreateSolidColorBrush(&raw const colour, None) }?)
    }

    /// 整数座標の矩形を塗る。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の FillRectangle を呼ぶ（ADR 0021）"
    )]
    fn fill(&self, rect: PixelRect, brush: &ID2D1SolidColorBrush) {
        let area = D2D_RECT_F {
            left: f32::from(rect.x()),
            top: f32::from(rect.y()),
            right: f32::from(rect.x()) + f32::from(rect.width()),
            bottom: f32::from(rect.y()) + f32::from(rect.height()),
        };
        // SAFETY: 描画は paint の BeginDraw と EndDraw の間で呼ばれる。area はこの関数の局所。
        unsafe {
            self.target.FillRectangle(&raw const area, brush);
        }
    }

    /// 字を 1 行描く。`baseline` はベースラインの左端（`centred` なら中央）。
    #[expect(
        unsafe_code,
        reason = "RS-010: DirectWrite の文字の配置と Direct2D の DrawTextLayout を使う（ADR 0021）"
    )]
    fn write(
        &self,
        (text, baseline): (&str, PixelPoint),
        (size, centred): (f32, bool),
        brush: &ID2D1SolidColorBrush,
    ) -> Result<(), Win32Failure> {
        let layout = self.layout(text, size)?;
        let mut metrics = DWRITE_TEXT_METRICS::default();
        let mut line = [DWRITE_LINE_METRICS::default()];
        let mut lines = 0_u32;
        // SAFETY: 出力はこの関数の局所。行は 1 行なので 1 つ分の配列で足りる。
        unsafe {
            layout.GetMetrics(&raw mut metrics)?;
            layout.GetLineMetrics(Some(&mut line), &raw mut lines)?;
        }
        let ascent = line.first().map_or(size, |first| first.baseline);
        let left = if centred { metrics.width / 2.0 } else { 0.0 };
        let origin = Vector2 {
            X: f32::from(baseline.x()) - left,
            Y: f32::from(baseline.y()) - ascent,
        };
        // SAFETY: 描画は paint の BeginDraw と EndDraw の間で呼ばれる。
        unsafe {
            self.target
                .DrawTextLayout(origin, &layout, brush, D2D1_DRAW_TEXT_OPTIONS_NONE);
        }
        Ok(())
    }

    /// 1 行の文字の配置。
    #[expect(
        unsafe_code,
        reason = "RS-010: DirectWrite の CreateTextFormat / CreateTextLayout を呼ぶ（ADR 0021）"
    )]
    fn layout(&self, text: &str, size: f32) -> Result<IDWriteTextLayout, Win32Failure> {
        let wide: Vec<u16> = text.encode_utf16().collect();
        // SAFETY: 家族名と地域名は静的な文字列。wide はこの関数の局所。
        unsafe {
            let format = self.words.CreateTextFormat(
                FONT_FAMILY,
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                w!("en-us"),
            )?;
            Ok(self
                .words
                .CreateTextLayout(&wide, &format, 10_000.0, size * 4.0)?)
        }
    }

    /// 高さの数字のために、点の周りに -90 度回す。`false` で元に戻す。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の SetTransform を呼ぶ（ADR 0021）"
    )]
    fn rotate_about(&self, point: PixelPoint, rotated: bool) {
        let transform = if rotated {
            Matrix3x2::rotation_around(
                -90.0,
                Vector2 {
                    X: f32::from(point.x()),
                    Y: f32::from(point.y()),
                },
            )
        } else {
            Matrix3x2::identity()
        };
        // SAFETY: transform はこの関数の局所。
        unsafe {
            self.target.SetTransform(&raw const transform);
        }
    }

    /// 合成を「置き換え」にして描く。アルファ 1 の字や穴を、下のインクに混ぜずに置くため（ADR 0008）。
    /// レンダーターゲットが Direct2D 1.1 でなければ、普通に重ねる。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の SetPrimitiveBlend を呼ぶ（ADR 0021）"
    )]
    fn copying(&self, draw: impl FnOnce() -> Result<(), Win32Failure>) -> Result<(), Win32Failure> {
        let context = self.target.cast::<ID2D1DeviceContext>().ok();
        if let Some(context) = &context {
            // SAFETY: 同じレンダーターゲットの別の面。描画の間に合成の方法を変えるだけ。
            unsafe {
                context.SetPrimitiveBlend(D2D1_PRIMITIVE_BLEND_COPY);
            }
        }
        let drawn = draw();
        if let Some(context) = &context {
            // SAFETY: 上で変えたものを既定に戻す。
            unsafe {
                context.SetPrimitiveBlend(D2D1_PRIMITIVE_BLEND_SOURCE_OVER);
            }
        }
        drawn
    }
}

/// 透明（アルファ 0）。
const TRANSPARENT: D2D1_COLOR_F = D2D1_COLOR_F {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
};

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
