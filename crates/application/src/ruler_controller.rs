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

use crate::copy_mark::CopyMark;
use crate::frame_notices::FrameNotices;
use crate::frame_style::FrameStyle;
use crate::frame_texts::FrameTexts;
use crate::modal_session::ModalSession;
use crate::ports::Ports;
use crate::product_version::ProductVersion;
use crate::ruler_frame::RulerFrame;
use crate::ruler_intent::RulerIntent;
use crate::settings_frame::SettingsFrame;
use crate::settings_notices::SettingsNotices;
use crate::settings_store_failure::SettingsStoreFailure;
use crate::transparency::Transparency;

/// 枠・設定・モーダルの編集状態・物理寸法・透明を 1 つ持ち、意図で次の状態へ置き換える。
///
/// 公開値はすべて不変で、変わるのは `apply` の中だけ（ARC-005）。設定が変わるたびに保存ポートへ書く（FR-012）。
#[derive(Debug)]
pub struct RulerController<'p> {
    ports: Ports<'p>,
    size: FrameSize,
    settings: RulerSettings,
    modal: Option<ModalSession>,
    metrics: Result<ScreenMetrics, ScreenMetricsFailure>,
    store_failure: Option<SettingsStoreFailure>,
    copy_mark: Option<CopyMark>,
    transparency: Transparency,
    version: ProductVersion,
}

