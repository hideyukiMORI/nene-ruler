//! 物差しの状態の唯一の所有者（ARC-004）。

use nene_ruler_core::axis::Axis;
use nene_ruler_core::frame_layout::FrameLayout;
use nene_ruler_core::frame_size::FrameSize;
use nene_ruler_core::length::Length;
use nene_ruler_core::length_text::LengthText;
use nene_ruler_core::ruler_settings::RulerSettings;
use nene_ruler_core::screen_metrics::ScreenMetrics;
use nene_ruler_core::screen_metrics_failure::ScreenMetricsFailure;
use nene_ruler_core::unit::Unit;

use crate::frame_texts::FrameTexts;
use crate::modal_session::ModalSession;
use crate::product_version::ProductVersion;
use crate::ruler_frame::RulerFrame;
use crate::ruler_intent::RulerIntent;
use crate::screen_metrics_port::ScreenMetricsPort;
use crate::settings_frame::SettingsFrame;
use crate::transparency::Transparency;

/// 枠・設定・モーダルの編集状態・物理寸法・透明を 1 つ持ち、意図で次の状態へ置き換える。
///
/// 公開値はすべて不変で、変わるのは `apply` の中だけ（ARC-005）。
#[derive(Debug)]
pub struct RulerController {
    size: FrameSize,
    settings: RulerSettings,
    modal: Option<ModalSession>,
    metrics: Result<ScreenMetrics, ScreenMetricsFailure>,
    transparency: Transparency,
    version: ProductVersion,
}

impl RulerController {
    /// 初期の枠と設定で始める。物理寸法はここで一度だけ取り込む（ARC-004 の表）。
    pub fn new(
        metrics_port: &dyn ScreenMetricsPort,
        transparency: Transparency,
        version: ProductVersion,
    ) -> Self {
        Self {
            size: FrameSize::INITIAL,
            settings: RulerSettings::INITIAL,
            modal: None,
            metrics: metrics_port.screen_metrics(),
            transparency,
            version,
        }
    }

    /// 意図を反映して次の状態にする。
    pub fn apply(&mut self, intent: RulerIntent) {
        match intent {
            RulerIntent::WindowResized {
                width_px,
                height_px,
            } => self.resize(width_px, height_px),
            RulerIntent::OpenSettings => self.modal = Some(ModalSession::open(self.settings.ink())),
            RulerIntent::CloseSettings => self.modal = None,
            RulerIntent::ChooseInk(ink) => {
                self.settings = self.settings.with_ink(ink);
                self.modal = Some(ModalSession::open(ink));
            }
            RulerIntent::ChooseUnit(unit) => self.settings = self.settings.with_unit(unit),
            RulerIntent::ChooseModalTheme(theme) => {
                self.settings = self.settings.with_modal_theme(theme);
            }
            RulerIntent::ChooseLayer(layer) => self.settings = self.settings.with_layer(layer),
            RulerIntent::HexTyped(character) => {
                self.edit(|session| session.clone().with_entry(session.entry().typed(character)));
            }
            RulerIntent::HexBackspace => {
                self.edit(|session| session.clone().with_entry(session.entry().backspaced()));
            }
            RulerIntent::HexSubmit => self.submit_hex(),
        }
    }

    /// いまの状態から表示フレームを作る。
    #[must_use]
    pub fn frame(&self) -> RulerFrame {
        RulerFrame::new(
            FrameLayout::new(self.size),
            FrameTexts::new(self.text(Axis::Horizontal), self.text(Axis::Vertical)),
            (self.settings.ink(), self.settings.layer()),
            self.transparency,
        )
    }

    /// モーダルが開いていれば、それを描くための値。
    #[must_use]
    pub fn settings_frame(&self) -> Option<SettingsFrame> {
        self.modal
            .as_ref()
            .map(|session| SettingsFrame::new(self.settings, session.clone(), self.version))
    }

    /// 物理寸法が取れなかった理由。利用者に示すため（FR-014）。
    #[must_use]
    pub fn metrics_failure(&self) -> Option<ScreenMetricsFailure> {
        self.metrics.err()
    }

    /// 窓の大きさを枠の大きさに写す。
    fn resize(&mut self, width_px: u16, height_px: u16) {
        self.size = FrameSize::new(
            self.clamp(FrameLayout::frame_px_for_window(width_px), Axis::Horizontal),
            self.clamp(FrameLayout::frame_px_for_window(height_px), Axis::Vertical),
        );
    }

    /// モーダルが開いているときだけ編集状態を置き換える。
    fn edit(&mut self, change: impl FnOnce(&ModalSession) -> ModalSession) {
        if let Some(session) = &self.modal {
            self.modal = Some(change(session));
        }
    }

