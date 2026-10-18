//! 設定モーダルの窓 1 つ分の状態と、その窓宛のメッセージの処理（ADR 0014 / 0016 / 0021）。

use nene_ruler_application::ruler_controller::RulerController;
use nene_ruler_application::ruler_intent::RulerIntent;
use nene_ruler_core::pixel_point::PixelPoint;
use nene_ruler_core::rgb::Rgb;
use nene_ruler_core::settings_control::SettingsControl;
use nene_ruler_core::settings_hit;
use windows::Win32::Foundation::HWND;

use crate::modal_focus::ModalFocus;
use crate::modal_key::ModalKey;
use crate::modal_reply::ModalReply;
use crate::non_client_hit::NonClientHit;

/// モーダルの窓・焦点・ピッカーを押したまま引いているか。
#[derive(Debug)]
pub struct ModalWindow {
    window: HWND,
    focus: ModalFocus,
    picking: Option<SettingsControl>,
}

impl ModalWindow {
    /// 作った窓から。見せた直後なので焦点を持っている。
    #[must_use]
    pub const fn new(window: HWND) -> Self {
        Self {
            window,
            focus: ModalFocus::Focused,
            picking: None,
        }
    }

    /// 窓。
    #[must_use]
    pub const fn window(&self) -> HWND {
        self.window
    }

    /// 焦点。
    #[must_use]
    pub const fn focus(&self) -> ModalFocus {
        self.focus
    }

    /// 焦点が出入りした。
    pub const fn refocused(&mut self, focus: ModalFocus) {
        self.focus = focus;
    }

    /// 窓の中の点の当たり判定。部品の無い所は掴んで動かせる（`HTCAPTION`・ADR 0014）。
    #[must_use]
    pub fn hit(controller: &RulerController<'_>, client: (i32, i32)) -> NonClientHit {
        if Self::control_at(controller, client).is_some() {
            NonClientHit::Client
        } else {
            NonClientHit::Caption
        }
    }

    /// 左ボタンで部品を押した。
    pub fn pressed(
        &mut self,
        controller: &mut RulerController<'_>,
        client: (i32, i32),
    ) -> ModalReply {
        let Some(control) = Self::control_at(controller, client) else {
            return ModalReply::Unchanged;
        };
        let intent = match control {
            SettingsControl::Preset(index) => {
                let Some(preset) = Rgb::PRESETS.get(usize::from(index)) else {
                    return ModalReply::Unchanged;
                };
                RulerIntent::ChooseInk(*preset)
            }
            // 入力欄は焦点があれば鍵を受けるので、押しても状態は変わらない。
            SettingsControl::HexField => return ModalReply::Unchanged,
            SettingsControl::Shade | SettingsControl::Hue => {
                self.picking = Some(control);
                let Some(intent) = Self::pick(controller, control, client) else {
                    return ModalReply::Unchanged;
                };
                intent
            }
            SettingsControl::Theme(theme) => RulerIntent::ChooseModalTheme(theme),
            SettingsControl::Layer(layer) => {
                controller.apply(RulerIntent::ChooseLayer(layer));
                return ModalReply::LayerChanged;
            }
            SettingsControl::Close => return ModalReply::Close,
            SettingsControl::Quit => return ModalReply::Quit,
        };
        controller.apply(intent);
        ModalReply::Changed
    }

    /// ポインタが動いた。ピッカーを押したまま引いていれば追いかける（ADR 0016）。
    pub fn dragged(&self, controller: &mut RulerController<'_>, client: (i32, i32)) -> ModalReply {
        let Some(control) = self.picking else {
            return ModalReply::Unchanged;
        };
        match Self::pick(controller, control, client) {
            Some(intent) => {
                controller.apply(intent);
                ModalReply::Changed
            }
            None => ModalReply::Unchanged,
        }
    }

    /// 左ボタンを離した。引くのを終える。
    pub const fn released(&mut self) -> bool {
        let was_picking = self.picking.is_some();
        self.picking = None;
        was_picking
    }

    /// 字を打った（`WM_CHAR`）。制御文字は鍵の方で扱う。
    pub fn typed(controller: &mut RulerController<'_>, character: char) -> ModalReply {
        if character.is_control() {
            return ModalReply::Unchanged;
        }
        controller.apply(RulerIntent::HexTyped(character));
        ModalReply::Changed
    }

    /// Backspace・Enter・Esc（`WM_KEYDOWN`）。
    pub fn key(controller: &mut RulerController<'_>, key: ModalKey) -> ModalReply {
        let intent = match key {
            ModalKey::Escape => return ModalReply::Close,
            ModalKey::Backspace => RulerIntent::HexBackspace,
            ModalKey::Return => RulerIntent::HexSubmit,
        };
        controller.apply(intent);
        ModalReply::Changed
    }

    /// 窓の中の点にある部品。
    fn control_at(controller: &RulerController<'_>, (x, y): (i32, i32)) -> Option<SettingsControl> {
        let point = PixelPoint::new(u16::try_from(x).ok()?, u16::try_from(y).ok()?);
        let layout = controller.settings_frame()?.layout();
        settings_hit::control_at(&layout.hits(controller.frame().layer()), point)
    }

    /// ピッカーの位置を意図にする。窓の外へ引いた位置は縁へ寄せる。
    fn pick(
        controller: &RulerController<'_>,
        control: SettingsControl,
        (x, y): (i32, i32),
    ) -> Option<RulerIntent> {
        let clamp = |value: i32| u16::try_from(value.max(0_i32)).unwrap_or(u16::MAX);
        let point = PixelPoint::new(clamp(x), clamp(y));
        let layout = controller.settings_frame()?.layout();
        match control {
            SettingsControl::Shade => {
                let (saturation, value) = layout.shade_at(point);
                Some(RulerIntent::PickShade { saturation, value })
            }
            SettingsControl::Hue => Some(RulerIntent::PickHue(layout.hue_at(point))),
            SettingsControl::Preset(_)
            | SettingsControl::HexField
            | SettingsControl::Theme(_)
            | SettingsControl::Layer(_)
            | SettingsControl::Close
            | SettingsControl::Quit => None,
        }
    }
}
