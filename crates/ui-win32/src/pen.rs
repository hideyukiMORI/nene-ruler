//! Direct2D / DirectWrite の描く道具。枠とモーダルの両方が使う（ADR 0021）。
//!
//! 1 DIP = 1 物理画素（レンダーターゲットの dpi を 96 に固定）。測る値は物理画素のまま（FR-016）。

use nene_ruler_core::pixel_point::PixelPoint;
use nene_ruler_core::pixel_rect::PixelRect;
use nene_ruler_core::rgb::Rgb;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D_RECT_F, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_FIGURE_BEGIN_FILLED,
    D2D1_FIGURE_END_CLOSED, D2D1_GRADIENT_STOP, D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE, D2D1_EXTEND_MODE_CLAMP,
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT, D2D1_GAMMA_2_2,
    D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES, D2D1_PRIMITIVE_BLEND_COPY,
    D2D1_PRIMITIVE_BLEND_SOURCE_OVER, D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE,
    D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE, D2D1CreateFactory, ID2D1Brush, ID2D1DCRenderTarget,
    ID2D1DeviceContext, ID2D1Factory, ID2D1SolidColorBrush,
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
use crate::text_align::TextAlign;
use crate::win32_failure::Win32Failure;

/// 字の家族。Windows の UI の字。
const FONT_FAMILY: windows::core::PCWSTR = w!("Segoe UI");

/// Direct2D と DirectWrite の工場と、DC に結ぶレンダーターゲット。
#[derive(Debug)]
pub struct Pen {
    target: ID2D1DCRenderTarget,
    factory: ID2D1Factory,
    words: IDWriteFactory,
}

impl Pen {
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

