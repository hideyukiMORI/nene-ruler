//! 設定モーダル。2 枚目の窓（FR-009）。不透明で、枠の窓の transient。

use nene_ruler_application::ruler_controller::RulerController;
use nene_ruler_application::ruler_intent::RulerIntent;
use nene_ruler_application::settings_frame::SettingsFrame;
use nene_ruler_core::pixel_point::PixelPoint;
use nene_ruler_core::rgb::Rgb;
use nene_ruler_core::settings_control::SettingsControl;
use nene_ruler_core::settings_hit;
use nene_ruler_core::settings_layout::SettingsLayout;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    AtomEnum, ButtonPressEvent, ColormapAlloc, ColormapWrapper, ConnectionExt as _, CreateGCAux,
    CreateWindowAux, EventMask, GcontextWrapper, InputFocus, PropMode, Window, WindowClass,
    WindowWrapper,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::display::Display;
use crate::key_input::KeyInput;
use crate::settings_canvas::render_settings;
use crate::settings_flow::SettingsFlow;
use crate::transfer::Transfer;
use crate::ui_failure::UiFailure;

/// 左ボタン。
const BUTTON_LEFT: u8 = 1;
/// 窓の題。
const TITLE: &str = "NeNe Ruler Settings";

/// モーダルの窓とその OS 資源。`Drop` で閉じる。
#[derive(Debug)]
pub struct SettingsWindow<'d> {
    display: &'d Display,
    window: WindowWrapper<&'d RustConnection>,
    gc: GcontextWrapper<&'d RustConnection>,
    _colormap: ColormapWrapper<&'d RustConnection>,
}

impl<'d> SettingsWindow<'d> {
    /// 枠の窓の transient として作り、最初のフレームを描いて表示し、キー入力の焦点を取る。
    ///
    /// # Errors
    ///
    /// X の要求か cairo が失敗した。
    pub fn open(
        display: &'d Display,
        owner: Window,
        frame: &SettingsFrame,
    ) -> Result<Self, UiFailure> {
        let connection = display.connection();
        let root = display.screen()?.root;
        let visual = display.visual();
        let bounds = SettingsLayout::window();
        let colormap = ColormapWrapper::create_colormap(
            connection,
            ColormapAlloc::NONE,
            root,
            visual.visual(),
        )?;
        let attributes = CreateWindowAux::new()
            .background_pixel(0)
            .border_pixel(0)
            .colormap(colormap.colormap())
            .event_mask(
                EventMask::EXPOSURE
                    | EventMask::STRUCTURE_NOTIFY
                    | EventMask::BUTTON_PRESS
                    | EventMask::KEY_PRESS,
            );
        let window = WindowWrapper::create_window(
            connection,
            visual.depth(),
            root,
            0,
            0,
            bounds.width(),
            bounds.height(),
            0,
            WindowClass::INPUT_OUTPUT,
            visual.visual(),
            &attributes,
        )?;
        let gc = GcontextWrapper::create_gc(connection, window.window(), &CreateGCAux::new())?;
        let opened = Self {
            display,
            window,
            gc,
            _colormap: colormap,
        };
        opened.declare_hints(owner)?;
        connection.map_window(opened.window.window())?;
        opened.present(frame)?;
        connection.set_input_focus(
            InputFocus::PARENT,
            opened.window.window(),
            x11rb::CURRENT_TIME,
        )?;
        connection.flush()?;
        Ok(opened)
    }

    /// この窓の ID。イベントの振り分けに使う。
    #[must_use]
    pub fn window(&self) -> Window {
        self.window.window()
    }

    /// フレームを描き切って転送する。
    ///
    /// # Errors
    ///
    /// X の要求か cairo が失敗した。
    pub fn present(&self, frame: &SettingsFrame) -> Result<(), UiFailure> {
        let mut canvas = render_settings(frame, self.display.visual().format())?;
        Transfer::new(
            (self.window.window(), self.gc.gcontext()),
            canvas.width(),
            self.display.visual().depth(),
        )
        .present(self.display.connection(), &mut canvas)
    }

