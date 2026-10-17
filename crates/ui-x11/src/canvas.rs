//! `RulerFrame` を cairo の `ImageSurface` に描く。X 無しで画素を検証できる（ADR 0002 の 4）。

use core::f64::consts::{FRAC_PI_2, PI};

use cairo::{Context, FontSlant, FontWeight, Format, ImageSurface, Operator};
use nene_ruler_application::copy_mark::CopyMark;
use nene_ruler_application::ruler_frame::RulerFrame;
use nene_ruler_application::transparency::Transparency;
use nene_ruler_core::axis::Axis;
use nene_ruler_core::edge::Edge;
use nene_ruler_core::frame_layout::FrameLayout;
use nene_ruler_core::pixel_point::PixelPoint;
use nene_ruler_core::pixel_rect::PixelRect;
use nene_ruler_core::rgb::Rgb;

use crate::ui_failure::UiFailure;

/// 数字のフォント。toy text API なので家族名だけ（ADR 0002）。
pub(crate) const FONT_FAMILY: &str = "sans-serif";
/// 幅・高さの数字の大きさ。
const LABEL_FONT_PX: f64 = 11.0;
/// 主目盛の数字の大きさ。
const TICK_FONT_PX: f64 = 9.0;
/// ギアの歯の数。
const GEAR_TEETH: u32 = 8;

/// 完成した 1 フレームの画素の列。
#[derive(Debug)]
pub struct Canvas {
    surface: ImageSurface,
    width: u16,
    height: u16,
}

impl Canvas {
    /// 枠のフレームを描き切る。透明なら背景はアルファ 0 のまま、不透明なら補色で塗る。
    ///
    /// # Errors
    ///
    /// cairo が surface を作れない、または描画に失敗した。
    pub fn render(frame: &RulerFrame, format: Format) -> Result<Self, UiFailure> {
        Self::paint(frame.layout().window(), format, |context| {
            paint_background(context, frame)?;
            underlay_hit_regions(context, frame)?;
            set_ink(context, frame.ink());
            for rect in frame.layout().ink_rects() {
                fill_rect(context, rect)?;
            }
            draw_tick_labels(context, frame)?;
            draw_label(context, frame, Axis::Horizontal)?;
            draw_label(context, frame, Axis::Vertical)?;
            draw_notices(context, frame)?;
            draw_gear(context, frame.layout().gear(), frame.ink())
        })
    }

    /// `bounds` の大きさの surface を作り、`draw` で描き切る。
    ///
    /// # Errors
    ///
    /// cairo が surface を作れない、または `draw` が失敗した。
    pub fn paint<D: FnOnce(&Context) -> Result<(), UiFailure>>(
        bounds: PixelRect,
        format: Format,
        draw: D,
    ) -> Result<Self, UiFailure> {
        let surface = ImageSurface::create(
            format,
            i32::from(bounds.width()),
            i32::from(bounds.height()),
        )?;
        {
            let context = Context::new(&surface)?;
            draw(&context)?;
        }
        surface.flush();
        Ok(Self {
            surface,
            width: bounds.width(),
            height: bounds.height(),
        })
    }

    /// 幅。
    #[must_use]
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// 高さ。
    #[must_use]
    pub const fn height(&self) -> u16 {
        self.height
    }

    /// 1 行のバイト数。
    ///
    /// # Errors
    ///
    /// cairo の stride が負（起こらないが型の上では `i32`）。
    pub fn stride(&self) -> Result<usize, UiFailure> {
        usize::try_from(self.surface.stride()).map_err(|_error| UiFailure::GeometryOutOfRange)
    }

    /// 画素の列をそのまま読む。ネイティブエンディアンの 32 ビット（X の `ZPixmap` と同じ並び）。
    ///
    /// # Errors
    ///
    /// 描画中の `Context` が残っていて借りられない。
    pub fn with_pixels<T, F: FnOnce(&[u8]) -> T>(&mut self, read: F) -> Result<T, UiFailure> {
        let bytes = self.surface.data()?;
        Ok(read(&bytes))
    }

    /// 1 画素の `(blue, green, red, alpha)`。リトルエンディアンの ARGB32 の並び。範囲外は `None`。
    ///
    /// # Errors
    ///
    /// 画素を借りられない。
    pub fn pixel(&mut self, x: u16, y: u16) -> Result<Option<[u8; 4]>, UiFailure> {
        if x >= self.width || y >= self.height {
            return Ok(None);
        }
        let stride = self.stride()?;
        let offset = usize::from(y) * stride + usize::from(x) * 4;
        self.with_pixels(|bytes| {
            bytes
                .get(offset..offset.checked_add(4)?)
                .and_then(|slice| <[u8; 4]>::try_from(slice).ok())
        })
    }
}