    /// `bounds` の大きさの透明な画素を作り、`draw` で描き切る。
    ///
    /// # Errors
    ///
    /// DIB か Direct2D の描画が失敗した。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の BindDC / BeginDraw / EndDraw を呼ぶ（ADR 0021）"
    )]
    pub fn paint<D: FnOnce(&Self) -> Result<(), Win32Failure>>(
        &self,
        bounds: PixelRect,
        draw: D,
    ) -> Result<LayeredBitmap, Win32Failure> {
        let bitmap = LayeredBitmap::new(bounds.width(), bounds.height())?;
        let area = RECT {
            left: 0,
            top: 0,
            right: i32::from(bounds.width()),
            bottom: i32::from(bounds.height()),
        };
        let clear = colour(Rgb::new(0, 0, 0), 0.0);
        // SAFETY: bitmap の DC はこの関数の間生きている。描画は BeginDraw と EndDraw の間だけ。
        unsafe {
            self.target.BindDC(bitmap.dc(), &raw const area)?;
            self.target.BeginDraw();
            self.target.Clear(Some(&raw const clear));
            self.target
                .SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
        }
        let drawn = draw(self);
        // SAFETY: 上の BeginDraw と対になる。
        unsafe { self.target.EndDraw(None, None) }?;
        drawn.map(|()| bitmap)
    }

    /// 1 色の筆。
    ///
    /// # Errors
    ///
    /// Direct2D が筆を作れなかった。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の CreateSolidColorBrush を呼ぶ（ADR 0021）"
    )]
    pub fn brush(&self, ink: Rgb, alpha: f32) -> Result<ID2D1Brush, Win32Failure> {
        let ink = colour(ink, alpha);
        // SAFETY: ink はこの関数の局所で、呼び出しの間だけ読まれる。
        let solid: ID2D1SolidColorBrush =
            unsafe { self.target.CreateSolidColorBrush(&raw const ink, None) }?;
        Ok(solid.into())
    }

    /// 整数座標の矩形を塗る。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の FillRectangle を呼ぶ（ADR 0021）"
    )]
    pub fn fill(&self, rect: PixelRect, brush: &ID2D1Brush) {
        let area = rect_f(rect);
        // SAFETY: 描画は paint の中で呼ばれる。area はこの関数の局所。
        unsafe {
            self.target.FillRectangle(&raw const area, brush);
        }
    }

    /// 円を塗る。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の FillEllipse を呼ぶ（ADR 0021）"
    )]
    pub fn fill_disc(&self, centre: Vector2, radius: f32, brush: &ID2D1Brush) {
        let disc = D2D1_ELLIPSE {
            point: centre,
            radiusX: radius,
            radiusY: radius,
        };
        // SAFETY: 描画は paint の中で呼ばれる。disc はこの関数の局所。
        unsafe {
            self.target.FillEllipse(&raw const disc, brush);
        }
    }

    /// 円の輪郭を描く。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の DrawEllipse を呼ぶ（ADR 0021）"
    )]
    pub fn stroke_circle(&self, centre: Vector2, radius: f32, (width, brush): (f32, &ID2D1Brush)) {
        let disc = D2D1_ELLIPSE {
            point: centre,
            radiusX: radius,
            radiusY: radius,
        };
        // SAFETY: 描画は paint の中で呼ばれる。disc はこの関数の局所。
        unsafe {
            self.target.DrawEllipse(&raw const disc, brush, width, None);
        }
    }

    /// 線分を描く。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の DrawLine を呼ぶ（ADR 0021）"
    )]
    pub fn stroke_line(&self, (from, to): (Vector2, Vector2), (width, brush): (f32, &ID2D1Brush)) {
        // SAFETY: 描画は paint の中で呼ばれる。
        unsafe {
            self.target.DrawLine(from, to, brush, width, None);
        }
    }

    /// 閉じた多角形を塗る。
    ///
    /// # Errors
    ///
    /// 幾何を作れなかった。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D のパスの幾何を組み立てて塗る（ADR 0021）"
    )]
    pub fn fill_polygon(&self, points: &[Vector2], brush: &ID2D1Brush) -> Result<(), Win32Failure> {
        let Some(first) = points.first() else {
            return Ok(());
        };
        // SAFETY: 幾何は工場から作り、シンクは Close まで、この関数の中だけで使う。描画は paint の中。
        unsafe {
            let geometry = self.factory.CreatePathGeometry()?;
            let sink = geometry.Open()?;
            sink.BeginFigure(*first, D2D1_FIGURE_BEGIN_FILLED);
            sink.AddLines(points);
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            sink.Close()?;
            self.target.FillGeometry(&geometry, brush, None);
        }
        Ok(())
    }

    /// 矩形を線形のグラデーションで塗る。`stops` は位置（0〜1）と色。`vertical` なら上から下へ。
    ///
    /// # Errors
    ///
    /// 筆を作れなかった。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の線形のグラデーションの筆を作る（ADR 0021）"
    )]
    pub fn fill_gradient(
        &self,
        rect: PixelRect,
        stops: &[(f32, Rgb, f32)],
        vertical: bool,
    ) -> Result<(), Win32Failure> {
        let area = rect_f(rect);
        let gradient: Vec<D2D1_GRADIENT_STOP> = stops
            .iter()
            .map(|&(position, ink, alpha)| D2D1_GRADIENT_STOP {
                position,
                color: colour(ink, alpha),
            })
            .collect();
        let ends = D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
            startPoint: Vector2 {
                X: area.left,
                Y: area.top,
            },
            endPoint: if vertical {
                Vector2 {
                    X: area.left,
                    Y: area.bottom,
                }
            } else {
                Vector2 {
                    X: area.right,
                    Y: area.top,
                }
            },
        };
        // SAFETY: 区切りと端はこの関数の局所。描画は paint の中。
        unsafe {
            let collection = self.target.CreateGradientStopCollection(
                &gradient,
                D2D1_GAMMA_2_2,
                D2D1_EXTEND_MODE_CLAMP,
            )?;
            let brush =
                self.target
                    .CreateLinearGradientBrush(&raw const ends, None, &collection)?;
            self.target.FillRectangle(&raw const area, &brush);
        }
        Ok(())
    }

    /// 字を 1 行描く。`baseline` はベースラインの左端（`Centre` なら中央）。描いた字の右端の x を返す。
    ///
    /// # Errors
    ///
    /// 字の配置を作れなかった。
    #[expect(
        unsafe_code,
        reason = "RS-010: DirectWrite の文字の配置と Direct2D の DrawTextLayout を使う（ADR 0021）"
    )]
    pub fn write(
        &self,
        (text, baseline): (&str, PixelPoint),
        (size, align): (f32, TextAlign),
        brush: &ID2D1Brush,
    ) -> Result<f32, Win32Failure> {
        let (layout, width, ascent) = self.measure(text, size)?;
        let left = match align {
            TextAlign::Left => 0.0,
            TextAlign::Centre => width / 2.0,
        };
        let origin = Vector2 {
            X: f32::from(baseline.x()) - left,
            Y: f32::from(baseline.y()) - ascent,
        };
        // SAFETY: 描画は paint の中で呼ばれる。
        unsafe {
            self.target
                .DrawTextLayout(origin, &layout, brush, D2D1_DRAW_TEXT_OPTIONS_NONE);
        }
        Ok(origin.X + width)
    }

    /// 矩形の中央に字を置く（ボタンの字）。
    ///
    /// # Errors
    ///
    /// 字の配置を作れなかった。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の DrawTextLayout を呼ぶ（ADR 0021）"
    )]
    pub fn write_in(
        &self,
        (text, rect): (&str, PixelRect),
        size: f32,
        brush: &ID2D1Brush,
    ) -> Result<(), Win32Failure> {
        let (layout, width, _ascent) = self.measure(text, size)?;
        let mut line = [DWRITE_LINE_METRICS::default()];
        let mut lines = 0_u32;
        // SAFETY: 出力はこの関数の局所。1 行。
        unsafe { layout.GetLineMetrics(Some(&mut line), &raw mut lines) }?;
        let height = line.first().map_or(size, |first| first.height);
        let origin = Vector2 {
            X: f32::from(rect.x()) + (f32::from(rect.width()) - width) / 2.0,
            Y: f32::from(rect.y()) + (f32::from(rect.height()) - height) / 2.0,
        };
        // SAFETY: 描画は paint の中で呼ばれる。
        unsafe {
            self.target
                .DrawTextLayout(origin, &layout, brush, D2D1_DRAW_TEXT_OPTIONS_NONE);
        }
        Ok(())
    }

    /// 点の周りに -90 度回す（高さの数字）。`false` で元に戻す。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の SetTransform を呼ぶ（ADR 0021）"
    )]
    pub fn rotate_about(&self, point: PixelPoint, rotated: bool) {
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
    ///
    /// # Errors
    ///
    /// `draw` が失敗した。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の SetPrimitiveBlend を呼ぶ（ADR 0021）"
    )]
    pub fn copying<D: FnOnce() -> Result<(), Win32Failure>>(
        &self,
        draw: D,
    ) -> Result<(), Win32Failure> {
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

    /// 1 行の字の配置と、その幅とベースラインまでの高さ。
    #[expect(
        unsafe_code,
        reason = "RS-010: DirectWrite の文字の配置を作り、寸法を読む（ADR 0021）"
    )]
    fn measure(
        &self,
        text: &str,
        size: f32,
    ) -> Result<(IDWriteTextLayout, f32, f32), Win32Failure> {
        let wide: Vec<u16> = text.encode_utf16().collect();
        let mut metrics = DWRITE_TEXT_METRICS::default();
        let mut line = [DWRITE_LINE_METRICS::default()];
        let mut lines = 0_u32;
        // SAFETY: 家族名と地域名は静的な文字列。wide と出力はこの関数の局所。1 行。
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
            let layout = self
                .words
                .CreateTextLayout(&wide, &format, 10_000.0, size * 4.0)?;
            layout.GetMetrics(&raw mut metrics)?;
            layout.GetLineMetrics(Some(&mut line), &raw mut lines)?;
            let ascent = line.first().map_or(size, |first| first.baseline);
            Ok((layout, metrics.width, ascent))
        }
    }
}

/// 0〜255 の色とアルファを Direct2D の色へ。
fn colour(ink: Rgb, alpha: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: f32::from(ink.red()) / 255.0,
        g: f32::from(ink.green()) / 255.0,
        b: f32::from(ink.blue()) / 255.0,
        a: alpha,
    }
}

/// 整数座標の矩形を Direct2D の矩形へ。
fn rect_f(rect: PixelRect) -> D2D_RECT_F {
    D2D_RECT_F {
        left: f32::from(rect.x()),
        top: f32::from(rect.y()),
        right: f32::from(rect.x()) + f32::from(rect.width()),
        bottom: f32::from(rect.y()) + f32::from(rect.height()),
    }
}
