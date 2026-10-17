//! 飾りの無い窓（枠とモーダル）が WM とやりとりする要求。2 つの窓で同じ経路を使う（ARC-001・ADR 0014）。

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ButtonPressEvent, ClientMessageEvent, ConnectionExt as _, EventMask, PropMode, Window,
};
use x11rb::wrapper::ConnectionExt as _;

use crate::display::Display;
use crate::ui_failure::UiFailure;
use crate::window_geometry::WindowGeometry;

/// `_MOTIF_WM_HINTS` の `flags`: 装飾の指定だけを行う。
const MWM_HINTS_DECORATIONS: u32 = 1 << 1;
/// `_NET_WM_MOVERESIZE` の向き。
pub(crate) const MOVERESIZE_SIZE_TOPLEFT: u32 = 0;
pub(crate) const MOVERESIZE_SIZE_BOTTOMRIGHT: u32 = 4;
pub(crate) const MOVERESIZE_MOVE: u32 = 8;
/// `_NET_WM_MOVERESIZE` / `_NET_WM_STATE` の `source_indication`: 通常のアプリケーション。
pub(crate) const SOURCE_APPLICATION: u32 = 1;

/// 飾りを外し、Weston の XWM が影を描かない窓の種類にする。map の前に呼ぶ（FR-001・ADR 0008）。
///
/// # Errors
///
/// X の要求が失敗した。
pub(crate) fn declare_undecorated(display: &Display, window: Window) -> Result<(), UiFailure> {
    let connection = display.connection();
    let atoms = display.atoms();
    connection.change_property32(
        PropMode::REPLACE,
        window,
        atoms.motif_wm_hints,
        atoms.motif_wm_hints,
        &[MWM_HINTS_DECORATIONS, 0, 0, 0, 0],
    )?;
    // Weston の XWM は飾りなしの窓に影を描く。_KDE_NET_WM_WINDOW_TYPE_OVERRIDE が先頭にあれば描かない。
    // 他の WM は知らない種類を飛ばして NORMAL を読む（ADR 0008）。
    connection.change_property32(
        PropMode::REPLACE,
        window,
        atoms.net_wm_window_type,
        AtomEnum::ATOM,
        &[
            atoms.kde_net_wm_window_type_override,
            atoms.net_wm_window_type_normal,
        ],
    )?;
    Ok(())
}

/// `_NET_WM_MOVERESIZE` を根の窓へ送る。暗黙の grab を外してから。
///
/// # Errors
///
/// X の要求が失敗した。
pub(crate) fn ask_wm_to_move_resize(
    display: &Display,
    window: Window,
    press: &ButtonPressEvent,
    direction: u32,
) -> Result<(), UiFailure> {
    let connection = display.connection();
    let root = display.screen()?.root;
    connection.ungrab_pointer(press.time)?;
    let message = ClientMessageEvent::new(
        32,
        window,
        display.atoms().net_wm_moveresize,
        [
            u32::try_from(i32::from(press.root_x)).unwrap_or(0),
            u32::try_from(i32::from(press.root_y)).unwrap_or(0),
            direction,
            u32::from(press.detail),
            SOURCE_APPLICATION,
        ],
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

/// 窓の根からの位置と大きさ。自前のドラッグの起点に使う。
///
/// # Errors
///
/// X の要求が失敗した。
pub(crate) fn geometry_of(display: &Display, window: Window) -> Result<WindowGeometry, UiFailure> {
    let connection = display.connection();
    let root = display.screen()?.root;
    let geometry = connection.get_geometry(window)?.reply()?;
    let origin = connection
        .translate_coordinates(window, root, 0, 0)?
        .reply()?;
    Ok(WindowGeometry::new(
        i32::from(origin.dst_x),
        i32::from(origin.dst_y),
        u32::from(geometry.width),
        u32::from(geometry.height),
    ))
}