    /// 入力欄の文字列を色にする。読めなければ拒否を見せる。
    fn submit_hex(&mut self) {
        let Some(session) = &self.modal else {
            return;
        };
        match session.entry().parse() {
            Ok(ink) => {
                self.settings = self.settings.with_ink(ink);
                self.modal = Some(ModalSession::open(ink));
            }
            Err(failure) => self.modal = Some(session.clone().rejected(failure)),
        }
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
        match (self.settings.unit(), self.metrics) {
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
    use nene_ruler_core::modal_theme::ModalTheme;
    use nene_ruler_core::rgb::Rgb;
    use nene_ruler_core::rgb_failure::RgbFailure;
    use nene_ruler_core::window_layer::WindowLayer;

    struct FixedScreen(Result<ScreenMetrics, ScreenMetricsFailure>);

    impl ScreenMetricsPort for FixedScreen {
        fn screen_metrics(&self) -> Result<ScreenMetrics, ScreenMetricsFailure> {
            self.0
        }
    }

    fn screen() -> FixedScreen {
        FixedScreen(ScreenMetrics::new(1920, 1080, 480, 270))
    }

    fn controller() -> RulerController {
        RulerController::new(
            &screen(),
            Transparency::Composited,
            ProductVersion::new("0.1.0"),
        )
    }

    #[test]
    fn starts_with_the_initial_frame_in_pixels() {
        let controller = controller();
        let frame = controller.frame();
        assert_eq!(frame.text(Axis::Horizontal).as_str(), "400 px");
        assert_eq!(frame.text(Axis::Vertical).as_str(), "240 px");
        assert_eq!(frame.ink(), Rgb::INITIAL_INK);
        assert_eq!(frame.layer(), WindowLayer::AlwaysOnTop);
        assert_eq!(frame.transparency(), Transparency::Composited);
        assert_eq!(controller.metrics_failure(), None);
        assert!(controller.settings_frame().is_none());
    }

    #[test]
    fn window_size_becomes_frame_size_within_the_screen() {
        let mut controller = controller();
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
    fn without_metrics_the_limit_is_the_absolute_maximum_and_units_fall_back() {
        let port = FixedScreen(Err(ScreenMetricsFailure::Unavailable));
        let mut controller = RulerController::new(
            &port,
            Transparency::Composited,
            ProductVersion::new("0.1.0"),
        );
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
        controller.apply(RulerIntent::ChooseUnit(Unit::Mm));
        assert!(
            controller
                .frame()
                .text(Axis::Vertical)
                .as_str()
                .ends_with("px")
        );
    }

    #[test]
    fn units_convert_through_the_metrics() {
        let mut controller = controller();
        controller.apply(RulerIntent::ChooseUnit(Unit::Mm));
        assert_eq!(
            controller.frame().text(Axis::Horizontal).as_str(),
            "100.0 mm"
        );
        controller.apply(RulerIntent::ChooseUnit(Unit::In));
        assert_eq!(controller.frame().text(Axis::Vertical).as_str(), "2.36 in");
    }

    #[test]
    fn the_modal_opens_with_the_current_ink_and_accepts_presets() {
        let mut controller = controller();
        controller.apply(RulerIntent::OpenSettings);
        let frame = controller.settings_frame().unwrap();
        assert_eq!(frame.session().entry().text(), "#E53935");
        assert!(frame.footer().contains("0.1.0"));
        controller.apply(RulerIntent::ChooseInk(Rgb::new(1, 2, 3)));
        assert_eq!(controller.frame().ink(), Rgb::new(1, 2, 3));
        assert_eq!(
            controller
                .settings_frame()
                .unwrap()
                .session()
                .entry()
                .text(),
            "#010203"
        );
        controller.apply(RulerIntent::CloseSettings);
        assert!(controller.settings_frame().is_none());
    }

    #[test]
    fn typing_a_hex_colour_applies_on_submit_and_rejection_is_visible() {
        let mut controller = controller();
        controller.apply(RulerIntent::OpenSettings);
        for _ in 0_u8..7 {
            controller.apply(RulerIntent::HexBackspace);
        }
        for character in "#00ff00".chars() {
            controller.apply(RulerIntent::HexTyped(character));
        }
        controller.apply(RulerIntent::HexSubmit);
        assert_eq!(controller.frame().ink(), Rgb::new(0, 0xFF, 0));
        controller.apply(RulerIntent::HexBackspace);
        controller.apply(RulerIntent::HexSubmit);
        let session = controller.settings_frame().unwrap().session().clone();
        assert_eq!(session.rejection(), Some(RgbFailure::WrongLength));
        assert_eq!(controller.frame().ink(), Rgb::new(0, 0xFF, 0));
        controller.apply(RulerIntent::HexTyped('0'));
        assert_eq!(
            controller.settings_frame().unwrap().session().rejection(),
            None
        );
    }

    #[test]
    fn typing_without_the_modal_does_nothing() {
        let mut controller = controller();
        controller.apply(RulerIntent::HexTyped('1'));
        controller.apply(RulerIntent::HexSubmit);
        assert!(controller.settings_frame().is_none());
        assert_eq!(controller.frame().ink(), Rgb::INITIAL_INK);
    }

    #[test]
    fn theme_and_layer_are_recorded() {
        let mut controller = controller();
        controller.apply(RulerIntent::OpenSettings);
        controller.apply(RulerIntent::ChooseModalTheme(ModalTheme::Light));
        controller.apply(RulerIntent::ChooseLayer(WindowLayer::Normal));
        let frame = controller.settings_frame().unwrap();
        assert_eq!(frame.settings().modal_theme(), ModalTheme::Light);
        assert_eq!(
            frame.palette(),
            nene_ruler_core::modal_palette::ModalPalette::of(ModalTheme::Light)
        );
        assert_eq!(controller.frame().layer(), WindowLayer::Normal);
    }

    #[test]
    fn hit_regions_come_from_the_frame() {
        assert_eq!(controller().frame().hit_regions().len(), 9);
    }
}
