//! 物差しの状態の唯一の所有者（ARC-004）。

use nene_ruler_core::axis::Axis;
use nene_ruler_core::frame_layout::FrameLayout;
use nene_ruler_core::frame_size::FrameSize;
use nene_ruler_core::ink_color::InkColor;
use nene_ruler_core::length::Length;
use nene_ruler_core::length_text::LengthText;
use nene_ruler_core::screen_metrics::ScreenMetrics;
use nene_ruler_core::screen_metrics_failure::ScreenMetricsFailure;
use nene_ruler_core::unit::Unit;

use crate::frame_texts::FrameTexts;
use crate::ruler_frame::RulerFrame;
use crate::ruler_intent::RulerIntent;
use crate::screen_metrics_port::ScreenMetricsPort;
use crate::transparency::Transparency;

/// 枠・単位・インク色・物理寸法・透明を 1 つ持ち、意図で次の状態へ置き換える。
///
/// 公開値はすべて不変で、変わるのは `apply` の中だけ（ARC-005）。
#[derive(Debug)]
pub struct RulerController {
    size: FrameSize,
    unit: Unit,
    ink: InkColor,
    metrics: Result<ScreenMetrics, ScreenMetricsFailure>,
    transparency: Transparency,
}

impl RulerController {
    /// 初期の枠で始める。物理寸法はここで一度だけ取り込む（ARC-004 の表）。
    pub fn new(metrics_port: &dyn ScreenMetricsPort, transparency: Transparency) -> Self {
        Self {
            size: FrameSize::INITIAL,
            unit: Unit::Px,
            ink: InkColor::INITIAL,
            metrics: metrics_port.screen_metrics(),
            transparency,
        }
    }

    /// 意図を反映して次の状態にする。
    pub fn apply(&mut self, intent: RulerIntent) {
        match intent {
            RulerIntent::WindowResized {
                width_px,
                height_px,
            } => {
                self.size = FrameSize::new(
                    self.clamp(FrameLayout::frame_px_for_window(width_px), Axis::Horizontal),
                    self.clamp(FrameLayout::frame_px_for_window(height_px), Axis::Vertical),
                );
            }
        }
    }

    /// いまの状態から表示フレームを作る。
    #[must_use]
    pub fn frame(&self) -> RulerFrame {
        RulerFrame::new(
            FrameLayout::new(self.size),
            FrameTexts::new(self.text(Axis::Horizontal), self.text(Axis::Vertical)),
            self.ink,
            self.transparency,
        )
    }

    /// 物理寸法が取れなかった理由。利用者に示すため（FR-014）。
    #[must_use]
    pub fn metrics_failure(&self) -> Option<ScreenMetricsFailure> {
        self.metrics.err()
    }

    /// 外から来た長さを下限と画面の大きさの間に収める（FR-005）。
    fn clamp(&self, px: u16, axis: Axis) -> Length {
        let screen = self
            .metrics
            .map_or(u32::from(Length::MAX_PX), |metrics| metrics.pixels(axis));
        let limit = u16::try_from(screen).unwrap_or(Length::MAX_PX);
        let max = Length::new(limit).unwrap_or(Length::INITIAL_WIDTH);
        Length::clamped(px, max)
    }

    /// 軸の長さを単位に従って綴る。物理寸法が無ければ px に落ちる（FR-008）。
    fn text(&self, axis: Axis) -> LengthText {
        let length = self.size.along(axis);
        match (self.unit, self.metrics) {
            (Unit::Px, _) | (Unit::Mm | Unit::In, Err(_)) => LengthText::in_pixels(length),
            (Unit::Mm, Ok(metrics)) => {
                LengthText::in_millimetres(length, metrics.millimetres_per_pixel(axis))
            }
            (Unit::In, Ok(metrics)) => {
                LengthText::in_inches(length, metrics.millimetres_per_pixel(axis))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedScreen(Result<ScreenMetrics, ScreenMetricsFailure>);

    impl ScreenMetricsPort for FixedScreen {
        fn screen_metrics(&self) -> Result<ScreenMetrics, ScreenMetricsFailure> {
            self.0
        }
    }

    fn screen() -> FixedScreen {
        FixedScreen(ScreenMetrics::new(1920, 1080, 480, 270))
    }

    #[test]
    fn starts_with_the_initial_frame_in_pixels() {
        let controller = RulerController::new(&screen(), Transparency::Composited);
        let frame = controller.frame();
        assert_eq!(frame.text(Axis::Horizontal).as_str(), "400 px");
        assert_eq!(frame.text(Axis::Vertical).as_str(), "240 px");
        assert_eq!(frame.ink(), InkColor::INITIAL);
        assert_eq!(frame.transparency(), Transparency::Composited);
        assert_eq!(controller.metrics_failure(), None);
    }

    #[test]
    fn window_size_becomes_frame_size_within_the_screen() {
        let mut controller = RulerController::new(&screen(), Transparency::Opaque);
        controller.apply(RulerIntent::WindowResized {
            width_px: 524,
            height_px: 324,
        });
        assert_eq!(controller.frame().layout().size().width().px(), 500);
        assert_eq!(controller.frame().layout().size().height().px(), 300);
        controller.apply(RulerIntent::WindowResized {
            width_px: 5_000,
            height_px: 10,
        });
        assert_eq!(controller.frame().layout().size().width().px(), 1920);
        assert_eq!(
            controller.frame().layout().size().height().px(),
            Length::MIN_PX
        );
    }

    #[test]
    fn without_metrics_the_limit_is_the_absolute_maximum() {
        let port = FixedScreen(Err(ScreenMetricsFailure::Unavailable));
        let mut controller = RulerController::new(&port, Transparency::Composited);
        assert_eq!(
            controller.metrics_failure(),
            Some(ScreenMetricsFailure::Unavailable)
        );
        controller.apply(RulerIntent::WindowResized {
            width_px: 20_000,
            height_px: 264,
        });
        assert_eq!(
            controller.frame().layout().size().width().px(),
            Length::MAX_PX
        );
    }

    #[test]
    fn hit_regions_come_from_the_frame() {
        let controller = RulerController::new(&screen(), Transparency::Composited);
        assert_eq!(controller.frame().hit_regions().len(), 9);
    }
}
