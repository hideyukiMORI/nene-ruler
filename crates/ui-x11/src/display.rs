//! X への接続と、窓を作る前に決まること（画面・atom・visual・透明）。

use nene_ruler_application::environment::Environment;
use nene_ruler_application::layering::Layering;
use nene_ruler_application::transparency::Transparency;
use nene_ruler_core::ui_scale::UiScale;
use x11rb::connection::Connection;
use x11rb::protocol::xfixes::ConnectionExt as _;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ConnectionExt as _, KeyButMask, KeyPressEvent, Screen,
};
use x11rb::rust_connection::RustConnection;

use crate::atoms::Atoms;
use crate::key_input::KeyInput;
use crate::key_map::KeyMap;
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
    key_map: KeyMap,
    wm_supported: Vec<Atom>,
    xft_dpi: Option<u32>,
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
        let key_map = Self::key_map(&connection)?;
        let wm_supported = Self::wm_supported(&connection, screen.root, atoms)?;
        let xft_dpi = Self::xft_dpi(&connection, screen.root)?;
        Ok(Self {
            connection,
            screen_index,
            atoms,
            visual,
            transparency,
            key_map,
            wm_supported,
            xft_dpi,
        })
    }

    /// ルート窓の `RESOURCE_MANAGER`（`xrdb` が置く文字列）の `Xft.dpi`。無ければ `None`（FR-016・ADR 0011）。
    fn xft_dpi(
        connection: &RustConnection,
        root: x11rb::protocol::xproto::Window,
    ) -> Result<Option<u32>, UiFailure> {
        let reply = connection
            .get_property(
                false,
                root,
                AtomEnum::RESOURCE_MANAGER,
                AtomEnum::STRING,
                0,
                u32::MAX / 4,
            )?
            .reply()?;
        Ok(UiScale::xft_dpi_in(&String::from_utf8_lossy(&reply.value)))
    }

    /// WM が `_NET_SUPPORTED` に挙げている atom。WM が無ければ空。
    fn wm_supported(
        connection: &RustConnection,
        root: x11rb::protocol::xproto::Window,
        atoms: Atoms,
    ) -> Result<Vec<Atom>, UiFailure> {
        let reply = connection
            .get_property(false, root, atoms.net_supported, AtomEnum::ATOM, 0, 1024)?
            .reply()?;
        Ok(reply.value32().map(Iterator::collect).unwrap_or_default())
    }

    /// WM がその atom を `_NET_SUPPORTED` に挙げているか。
    #[must_use]
    pub fn wm_supports(&self, atom: Atom) -> bool {
        self.wm_supported.contains(&atom)
    }

    /// 透明・最前面の対応・`Xft.dpi`。application が起動時に一度取り込む（ARC-004）。
    #[must_use]
    pub fn environment(&self) -> Environment {
        let layering = if self.wm_supports(self.atoms.net_wm_state_above) {
            Layering::Supported
        } else {
            Layering::Unsupported
        };
        Environment::new(self.transparency, layering).with_xft_dpi(self.xft_dpi)
    }

    /// キーボード配列を読む。
    fn key_map(connection: &RustConnection) -> Result<KeyMap, UiFailure> {
        let setup = connection.setup();
        let first = setup.min_keycode;
        let count = setup.max_keycode.saturating_sub(first).saturating_add(1);
        let mapping = connection.get_keyboard_mapping(first, count)?.reply()?;
        Ok(KeyMap::from_mapping(first, &mapping))
    }

    /// 押された鍵を、この製品が反応する鍵に読み替える。
    #[must_use]
    pub fn key_input(&self, key: &KeyPressEvent) -> KeyInput {
        let shifted = key.state.contains(KeyButMask::SHIFT);
        self.key_map
            .keysym(key.detail, shifted)
            .map_or(KeyInput::Other, KeyInput::from_keysym)
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
