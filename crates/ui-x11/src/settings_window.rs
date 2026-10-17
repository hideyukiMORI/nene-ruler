//! 設定モーダル。2 枚目の窓（FR-009）。不透明で、枠の窓の transient。

use nene_ruler_application::ruler_controller::RulerController;
use nene_ruler_application::ruler_intent::RulerIntent;
use nene_ruler_application::settings_frame::SettingsFrame;
use nene_ruler_core::hit_target::HitTarget;
use nene_ruler_core::pixel_point::PixelPoint;
use nene_ruler_core::rgb::Rgb;
use nene_ruler_core::settings_control::SettingsControl;
use nene_ruler_core::settings_hit;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    AtomEnum, ButtonPressEvent, ColormapAlloc, ColormapWrapper, ConfigureWindowAux,
    ConnectionExt as _, CreateGCAux, CreateWindowAux, EventMask, GcontextWrapper, InputFocus,
    KeyButMask, PropMode, Window, WindowClass, WindowWrapper,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::alt_f4_gesture::AltF4Gesture;
use crate::display::Display;
use crate::drag::Drag;
use crate::field_focus::FieldFocus;
use crate::key_input::KeyInput;
use crate::settings_canvas::render_settings;
use crate::settings_flow::SettingsFlow;
use crate::transfer::Transfer;
use crate::ui_failure::UiFailure;
use crate::undecorated::{self, MOVERESIZE_MOVE};

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
    alt_f4: AltF4Gesture,
    wm_moves: bool,
    drag: Option<Drag>,
    focus: FieldFocus,
    picking: Option<SettingsControl>,
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
        let bounds = frame.layout().window();
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
                    | EventMask::BUTTON_RELEASE
                    | EventMask::POINTER_MOTION
                    | EventMask::FOCUS_CHANGE
                    | EventMask::KEY_PRESS
                    | EventMask::KEY_RELEASE,
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
            alt_f4: AltF4Gesture::IDLE,
            wm_moves: display.wm_supports(display.atoms().net_wm_moveresize),
            drag: None,
            focus: FieldFocus::Unfocused,
            picking: None,
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
        let mut canvas = render_settings(frame, self.display.visual().format(), self.focus)?;
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
        &mut self,
        controller: &mut RulerController<'_>,
        event: &Event,
    ) -> Result<SettingsFlow, UiFailure> {
        if let Event::Expose(expose) = event {
            if expose.count == 0 {
                self.expose(controller)?;
            }
        } else if let Event::ButtonPress(press) = event {
            return self.pressed(controller, press);
        } else if let Event::MotionNotify(motion) = event {
            self.dragged((motion.root_x, motion.root_y))?;
            if let Some(control) = self.picking
                && let Some(intent) =
                    Self::pick(controller, control, (motion.event_x, motion.event_y))
            {
                controller.apply(intent);
                return Ok(SettingsFlow::Changed);
            }
        } else if let Event::ButtonRelease(_) = event {
            self.drag = None;
            self.picking = None;
        } else if let Event::FocusIn(_) = event {
            self.refocused(controller, FieldFocus::Focused)?;
        } else if let Event::FocusOut(_) = event {
            self.refocused(controller, FieldFocus::Unfocused)?;
        } else if let Event::KeyPress(key) = event {
            let input = self.display.key_input(key);
            if self
                .alt_f4
                .pressed(input, key.state.contains(KeyButMask::MOD1))
            {
                return Ok(SettingsFlow::Close);
            }
            return Ok(Self::typed(controller, input));
        } else if let Event::KeyRelease(key) = event {
            // WSLg では Alt+F4 の押下が届かず、F4 の離しだけが届く（ADR 0009）。
            let input = self.display.key_input(key);
            if self
                .alt_f4
                .released(input, key.state.contains(KeyButMask::MOD1))
            {
                return Ok(SettingsFlow::Close);
            }
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

    /// 焦点が出入りした。入力欄の棒と縁だけが変わるので、モーダルだけ描き直す（Issue #40）。
    fn refocused(
        &mut self,
        controller: &RulerController<'_>,
        focus: FieldFocus,
    ) -> Result<(), UiFailure> {
        if self.focus == focus {
            return Ok(());
        }
        self.focus = focus;
        self.expose(controller)
    }

    /// 露出で描き直す。
    fn expose(&self, controller: &RulerController<'_>) -> Result<(), UiFailure> {
        match controller.settings_frame() {
            Some(frame) => self.present(&frame),
            None => Ok(()),
        }
    }

    /// 左ボタンで押した部品を意図にする。部品の無い所を掴むと窓を動かす（ADR 0014）。
    fn pressed(
        &mut self,
        controller: &mut RulerController<'_>,
        press: &ButtonPressEvent,
    ) -> Result<SettingsFlow, UiFailure> {
        if press.detail != BUTTON_LEFT {
            return Ok(SettingsFlow::Continue);
        }
        // どこを押しても、打った鍵が入力欄に入るよう焦点を取り直す（Issue #40）。
        self.display.connection().set_input_focus(
            InputFocus::PARENT,
            self.window.window(),
            press.time,
        )?;
        self.display.connection().flush()?;
        let Some(point) = PixelPoint::from_event(press.event_x, press.event_y) else {
            return Ok(SettingsFlow::Continue);
        };
        let current_layer = controller.frame().layer();
        let Some(layout) = controller.settings_frame().map(|frame| frame.layout()) else {
            return Ok(SettingsFlow::Continue);
        };
        let Some(control) = settings_hit::control_at(&layout.hits(current_layer), point) else {
            self.start_move(press)?;
            return Ok(SettingsFlow::Continue);
        };
        let intent = match control {
            SettingsControl::Preset(index) => {
                let Some(preset) = Rgb::PRESETS.get(usize::from(index)) else {
                    return Ok(SettingsFlow::Continue);
                };
                RulerIntent::ChooseInk(*preset)
            }
            // 入力欄は常に鍵を受けるので、押しても状態は変わらない。
            SettingsControl::HexField => return Ok(SettingsFlow::Continue),
            SettingsControl::Shade | SettingsControl::Hue => {
                // 押したまま引くと追いかける（ADR 0016）。
                self.picking = Some(control);
                let Some(intent) = Self::pick(controller, control, (press.event_x, press.event_y))
                else {
                    return Ok(SettingsFlow::Continue);
                };
                intent
            }
            SettingsControl::Theme(theme) => RulerIntent::ChooseModalTheme(theme),
            SettingsControl::Layer(layer) => RulerIntent::ChooseLayer(layer),
            SettingsControl::Close => return Ok(SettingsFlow::Close),
            SettingsControl::Quit => return Ok(SettingsFlow::Quit),
        };
        controller.apply(intent);
        Ok(SettingsFlow::Changed)
    }

    /// ピッカーの位置を意図にする。窓の外へ引いた位置は縁へ寄せる。
    fn pick(
        controller: &RulerController<'_>,
        control: SettingsControl,
        (x, y): (i16, i16),
    ) -> Option<RulerIntent> {
        let point = PixelPoint::from_event(x.max(0), y.max(0))?;
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

    /// 窓を動かし始める。WM に任せ、できなければ自前で引く（枠の窓と同じ・ADR 0005 の 4）。
    fn start_move(&mut self, press: &ButtonPressEvent) -> Result<(), UiFailure> {
        let window = self.window.window();
        if self.wm_moves {
            undecorated::ask_wm_to_move_resize(self.display, window, press, MOVERESIZE_MOVE)
        } else {
            let start = undecorated::geometry_of(self.display, window)?;
            self.drag = Some(Drag::begin(
                HitTarget::Move,
                (press.root_x, press.root_y),
                start,
            ));
            Ok(())
        }
    }

    /// 自前のドラッグ中なら窓を動かす。
    fn dragged(&self, root: (i16, i16)) -> Result<(), UiFailure> {
        let Some(drag) = self.drag else {
            return Ok(());
        };
        let geometry = drag.geometry_at(root, 0);
        self.display.connection().configure_window(
            self.window.window(),
            &ConfigureWindowAux::new().x(geometry.x()).y(geometry.y()),
        )?;
        self.display.connection().flush()?;
        Ok(())
    }

    /// 鍵を入力欄の意図にする。Esc は閉じる。
    fn typed(controller: &mut RulerController<'_>, input: KeyInput) -> SettingsFlow {
        let intent = match input {
            KeyInput::Escape => return SettingsFlow::Close,
            KeyInput::Return => RulerIntent::HexSubmit,
            KeyInput::Backspace => RulerIntent::HexBackspace,
            KeyInput::Character(character) => RulerIntent::HexTyped(character),
            KeyInput::F4 | KeyInput::Alt | KeyInput::Arrow(_) | KeyInput::Other => {
                return SettingsFlow::Continue;
            }
        };
        controller.apply(intent);
        SettingsFlow::Changed
    }

    /// transient・飾りなし・閉じる要求・題を WM へ伝える。閉じるボタンと移動は自前（ADR 0014）。
    fn declare_hints(&self, owner: Window) -> Result<(), UiFailure> {
        let connection = self.display.connection();
        let atoms = self.display.atoms();
        let window = self.window.window();
        undecorated::declare_undecorated(self.display, window)?;
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