/// 不透明なら背景をインクの補色で塗る。透明なら何もしない（アルファ 0 のまま）。
fn paint_background(context: &Context, frame: &RulerFrame) -> Result<(), UiFailure> {
    match frame.transparency() {
        Transparency::Composited => Ok(()),
        Transparency::Opaque => {
            set_ink(context, frame.ink().contrasting_background());
            context.paint()?;
            Ok(())
        }
    }
}

/// 目に見えないが当たり判定には乗るアルファ。WSLg は描いた画素だけを当たりにする（ADR 0008）。
const TOUCHABLE_ALPHA: f64 = 1.0_f64 / 255.0_f64;

/// 当たり判定の矩形をアルファ 1/255 のインクで下塗りする。透明なときだけ（不透明なら背景がある）。
fn underlay_hit_regions(context: &Context, frame: &RulerFrame) -> Result<(), UiFailure> {
    if frame.transparency() == Transparency::Opaque {
        return Ok(());
    }
    set_touchable(context, frame.ink());
    for region in frame.hit_regions() {
        fill_rect(context, region.rect())?;
    }
    Ok(())
}

/// 描く色を、見えないが触れるインク色にする（`Operator::Source` で置けば透明の代わりになる）。
fn set_touchable(context: &Context, ink: Rgb) {
    context.set_source_rgba(
        f64::from(ink.red()) / 255.0_f64,
        f64::from(ink.green()) / 255.0_f64,
        f64::from(ink.blue()) / 255.0_f64,
        TOUCHABLE_ALPHA,
    );
}

/// 描く色を不透明のインク色にする。
pub(crate) fn set_ink(context: &Context, ink: Rgb) {
    context.set_source_rgb(
        f64::from(ink.red()) / 255.0_f64,
        f64::from(ink.green()) / 255.0_f64,
        f64::from(ink.blue()) / 255.0_f64,
    );
}

/// 整数座標の矩形を塗る。線・目盛・鉤括弧はすべてこれ（アンチエイリアスが要らない）。
pub(crate) fn fill_rect(context: &Context, rect: PixelRect) -> Result<(), UiFailure> {
    context.rectangle(
        f64::from(rect.x()),
        f64::from(rect.y()),
        f64::from(rect.width()),
        f64::from(rect.height()),
    );
    context.fill()?;
    Ok(())
}

