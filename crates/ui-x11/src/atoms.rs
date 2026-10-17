//! 使う atom をまとめて intern する。

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, ConnectionExt as _};

use crate::ui_failure::UiFailure;

/// 窓の装飾・最前面・移動と伸縮・合成器の検出・閉じる要求に使う atom。
#[derive(Debug, Clone, Copy)]
pub struct Atoms {
    /// `_MOTIF_WM_HINTS`。装飾だけを外す（FR-001）。
    pub motif_wm_hints: Atom,
    /// `WM_PROTOCOLS`。
    pub wm_protocols: Atom,
    /// `WM_DELETE_WINDOW`。Alt+F4 は WM からこれで届く（FR-015）。
    pub wm_delete_window: Atom,
    /// `_NET_WM_STATE`。
    pub net_wm_state: Atom,
    /// `_NET_WM_STATE_ABOVE`。常に最前面（FR-013）。
    pub net_wm_state_above: Atom,
    /// `_NET_WM_MOVERESIZE`。WM に移動と伸縮を任せる（ADR 0005 の 4）。
    pub net_wm_moveresize: Atom,
    /// `_NET_SUPPORTED`。WM が `_NET_WM_MOVERESIZE` に対応するか。
    pub net_supported: Atom,
    /// `_NET_WM_NAME`。
    pub net_wm_name: Atom,
    /// `UTF8_STRING`。
    pub utf8_string: Atom,
    /// `_NET_WM_CM_S0`。合成器の選択（画面 0）。
    pub net_wm_cm_s0: Atom,
}

/// 1 つの atom を intern する。
fn intern<C: Connection>(connection: &C, name: &[u8]) -> Result<Atom, UiFailure> {
    Ok(connection.intern_atom(false, name)?.reply()?.atom)
}

impl Atoms {
    /// 全部を intern する。
    ///
    /// # Errors
    ///
    /// X の要求が失敗した。
    pub fn intern<C: Connection>(connection: &C) -> Result<Self, UiFailure> {
        Ok(Self {
            motif_wm_hints: intern(connection, b"_MOTIF_WM_HINTS")?,
            wm_protocols: intern(connection, b"WM_PROTOCOLS")?,
            wm_delete_window: intern(connection, b"WM_DELETE_WINDOW")?,
            net_wm_state: intern(connection, b"_NET_WM_STATE")?,
            net_wm_state_above: intern(connection, b"_NET_WM_STATE_ABOVE")?,
            net_wm_moveresize: intern(connection, b"_NET_WM_MOVERESIZE")?,
            net_supported: intern(connection, b"_NET_SUPPORTED")?,
            net_wm_name: intern(connection, b"_NET_WM_NAME")?,
            utf8_string: intern(connection, b"UTF8_STRING")?,
            net_wm_cm_s0: intern(connection, b"_NET_WM_CM_S0")?,
        })
    }
}
