//! X への接続と、窓を作る前に決まること（画面・atom・visual・透明）。

use nene_ruler_application::transparency::Transparency;
use x11rb::connection::Connection;
use x11rb::protocol::xfixes::ConnectionExt as _;
use x11rb::protocol::xproto::{ConnectionExt as _, Screen};
use x11rb::rust_connection::RustConnection;

use crate::atoms::Atoms;
use crate::ui_failure::UiFailure;
use crate::window_visual::WindowVisual;

/// 要求する `XFixes` の版。入力 shape（`SetWindowShapeRegion`）は 2.0 から。
const XFIXES_MAJOR: u32 = 2;
const XFIXES_MINOR: u32 = 0;

/// 接続と、その画面で決まった visual と透明。adapters-x11 とは別の接続（ADR 0002 の 1）。
#[derive(Debug)]
pub struct Display {
    connection: RustConnection,
    screen_index: usize,
    atoms: Atoms,
    visual: WindowVisual,
    transparency: Transparency,
}

impl Display {
    /// `DISPLAY` に繋ぎ、合成器の有無と visual から透明に描けるかを決める（ADR 0005 の 5）。
    ///
    /// # Errors
    ///
    /// 繋げない・画面が無い・使える visual が無い・X の要求が失敗した。
    pub fn connect() -> Result<Self, UiFailure> {
        let (connection, screen_index) = x11rb::connect(None)?;
        let atoms = Atoms::intern(&connection)?;
        // XFixes は版の交渉を先に要する。入力 shape は版 2 から。
        connection
            .xfixes_query_version(XFIXES_MAJOR, XFIXES_MINOR)?
            .reply()?;
        let screen = connection
            .setup()
            .roots
            .get(screen_index)
            .ok_or(UiFailure::NoScreen)?;
        let composited = connection
            .get_selection_owner(atoms.net_wm_cm_s0)?
            .reply()?
            .owner
            != 0;
        let (visual, transparency) = match (composited, WindowVisual::argb(screen)) {
            (true, Some(argb)) => (argb, Transparency::Composited),
            (false, Some(argb)) => (argb, Transparency::Opaque),
            (true | false, None) => (
                WindowVisual::root(screen).ok_or(UiFailure::NoUsableVisual)?,
                Transparency::Opaque,
            ),
        };
        Ok(Self {
            connection,
            screen_index,
            atoms,
            visual,
            transparency,
        })
    }

    /// 接続。
    #[must_use]
    pub const fn connection(&self) -> &RustConnection {
        &self.connection
    }

    /// 既定の画面。
    ///
    /// # Errors
    ///
    /// 接続時に在った画面が消えることは無いが、型の上では `Option` なので失敗として返す。
    pub fn screen(&self) -> Result<&Screen, UiFailure> {
        self.connection
            .setup()
            .roots
            .get(self.screen_index)
            .ok_or(UiFailure::NoScreen)
    }

    /// intern 済みの atom。
    #[must_use]
    pub const fn atoms(&self) -> Atoms {
        self.atoms
    }

    /// 窓に使う visual。
    #[must_use]
    pub const fn visual(&self) -> WindowVisual {
        self.visual
    }

    /// 透明に描けるか。application が起動時に一度取り込む（ARC-004）。
    #[must_use]
    pub const fn transparency(&self) -> Transparency {
        self.transparency
    }
}