/// 上辺と左辺の主目盛の数字。
fn draw_tick_labels(context: &Context, frame: &RulerFrame) -> Result<(), UiFailure> {
    context.select_font_face(FONT_FAMILY, FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(TICK_FONT_PX);
    for edge in [Edge::Top, Edge::Left] {
        for label in frame.layout().tick_labels(edge) {
            let baseline = label.baseline();
            context.move_to(f64::from(baseline.x()), f64::from(baseline.y()));
            context.show_text(&label.value().to_string())?;
        }
    }
    Ok(())
}

/// 幅・高さの数字。横軸は上辺の中央、縦軸は左辺の中央に 90 度回して。
/// コピーの印があれば、成功は反転（インクの箱に透明の文字）、失敗は取り消し線（ADR 0007 の 5）。
fn draw_label(context: &Context, frame: &RulerFrame, axis: Axis) -> Result<(), UiFailure> {
    let text = frame.text(axis).as_str();
    let baseline = frame.layout().label_baseline(axis);
    let mark = frame.copy_mark().filter(|mark| mark.axis() == axis);
    let hit = frame.layout().label_hit(axis, frame.text(axis).chars());
    if mark.is_some_and(CopyMark::succeeded) {
        fill_rect(context, hit)?;
    }
    context.select_font_face(FONT_FAMILY, FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(LABEL_FONT_PX);
    let extents = context.text_extents(text)?;
    let centred = -(extents.width() / 2.0_f64 + extents.x_bearing());
    context.save()?;
    translate_to(context, baseline);
    if axis == Axis::Vertical {
        context.rotate(-FRAC_PI_2);
    }
    if mark.is_some_and(CopyMark::succeeded) {
        // 字画は透明にせず「見えないが触れる」色で置く（ADR 0008）。
        context.set_operator(Operator::Source);
        set_touchable(context, frame.ink());
    }
    context.move_to(centred, 0.0_f64);
    context.show_text(text)?;
    context.restore()?;
    if mark.is_some_and(|mark| !mark.succeeded()) {
        strike_through(context, hit, axis)?;
    }
    Ok(())
}

/// 枠の中の通知（FR-014）。下辺の目盛の上に、下から上へ。
fn draw_notices(context: &Context, frame: &RulerFrame) -> Result<(), UiFailure> {
    context.select_font_face(FONT_FAMILY, FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(f64::from(FrameLayout::notice_px()));
    for (index, notice) in frame.notices().list().into_iter().enumerate() {
        let index = u16::try_from(index).map_err(|_error| UiFailure::GeometryOutOfRange)?;
        let baseline = frame.layout().notice_baseline(index);
        context.move_to(f64::from(baseline.x()), f64::from(baseline.y()));
        context.show_text(notice.describe())?;
    }
    Ok(())
}

/// 数字の中央を横切る 1 px の線。縦軸の数字は縦に走る。
fn strike_through(context: &Context, hit: PixelRect, axis: Axis) -> Result<(), UiFailure> {
    let line = match axis {
        Axis::Horizontal => PixelRect::new(hit.x(), hit.y() + hit.height() / 2, hit.width(), 1),
        Axis::Vertical => PixelRect::new(hit.x() + hit.width() / 2, hit.y(), 1, hit.height()),
    };
    fill_rect(context, line)
}

/// 原点を点へ移す。
pub(crate) fn translate_to(context: &Context, point: PixelPoint) {
    context.translate(f64::from(point.x()), f64::from(point.y()));
}

/// ギア。丸をインク色で塗り、歯車の形を「見えないが触れる」色で抜き、軸だけインクで戻す（FR-009 / ADR 0008）。
fn draw_gear(context: &Context, bounds: PixelRect, ink: Rgb) -> Result<(), UiFailure> {
    let radius = f64::from(bounds.width()) / 2.0_f64;
    let centre_x = f64::from(bounds.x()) + radius;
    let centre_y = f64::from(bounds.y()) + radius;
    context.new_path();
    context.arc(centre_x, centre_y, radius, 0.0_f64, 2.0_f64 * PI);
    context.fill()?;

    context.save()?;
    context.set_operator(Operator::Source);
    set_touchable(context, ink);
    gear_path(context, centre_x, centre_y, radius);
    context.fill()?;
    context.restore()?;
    set_ink(context, ink);

    context.new_path();
    context.arc(centre_x, centre_y, radius * 0.2_f64, 0.0_f64, 2.0_f64 * PI);
    context.fill()?;
    Ok(())
}

/// 歯車の輪郭。内径と外径を歯ごとに行き来する。
fn gear_path(context: &Context, centre_x: f64, centre_y: f64, radius: f64) {
    let inner = radius * 0.48_f64;
    let outer = radius * 0.72_f64;
    let pitch = 2.0_f64 * PI / f64::from(GEAR_TEETH);
    let half_tooth = pitch / 4.0_f64;
    context.new_path();
    for tooth in 0..GEAR_TEETH {
        let middle = f64::from(tooth) * pitch;
        context.arc(
            centre_x,
            centre_y,
            inner,
            middle - 2.0_f64 * half_tooth,
            middle - half_tooth,
        );
        context.arc(
            centre_x,
            centre_y,
            outer,
            middle - half_tooth,
            middle + half_tooth,
        );
    }
    context.close_path();
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ruler_application::ruler_controller::RulerController;
    use nene_ruler_application::screen_metrics_port::ScreenMetricsPort;
    use nene_ruler_core::screen_metrics::ScreenMetrics;
    use nene_ruler_core::screen_metrics_failure::ScreenMetricsFailure;

    struct NoScreen;

    impl ScreenMetricsPort for NoScreen {
        fn screen_metrics(&self) -> Result<ScreenMetrics, ScreenMetricsFailure> {
            Err(ScreenMetricsFailure::Unavailable)
        }
    }

    struct NoStore;

    impl nene_ruler_application::settings_store_port::SettingsStorePort for NoStore {
        fn load(
            &self,
        ) -> Result<
            Option<nene_ruler_core::ruler_settings::RulerSettings>,
            nene_ruler_application::settings_store_failure::SettingsStoreFailure,
        > {
            Ok(None)
        }

        fn save(
            &self,
            _settings: nene_ruler_core::ruler_settings::RulerSettings,
        ) -> Result<(), nene_ruler_application::settings_store_failure::SettingsStoreFailure>
        {
            Ok(())
        }
    }

    struct FakeClipboard(Result<(), nene_ruler_application::clipboard_failure::ClipboardFailure>);

    impl nene_ruler_application::clipboard_port::ClipboardPort for FakeClipboard {
        fn copy(
            &self,
            _text: &str,
        ) -> Result<(), nene_ruler_application::clipboard_failure::ClipboardFailure> {
            self.0
        }
    }

    fn controller_with(
        transparency: Transparency,
        clipboard: &FakeClipboard,
    ) -> RulerController<'_> {
        RulerController::new(
            nene_ruler_application::ports::Ports::new(&NoScreen, &NoStore, clipboard),
            nene_ruler_application::environment::Environment::new(
                transparency,
                nene_ruler_application::layering::Layering::Supported,
            ),
            nene_ruler_application::product_version::ProductVersion::new("0.1.0"),
        )
    }

    fn canvas(transparency: Transparency) -> Canvas {
        let board = FakeClipboard(Ok(()));
        let controller = controller_with(transparency, &board);
        Canvas::render(&controller.frame(), Format::ARgb32).unwrap()
    }

    #[test]
    fn notices_are_drawn_only_when_something_is_degraded() {
        let board = FakeClipboard(Ok(()));
        let opaque = controller_with(Transparency::Opaque, &board);
        let frame = opaque.frame();
        let baseline = frame.layout().notice_baseline(0);
        let mut canvas = Canvas::render(&frame, Format::ARgb32).unwrap();
        // 不透明の背景（白）の上に、通知の文字のインクが混ざる。
        let row: Vec<[u8; 4]> = (baseline.x()..baseline.x() + 120)
            .filter_map(|x| canvas.pixel(x, baseline.y() - 3).unwrap())
            .collect();
        assert!(row.iter().any(|p| *p != [0xFF, 0xFF, 0xFF, 0xFF]));
        let mut clean = self::canvas(Transparency::Composited);
        let clean_row: Vec<[u8; 4]> = (baseline.x()..baseline.x() + 120)
            .filter_map(|x| clean.pixel(x, baseline.y() - 3).unwrap())
            .collect();
        assert!(clean_row.iter().all(|p| p[3] == 0));
    }

    #[test]
    fn a_successful_copy_inverts_the_number() {
        use nene_ruler_application::ruler_intent::RulerIntent;
        let board = FakeClipboard(Ok(()));
        let mut controller = controller_with(Transparency::Composited, &board);
        controller.apply(RulerIntent::CopyText(Axis::Horizontal));
        let frame = controller.frame();
        let hit = frame
            .layout()
            .label_hit(Axis::Horizontal, frame.text(Axis::Horizontal).chars());
        let mut canvas = Canvas::render(&frame, Format::ARgb32).unwrap();
        // 箱の角はインク、文字の無い所も含めて箱の中は透明の文字以外がインク。
        assert_eq!(
            canvas.pixel(hit.x(), hit.y()).unwrap(),
            Some([0x35, 0x39, 0xE5, 0xFF])
        );
        let inside: Vec<[u8; 4]> = (hit.x()..=hit.right())
            .filter_map(|x| canvas.pixel(x, hit.y() + 2).unwrap())
            .collect();
        assert!(inside.iter().any(|p| p[3] == 0xFF));
        assert!(inside.iter().any(|p| p[3] < 0xFF));
        assert!(inside.iter().all(|p| p[3] >= 1));
    }

    #[test]
    fn a_failed_copy_strikes_the_number_through() {
        use nene_ruler_application::clipboard_failure::ClipboardFailure;
        use nene_ruler_application::ruler_intent::RulerIntent;
        let board = FakeClipboard(Err(ClipboardFailure::Rejected));
        let mut controller = controller_with(Transparency::Composited, &board);
        controller.apply(RulerIntent::CopyText(Axis::Vertical));
        let frame = controller.frame();
        let hit = frame
            .layout()
            .label_hit(Axis::Vertical, frame.text(Axis::Vertical).chars());
        let mut canvas = Canvas::render(&frame, Format::ARgb32).unwrap();
        let x = hit.x() + hit.width() / 2;
        let column: Vec<[u8; 4]> = (hit.y()..=hit.bottom())
            .filter_map(|y| canvas.pixel(x, y).unwrap())
            .collect();
        assert!(column.iter().all(|p| p[3] == 0xFF));
        // 反転ではないので、数字の箱の外（右隣）は透明のまま。
        assert_eq!(
            canvas
                .pixel(hit.right() + 3, hit.y() + hit.height() / 2)
                .unwrap(),
            Some([0, 0, 0, 0])
        );
    }

    #[test]
    fn transparent_background_has_zero_alpha() {
        let mut canvas = canvas(Transparency::Composited);
        assert_eq!((canvas.width(), canvas.height()), (424, 264));
        assert_eq!(canvas.pixel(200, 120).unwrap(), Some([0, 0, 0, 0]));
        // 窓の角は上辺の帯・左辺の帯・左上の鉤括弧の当たり判定が重なる所。
        // 「見えないが触れる」アルファ 1/255 が重なって 3 になるが、目には見えない。
        let corner = canvas.pixel(0, 0).unwrap().unwrap();
        assert!((1..=4).contains(&corner[3]), "{corner:?}");
    }

    #[test]
    fn every_hit_region_is_touchable_and_the_interior_is_not() {
        let mut canvas = canvas(Transparency::Composited);
        let board = FakeClipboard(Ok(()));
        let frame = controller_with(Transparency::Composited, &board).frame();
        for region in frame.hit_regions() {
            let rect = region.rect();
            for (x, y) in [
                (rect.x(), rect.y()),
                (rect.right(), rect.bottom()),
                (rect.x() + rect.width() / 2, rect.y() + rect.height() / 2),
            ] {
                assert!(
                    canvas.pixel(x, y).unwrap().is_some_and(|p| p[3] >= 1),
                    "{region:?}"
                );
            }
        }
        assert_eq!(canvas.pixel(100, 60).unwrap(), Some([0, 0, 0, 0]));
    }

    #[test]
    fn lines_and_anchors_are_opaque_ink() {
        let mut canvas = canvas(Transparency::Composited);
        let ink = [0x35, 0x39, 0xE5, 0xFF];
        assert_eq!(canvas.pixel(200, 4).unwrap(), Some(ink));
        assert_eq!(canvas.pixel(4, 120).unwrap(), Some(ink));
        assert_eq!(canvas.pixel(403, 120).unwrap(), Some(ink));
        assert_eq!(canvas.pixel(200, 243).unwrap(), Some(ink));
        assert_eq!(canvas.pixel(10, 5).unwrap(), Some(ink));
        assert_eq!(canvas.pixel(54, 10).unwrap(), Some(ink));
        assert_eq!(canvas.pixel(403, 240).unwrap(), Some(ink));
    }

    #[test]
    fn gear_is_a_disc_with_a_transparent_gear_cut_out() {
        let mut canvas = canvas(Transparency::Composited);
        let ink = [0x35, 0x39, 0xE5, 0xFF];
        assert_eq!(canvas.pixel(409, 256).unwrap(), Some(ink));
        // 歯車の抜きは透明ではなく「見えないが触れる」アルファ 1（ADR 0008）。
        assert_eq!(canvas.pixel(418, 256).unwrap().map(|p| p[3]), Some(1));
        assert_eq!(canvas.pixel(415, 255).unwrap(), Some(ink));
        assert_eq!(canvas.pixel(430, 256).unwrap(), None);
    }

    #[test]
    fn opaque_fallback_paints_a_contrasting_background() {
        let mut canvas = canvas(Transparency::Opaque);
        assert_eq!(
            canvas.pixel(200, 120).unwrap(),
            Some([0xFF, 0xFF, 0xFF, 0xFF])
        );
        assert_eq!(
            canvas.pixel(200, 4).unwrap(),
            Some([0x35, 0x39, 0xE5, 0xFF])
        );
    }

    #[test]
    fn labels_leave_ink_near_their_baselines() {
        let mut canvas = canvas(Transparency::Composited);
        let row: Vec<u8> = (190_u16..220)
            .map(|x| canvas.pixel(x, 38).unwrap().map_or(0, |p| p[3]))
            .collect();
        assert!(row.iter().any(|alpha| *alpha > 0));
        assert!(canvas.pixel(100, 60).unwrap().is_some_and(|p| p[3] == 0));
    }
}
