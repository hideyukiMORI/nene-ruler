//! 枠の窓。作る・描いたものを転送する・当たり判定を X へ写す・イベントを意図にする。

use nene_ruler_application::ruler_controller::RulerController;
use nene_ruler_application::ruler_frame::RulerFrame;
use nene_ruler_application::ruler_intent::RulerIntent;
use nene_ruler_core::anchor::Anchor;
use nene_ruler_core::hit_region::{self, HitRegion};
use nene_ruler_core::hit_target::HitTarget;
use nene_ruler_core::length::Length;
use nene_ruler_core::nudge::Nudge;
use nene_ruler_core::pixel_point::PixelPoint;
use nene_ruler_core::pixel_rect::PixelRect;
use nene_ruler_core::window_layer::WindowLayer;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::shape;
use x11rb::protocol::xfixes::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{
    AtomEnum, ButtonPressEvent, ChangeWindowAttributesAux, ClientMessageEvent, ColormapAlloc,
    ColormapWrapper, ConfigureWindowAux, ConnectionExt as _, CreateGCAux, CreateWindowAux, Cursor,
    EventMask, GcontextWrapper, KeyButMask, PropMode, Rectangle, Window, WindowClass,
    WindowWrapper,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::alt_f4_gesture::AltF4Gesture;
use crate::canvas::Canvas;
use crate::display::Display;
use crate::drag::Drag;
use crate::flow::Flow;
use crate::key_input::KeyInput;
use crate::pointer_shapes::PointerShapes;
use crate::transfer::Transfer;
use crate::ui_failure::UiFailure;
use crate::undecorated::{
    self, MOVERESIZE_MOVE, MOVERESIZE_SIZE_BOTTOMRIGHT, MOVERESIZE_SIZE_TOPLEFT, SOURCE_APPLICATION,
};

/// `_NET_WM_STATE` の action。
const NET_WM_STATE_REMOVE: u32 = 0;
const NET_WM_STATE_ADD: u32 = 1;
/// 左ボタン。
const BUTTON_LEFT: u8 = 1;
/// 窓の題。
const TITLE: &str = "NeNe Ruler";

/// 枠の窓とその OS 資源。`Drop` で閉じる（規約 第 4 節）。
#[derive(Debug)]
pub struct RulerWindow<'d> {
    display: &'d Display,
    window: WindowWrapper<&'d RustConnection>,
    gc: GcontextWrapper<&'d RustConnection>,
    _colormap: ColormapWrapper<&'d RustConnection>,
    wm_moves: bool,
    drag: Option<Drag>,
    alt_f4: AltF4Gesture,
    shapes: PointerShapes<'d>,
    shown: Cursor,
}

impl<'d> RulerWindow<'d> {
    /// 枠なし・最前面・透明の窓を作り、最初のフレームを描いて表示する。
    ///
    /// # Errors
    ///
    /// X の要求か cairo が失敗した。
    pub fn open(display: &'d Display, frame: &RulerFrame) -> Result<Self, UiFailure> {
        let connection = display.connection();
        let screen = display.screen()?;
        let root = screen.root;
        let visual = display.visual();
        let window_rect = frame.layout().window();

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
                    | EventMask::LEAVE_WINDOW
                    | EventMask::KEY_PRESS
                    | EventMask::KEY_RELEASE,
            );
        let window = WindowWrapper::create_window(
            connection,
            visual.depth(),
            root,
            0,
            0,
            window_rect.width(),
            window_rect.height(),
            0,
            WindowClass::INPUT_OUTPUT,
            visual.visual(),
            &attributes,
        )?;
        let gc = GcontextWrapper::create_gc(connection, window.window(), &CreateGCAux::new())?;
        let wm_moves = display.wm_supports(display.atoms().net_wm_moveresize);