impl<'p> RulerController<'p> {
    /// 初期の枠で始める。物理寸法と保存された設定はここで一度だけ取り込む（ARC-004 の表）。
    /// 設定が読めなければ既定値で始め、理由を持っておく（ARC-009: 黙って落とさない）。
    #[must_use]
    pub fn new(ports: Ports<'p>, transparency: Transparency, version: ProductVersion) -> Self {
        let (settings, store_failure) = match ports.store().load() {
            Ok(Some(saved)) => (saved, None),
            Ok(None) => (RulerSettings::INITIAL, None),
            Err(failure) => (RulerSettings::INITIAL, Some(failure)),
        };
        Self {
            ports,
            size: FrameSize::INITIAL,
            settings,
            modal: None,
            metrics: ports.metrics().screen_metrics(),
            store_failure,
            copy_mark: None,
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
                self.change(self.settings.with_ink(ink));
                self.modal = Some(ModalSession::open(ink));
            }
            RulerIntent::ChooseUnit(unit) => self.change(self.settings.with_unit(unit)),
            RulerIntent::ChooseModalTheme(theme) => {
                self.change(self.settings.with_modal_theme(theme));
            }
            RulerIntent::ChooseLayer(layer) => self.change(self.settings.with_layer(layer)),
            RulerIntent::HexTyped(character) => {
                self.edit(|session| session.clone().with_entry(session.entry().typed(character)));
            }
            RulerIntent::HexBackspace => {
                self.edit(|session| session.clone().with_entry(session.entry().backspaced()));
            }
            RulerIntent::HexSubmit => self.submit_hex(),
            RulerIntent::CopyText(axis) => {
                let outcome = self.ports.clipboard().copy(self.text(axis).as_str());
                self.copy_mark = Some(CopyMark::new(axis, outcome));
            }
            RulerIntent::ClearCopyMark => self.copy_mark = None,
        }
    }

    /// いまの状態から表示フレームを作る。
    #[must_use]
    pub fn frame(&self) -> RulerFrame {
        RulerFrame::new(
            FrameLayout::new(self.size),
            FrameTexts::new(self.text(Axis::Horizontal), self.text(Axis::Vertical)),
            FrameStyle::new(
                self.settings.ink(),
                self.settings.layer(),
                self.transparency,
                self.copy_mark,
            )
            .with_notices(FrameNotices::new(
                self.transparency == Transparency::Opaque,
                self.metrics.is_err() && self.settings.unit() != Unit::Px,
            )),
        )
    }

    /// モーダルが開いていれば、それを描くための値。
    #[must_use]
    pub fn settings_frame(&self) -> Option<SettingsFrame> {
        self.modal.as_ref().map(|session| {
            SettingsFrame::new(
                self.settings,
                session.clone(),
                self.version,
                SettingsNotices::new(self.store_failure, self.metrics.err()),
            )
        })
    }

    /// 物理寸法が取れなかった理由。利用者に示すため（FR-014）。
    #[must_use]
    pub fn metrics_failure(&self) -> Option<ScreenMetricsFailure> {
        self.metrics.err()
    }

    /// 設定の読み書きが失敗していれば、その理由（FR-014）。
    #[must_use]
    pub const fn store_failure(&self) -> Option<SettingsStoreFailure> {
        self.store_failure
    }

    /// 設定を置き換えて保存する。保存の失敗は持っておき、成功すれば消す。
    fn change(&mut self, settings: RulerSettings) {
        self.settings = settings;
        self.store_failure = self.ports.store().save(settings).err();
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
                self.change(self.settings.with_ink(ink));
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
    use crate::clipboard_failure::ClipboardFailure;
    use crate::clipboard_port::ClipboardPort;
    use crate::screen_metrics_port::ScreenMetricsPort;
    use crate::settings_store_port::SettingsStorePort;
    use alloc::borrow::ToOwned as _;
    use alloc::string::String;
    use core::cell::RefCell;
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

    /// 読み込みの結果を固定し、保存された値を覚える偽の保存先。
    struct MemoryStore {
        loaded: Result<Option<RulerSettings>, SettingsStoreFailure>,
        saved: RefCell<Option<RulerSettings>>,
        save_failure: Option<SettingsStoreFailure>,
    }

    impl MemoryStore {
        fn empty() -> Self {
            Self {
                loaded: Ok(None),
                saved: RefCell::new(None),
                save_failure: None,
            }
        }
    }

    impl SettingsStorePort for MemoryStore {
        fn load(&self) -> Result<Option<RulerSettings>, SettingsStoreFailure> {
            self.loaded
        }

        fn save(&self, settings: RulerSettings) -> Result<(), SettingsStoreFailure> {
            *self.saved.borrow_mut() = Some(settings);
            self.save_failure.map_or(Ok(()), Err)
        }
    }

    /// 置かれた文字列を覚える偽のクリップボード。
    struct MemoryClipboard {
        copied: RefCell<Option<String>>,
        failure: Option<ClipboardFailure>,
    }

    impl ClipboardPort for MemoryClipboard {
        fn copy(&self, text: &str) -> Result<(), ClipboardFailure> {
            *self.copied.borrow_mut() = Some(text.to_owned());
            self.failure.map_or(Ok(()), Err)
        }
    }

    fn clipboard() -> MemoryClipboard {
        MemoryClipboard {
            copied: RefCell::new(None),
            failure: None,
        }
    }

    fn screen() -> FixedScreen {
        FixedScreen(ScreenMetrics::new(1920, 1080, 480, 270))
    }

    fn controller<'p>(screen: &'p FixedScreen, store: &'p MemoryStore) -> RulerController<'p> {
        static NO_CLIPBOARD: NoClipboard = NoClipboard;
        RulerController::new(
            Ports::new(screen, store, &NO_CLIPBOARD),
            Transparency::Composited,
            ProductVersion::new("0.1.0"),
        )
    }

    /// 何もしないクリップボード。コピーを見ないテストで使う。
    struct NoClipboard;

    impl ClipboardPort for NoClipboard {
        fn copy(&self, _text: &str) -> Result<(), ClipboardFailure> {
            Ok(())
        }
    }

    #[test]
    fn starts_with_the_initial_frame_in_pixels() {
        let (screen, store) = (screen(), MemoryStore::empty());
        let controller = controller(&screen, &store);
        let frame = controller.frame();
        assert_eq!(frame.text(Axis::Horizontal).as_str(), "400 px");
        assert_eq!(frame.text(Axis::Vertical).as_str(), "240 px");
        assert_eq!(frame.ink(), Rgb::INITIAL_INK);
        assert_eq!(frame.layer(), WindowLayer::AlwaysOnTop);
        assert_eq!(frame.transparency(), Transparency::Composited);
        assert_eq!(controller.metrics_failure(), None);
        assert_eq!(controller.store_failure(), None);
        assert!(controller.settings_frame().is_none());
        assert_eq!(*store.saved.borrow(), None);
    }

    #[test]
    fn window_size_becomes_frame_size_within_the_screen() {
        let (screen, store) = (screen(), MemoryStore::empty());
        let mut controller = controller(&screen, &store);
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
        let screen = FixedScreen(Err(ScreenMetricsFailure::Unavailable));
        let store = MemoryStore::empty();
        let mut controller = controller(&screen, &store);
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
        let (screen, store) = (screen(), MemoryStore::empty());
        let mut controller = controller(&screen, &store);
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
        let (screen, store) = (screen(), MemoryStore::empty());
        let mut controller = controller(&screen, &store);
        controller.apply(RulerIntent::OpenSettings);
        let frame = controller.settings_frame().unwrap();
        assert_eq!(frame.session().entry().text(), "#E53935");
        assert!(frame.footer().contains("0.1.0"));
        assert_eq!(frame.store_failure(), None);
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
        let (screen, store) = (screen(), MemoryStore::empty());
        let mut controller = controller(&screen, &store);
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
        let (screen, store) = (screen(), MemoryStore::empty());
        let mut controller = controller(&screen, &store);
        controller.apply(RulerIntent::HexTyped('1'));
        controller.apply(RulerIntent::HexSubmit);
        assert!(controller.settings_frame().is_none());
        assert_eq!(controller.frame().ink(), Rgb::INITIAL_INK);
    }

    #[test]
    fn theme_and_layer_are_recorded() {
        let (screen, store) = (screen(), MemoryStore::empty());
        let mut controller = controller(&screen, &store);
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
        let (screen, store) = (screen(), MemoryStore::empty());
        assert_eq!(controller(&screen, &store).frame().hit_regions().len(), 9);
    }

    #[test]
    fn saved_settings_are_loaded_and_every_change_is_saved() {
        let saved = RulerSettings::INITIAL
            .with_ink(Rgb::new(9, 8, 7))
            .with_unit(Unit::In);
        let screen = screen();
        let store = MemoryStore {
            loaded: Ok(Some(saved)),
            saved: RefCell::new(None),
            save_failure: None,
        };
        let mut controller = controller(&screen, &store);
        assert_eq!(controller.frame().ink(), Rgb::new(9, 8, 7));
        assert_eq!(
            controller.frame().text(Axis::Horizontal).as_str(),
            "3.94 in"
        );
        controller.apply(RulerIntent::ChooseLayer(WindowLayer::Normal));
        assert_eq!(
            *store.saved.borrow(),
            Some(saved.with_layer(WindowLayer::Normal))
        );
        assert_eq!(controller.store_failure(), None);
    }

    #[test]
    fn an_unreadable_store_starts_with_defaults_and_is_visible() {
        let screen = screen();
        let store = MemoryStore {
            loaded: Err(SettingsStoreFailure::UnknownVersion),
            saved: RefCell::new(None),
            save_failure: None,
        };
        let mut controller = controller(&screen, &store);
        assert_eq!(controller.frame().ink(), Rgb::INITIAL_INK);
        assert_eq!(
            controller.store_failure(),
            Some(SettingsStoreFailure::UnknownVersion)
        );
        controller.apply(RulerIntent::OpenSettings);
        assert_eq!(
            controller.settings_frame().unwrap().store_failure(),
            Some(SettingsStoreFailure::UnknownVersion)
        );
        // 保存が成功すれば、読めなかった理由は役目を終える。
        controller.apply(RulerIntent::ChooseUnit(Unit::Mm));
        assert_eq!(controller.store_failure(), None);
    }

    #[test]
    fn a_failed_save_is_visible_but_the_change_stays() {
        let screen = screen();
        let store = MemoryStore {
            loaded: Ok(None),
            saved: RefCell::new(None),
            save_failure: Some(SettingsStoreFailure::Unwritable),
        };
        let mut controller = controller(&screen, &store);
        controller.apply(RulerIntent::ChooseInk(Rgb::new(1, 1, 1)));
        assert_eq!(controller.frame().ink(), Rgb::new(1, 1, 1));
        assert_eq!(
            controller.store_failure(),
            Some(SettingsStoreFailure::Unwritable)
        );
    }

    #[test]
    fn copying_sends_the_displayed_string_and_marks_the_number() {
        let (screen, store, board) = (screen(), MemoryStore::empty(), clipboard());
        let mut controller = RulerController::new(
            Ports::new(&screen, &store, &board),
            Transparency::Composited,
            ProductVersion::new("0.1.0"),
        );
        controller.apply(RulerIntent::ChooseUnit(Unit::Mm));
        controller.apply(RulerIntent::CopyText(Axis::Vertical));
        assert_eq!(board.copied.borrow().as_deref(), Some("60.0 mm"));
        let mark = controller.frame().copy_mark().unwrap();
        assert_eq!(mark.axis(), Axis::Vertical);
        assert!(mark.succeeded());
        controller.apply(RulerIntent::ClearCopyMark);
        assert!(controller.frame().copy_mark().is_none());
    }

    #[test]
    fn a_failed_copy_is_marked_as_such() {
        let screen = screen();
        let store = MemoryStore::empty();
        let board = MemoryClipboard {
            copied: RefCell::new(None),
            failure: Some(ClipboardFailure::Rejected),
        };
        let mut controller = RulerController::new(
            Ports::new(&screen, &store, &board),
            Transparency::Composited,
            ProductVersion::new("0.1.0"),
        );
        controller.apply(RulerIntent::CopyText(Axis::Horizontal));
        let mark = controller.frame().copy_mark().unwrap();
        assert!(!mark.succeeded());
        assert_eq!(mark.failure(), Some(ClipboardFailure::Rejected));
    }

    #[test]
    fn notices_follow_transparency_and_metrics() {
        use crate::frame_notice::FrameNotice;
        static NO_CLIPBOARD: NoClipboard = NoClipboard;
        let screen = FixedScreen(Err(ScreenMetricsFailure::Unavailable));
        let store = MemoryStore::empty();
        let mut controller = RulerController::new(
            Ports::new(&screen, &store, &NO_CLIPBOARD),
            Transparency::Opaque,
            ProductVersion::new("0.1.0"),
        );
        assert_eq!(controller.frame().notices().list(), [FrameNotice::Opaque]);
        controller.apply(RulerIntent::ChooseUnit(Unit::Mm));
        assert_eq!(
            controller.frame().notices().list(),
            [FrameNotice::Opaque, FrameNotice::NoScreenMetrics]
        );
        controller.apply(RulerIntent::OpenSettings);
        assert_eq!(
            controller.settings_frame().unwrap().metrics_failure(),
            Some(ScreenMetricsFailure::Unavailable)
        );
        let (real_screen, fresh_store) = (self::screen(), MemoryStore::empty());
        let fine = self::controller(&real_screen, &fresh_store);
        assert!(fine.frame().notices().list().is_empty());
    }
}