    /// この窓宛の 1 つのイベントを意図にする。
    ///
    /// # Errors
    ///
    /// 描き直しに失敗した。
    pub(crate) fn handle(
        &self,
        controller: &mut RulerController<'_>,
        event: &Event,
    ) -> Result<SettingsFlow, UiFailure> {
        if let Event::Expose(expose) = event {
            if expose.count == 0 {
                self.expose(controller)?;
            }
        } else if let Event::ButtonPress(press) = event {
            return Ok(Self::pressed(controller, press));
        } else if let Event::KeyPress(key) = event {
            return Ok(Self::typed(controller, self.display.key_input(key)));
        } else if let Event::ClientMessage(message) = event {
            let atoms = self.display.atoms();
            if message.type_ == atoms.wm_protocols
                && message.data.as_data32().first() == Some(&atoms.wm_delete_window)
            {
                return Ok(SettingsFlow::Close);
            }
        }
        Ok(SettingsFlow::Continue)
    }

    /// 露出で描き直す。
    fn expose(&self, controller: &RulerController<'_>) -> Result<(), UiFailure> {
        match controller.settings_frame() {
            Some(frame) => self.present(&frame),
            None => Ok(()),
        }
    }

    /// 左ボタンで押した部品を意図にする。
    fn pressed(controller: &mut RulerController<'_>, press: &ButtonPressEvent) -> SettingsFlow {
        if press.detail != BUTTON_LEFT {
            return SettingsFlow::Continue;
        }
        let Some(point) = PixelPoint::from_event(press.event_x, press.event_y) else {
            return SettingsFlow::Continue;
        };
        let current_layer = controller.frame().layer();
        let Some(control) = settings_hit::control_at(&SettingsLayout::hits(current_layer), point)
        else {
            return SettingsFlow::Continue;
        };
        let intent = match control {
            SettingsControl::Preset(index) => {
                let Some(preset) = Rgb::PRESETS.get(usize::from(index)) else {
                    return SettingsFlow::Continue;
                };
                RulerIntent::ChooseInk(*preset)
            }
            // 入力欄は常に鍵を受けるので、押しても状態は変わらない。
            SettingsControl::HexField => return SettingsFlow::Continue,
            SettingsControl::Unit(unit) => RulerIntent::ChooseUnit(unit),
            SettingsControl::Theme(theme) => RulerIntent::ChooseModalTheme(theme),
            SettingsControl::Layer(layer) => RulerIntent::ChooseLayer(layer),
        };
        controller.apply(intent);
        SettingsFlow::Changed
    }

    /// 鍵を入力欄の意図にする。Esc は閉じる。
    fn typed(controller: &mut RulerController<'_>, input: KeyInput) -> SettingsFlow {
        let intent = match input {
            KeyInput::Escape => return SettingsFlow::Close,
            KeyInput::Return => RulerIntent::HexSubmit,
            KeyInput::Backspace => RulerIntent::HexBackspace,
            KeyInput::Character(character) => RulerIntent::HexTyped(character),
            KeyInput::Arrow(_) | KeyInput::Other => return SettingsFlow::Continue,
        };
        controller.apply(intent);
        SettingsFlow::Changed
    }

    /// transient・閉じる要求・題を WM へ伝える。装飾は WM に任せる（枠の窓と違い、題を持つ）。
    fn declare_hints(&self, owner: Window) -> Result<(), UiFailure> {
        let connection = self.display.connection();
        let atoms = self.display.atoms();
        let window = self.window.window();
        connection.change_property32(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_TRANSIENT_FOR,
            AtomEnum::WINDOW,
            &[owner],
        )?;
        connection.change_property32(
            PropMode::REPLACE,
            window,
            atoms.wm_protocols,
            AtomEnum::ATOM,
            &[atoms.wm_delete_window],
        )?;
        connection.change_property8(
            PropMode::REPLACE,
            window,
            atoms.net_wm_name,
            atoms.utf8_string,
            TITLE.as_bytes(),
        )?;
        connection.change_property8(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            TITLE.as_bytes(),
        )?;
        Ok(())
    }
}