        let opened = Self {
            display,
            window,
            gc,
            _colormap: colormap,
            wm_moves,
            drag: None,
            alt_f4: AltF4Gesture::IDLE,
            shapes: PointerShapes::load(display)?,
            shown: x11rb::NONE,
        };
        opened.declare_hints(frame.layer())?;
        opened.set_input_shape(&frame.hit_regions())?;
        connection.map_window(opened.window.window())?;
        opened.present(frame)?;
        Ok(opened)
    }

    /// この窓の ID。
    #[must_use]
    pub fn window(&self) -> Window {
        self.window.window()
    }

    /// 設定が変わった後に、枠を描き直し、当たり判定と重なり順を合わせる。
    ///
    /// # Errors
    ///
    /// X の要求か cairo が失敗した。
    pub fn refresh(&self, controller: &RulerController<'_>) -> Result<(), UiFailure> {
        let frame = controller.frame();
        self.set_input_shape(&frame.hit_regions())?;
        self.set_layer(frame.layer())?;
        self.present(&frame)
    }

    /// 1 つのイベント。X のイベントは開いた集合なので `if let` の連なりで書く（規約 第 4 節）。
    pub(crate) fn step(
        &mut self,
        controller: &mut RulerController<'_>,
        event: &Event,
    ) -> Result<Flow, UiFailure> {
        if let Event::Expose(expose) = event {
            if expose.count == 0 {
                self.present(&controller.frame())?;
            }
        } else if let Event::ConfigureNotify(configure) = event {
            controller.apply(RulerIntent::WindowResized {
                width_px: configure.width,
                height_px: configure.height,
            });
            let frame = controller.frame();
            self.set_input_shape(&frame.hit_regions())?;
            self.present(&frame)?;
        } else if let Event::ButtonPress(press) = event {
            return self.pressed(controller, press);
        } else if let Event::MotionNotify(motion) = event {
            self.dragged((motion.root_x, motion.root_y))?;
            self.shape_pointer(controller, (motion.event_x, motion.event_y))?;
            self.moved_off_the_mark(controller, Some((motion.event_x, motion.event_y)))?;
        } else if let Event::LeaveNotify(_) = event {
            // 中身は入力 shape の外なので、数字から中身へ動くと MotionNotify ではなく LeaveNotify が届く。
            self.moved_off_the_mark(controller, None)?;
        } else if let Event::ButtonRelease(_) = event {
            self.drag = None;
        } else if let Event::KeyPress(key) = event {
            let input = self.display.key_input(key);
            if self
                .alt_f4
                .pressed(input, key.state.contains(KeyButMask::MOD1))
            {
                return Ok(Flow::Quit);
            }
            match input {
                KeyInput::Escape => return Ok(Flow::Quit),
                KeyInput::Arrow(direction) => {
                    self.nudge(Nudge::new(direction, key.state.contains(KeyButMask::SHIFT)))?;
                }
                KeyInput::Return
                | KeyInput::Backspace
                | KeyInput::Character(_)
                | KeyInput::F4
                | KeyInput::Alt
                | KeyInput::Other => {}
            }
        } else if let Event::KeyRelease(key) = event {
            // WSLg では Alt+F4 の押下が届かず、F4 の離しだけが届く（ADR 0009）。
            let input = self.display.key_input(key);
            if self
                .alt_f4
                .released(input, key.state.contains(KeyButMask::MOD1))
            {
                return Ok(Flow::Quit);
            }
        } else if let Event::ClientMessage(message) = event {
            let atoms = self.display.atoms();
            if message.type_ == atoms.wm_protocols
                && message.data.as_data32().first() == Some(&atoms.wm_delete_window)
            {
                return Ok(Flow::Quit);
            }
        }
        Ok(Flow::Continue)
    }

    /// 左ボタンで掴んだ場所の意味に従う。移動と伸縮は WM に任せ、できなければ自前で引く。
    fn pressed(
        &mut self,
        controller: &mut RulerController<'_>,
        press: &ButtonPressEvent,
    ) -> Result<Flow, UiFailure> {
        if press.detail != BUTTON_LEFT {
            return Ok(Flow::Continue);
        }
        let frame = controller.frame();
        let Some(point) = PixelPoint::from_event(press.event_x, press.event_y) else {
            return Ok(Flow::Continue);
        };
        let Some(target) = hit_region::target_at(&frame.hit_regions(), point) else {
            return Ok(Flow::Continue);
        };
        let direction = match target {
            HitTarget::Move => MOVERESIZE_MOVE,
            HitTarget::Resize(Anchor::TopLeft) => MOVERESIZE_SIZE_TOPLEFT,
            HitTarget::Resize(Anchor::BottomRight) => MOVERESIZE_SIZE_BOTTOMRIGHT,
            HitTarget::Gear => return Ok(Flow::OpenSettings),
            HitTarget::Copy(axis) => {
                controller.apply(RulerIntent::CopyText(axis));
                self.present(&controller.frame())?;
                return Ok(Flow::Continue);
            }
        };
        if self.wm_moves {
            undecorated::ask_wm_to_move_resize(
                self.display,
                self.window.window(),
                press,
                direction,
            )?;
        } else {
            let start = undecorated::geometry_of(self.display, self.window.window())?;
            self.drag = Some(Drag::begin(target, (press.root_x, press.root_y), start));
        }
        Ok(Flow::Continue)
    }

    /// ポインタの下の当たり判定に合わせて窓のカーソルを差し替える。変わったときだけ X へ送る（Issue #36）。
    fn shape_pointer(
        &mut self,
        controller: &RulerController<'_>,
        position: (i16, i16),
    ) -> Result<(), UiFailure> {
        let target = PixelPoint::from_event(position.0, position.1)
            .and_then(|point| hit_region::target_at(&controller.frame().hit_regions(), point));
        let cursor = self.shapes.cursor_for(target);
        if cursor == self.shown {
            return Ok(());
        }
        self.display.connection().change_window_attributes(
            self.window.window(),
            &ChangeWindowAttributesAux::new().cursor(cursor),
        )?;
        self.display.connection().flush()?;
        self.shown = cursor;
        Ok(())
    }

    /// コピーの印があり、ポインタがその数字から離れたら印を消す（ADR 0007 の 5）。
    fn moved_off_the_mark(
        &self,
        controller: &mut RulerController<'_>,
        position: Option<(i16, i16)>,
    ) -> Result<(), UiFailure> {
        let frame = controller.frame();
        let Some(mark) = frame.copy_mark() else {
            return Ok(());
        };
        let hit = frame
            .layout()
            .label_hit(mark.axis(), frame.text(mark.axis()).chars());
        let still_over = position
            .and_then(|(x, y)| PixelPoint::from_event(x, y))
            .is_some_and(|point| hit.contains(point));
        if still_over {
            return Ok(());
        }
        controller.apply(RulerIntent::ClearCopyMark);
        self.present(&controller.frame())
    }

    /// 矢印で枠を動かす（FR-015）。WM があれば WM が位置を決め直す。
    fn nudge(&self, nudge: Nudge) -> Result<(), UiFailure> {
        let geometry = undecorated::geometry_of(self.display, self.window.window())?;
        self.display.connection().configure_window(
            self.window.window(),
            &ConfigureWindowAux::new()
                .x(geometry.x().saturating_add(nudge.dx()))
                .y(geometry.y().saturating_add(nudge.dy())),
        )?;
        self.display.connection().flush()?;
        Ok(())
    }

    /// 自前のドラッグ中なら窓の幾何を更新する。
    fn dragged(&self, root: (i16, i16)) -> Result<(), UiFailure> {
        let Some(drag) = self.drag else {
            return Ok(());
        };
        let minimum = u32::from(Length::MIN_PX);
        let geometry = drag.geometry_at(root, minimum);
        self.display.connection().configure_window(
            self.window.window(),
            &ConfigureWindowAux::new()
                .x(geometry.x())
                .y(geometry.y())
                .width(geometry.width())
                .height(geometry.height()),
        )?;
        self.display.connection().flush()?;
        Ok(())
    }

    /// 装飾なし・最前面・閉じる要求・題を WM へ伝える（FR-001 / FR-013 / FR-015）。
    fn declare_hints(&self, layer: WindowLayer) -> Result<(), UiFailure> {
        let connection = self.display.connection();
        let atoms = self.display.atoms();
        let window = self.window.window();
        undecorated::declare_undecorated(self.display, window)?;
        let states: &[u32] = match layer {
            WindowLayer::AlwaysOnTop => &[atoms.net_wm_state_above],
            WindowLayer::Normal => &[],
        };
        connection.change_property32(
            PropMode::REPLACE,
            window,
            atoms.net_wm_state,
            AtomEnum::ATOM,
            states,
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

    /// 当たり判定の矩形だけを入力 shape にする。それ以外はクリックが下へ届く（FR-017）。
    fn set_input_shape(&self, regions: &[HitRegion]) -> Result<(), UiFailure> {
        let connection = self.display.connection();
        let rectangles = regions
            .iter()
            .map(|region| x_rectangle(region.rect()))
            .collect::<Result<Vec<Rectangle>, UiFailure>>()?;
        let region = xfixes::RegionWrapper::create_region(connection, &rectangles)?;
        connection.xfixes_set_window_shape_region(
            self.window.window(),
            shape::SK::INPUT,
            0,
            0,
            region.region(),
        )?;
        Ok(())
    }

    /// フレームを描き切ってから `PutImage` で転送する（ADR 0002 の 4）。
    fn present(&self, frame: &RulerFrame) -> Result<(), UiFailure> {
        let mut canvas = Canvas::render(frame, self.display.visual().format())?;
        Transfer::new(
            (self.window.window(), self.gc.gcontext()),
            canvas.width(),
            self.display.visual().depth(),
        )
        .present(self.display.connection(), &mut canvas)
    }

    /// 表示中の窓の重なり順を変える。`_NET_WM_STATE` は map の後は WM へのメッセージで変える（FR-013）。
    fn set_layer(&self, layer: WindowLayer) -> Result<(), UiFailure> {
        let connection = self.display.connection();
        let atoms = self.display.atoms();
        let root = self.display.screen()?.root;
        let action = match layer {
            WindowLayer::AlwaysOnTop => NET_WM_STATE_ADD,
            WindowLayer::Normal => NET_WM_STATE_REMOVE,
        };
        let message = ClientMessageEvent::new(
            32,
            self.window.window(),
            atoms.net_wm_state,
            [action, atoms.net_wm_state_above, 0, SOURCE_APPLICATION, 0],
        );
        connection.send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            message,
        )?;
        connection.flush()?;
        Ok(())
    }
}

/// core の矩形を X の矩形へ。`Length::MAX_PX` の範囲なら必ず収まる。
fn x_rectangle(rect: PixelRect) -> Result<Rectangle, UiFailure> {
    Ok(Rectangle {
        x: i16::try_from(rect.x()).map_err(|_error| UiFailure::GeometryOutOfRange)?,
        y: i16::try_from(rect.y()).map_err(|_error| UiFailure::GeometryOutOfRange)?,
        width: rect.width(),
        height: rect.height(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangles_convert_within_the_x_limits() {
        let rect = x_rectangle(PixelRect::new(4, 4, 400, 1)).unwrap();
        assert_eq!((rect.x, rect.y, rect.width, rect.height), (4, 4, 400, 1));
    }
}
