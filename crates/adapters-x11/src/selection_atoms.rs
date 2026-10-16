//! 選択（クリップボード）の手順で使う atom。

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, ConnectionExt as _};

use crate::adapter_failure::AdapterFailure;

/// `CLIPBOARD` と、答えられる target。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionAtoms {
    /// `CLIPBOARD` 選択。
    pub clipboard: Atom,
    /// `TARGETS`。答えられる target の一覧を求める要求。
    pub targets: Atom,
    /// `UTF8_STRING`。
    pub utf8_string: Atom,
    /// `TEXT`。所有者が好きな文字の形で答えてよい要求。
    pub text: Atom,
}

impl SelectionAtoms {
    /// 全部を intern する。
    ///
    /// # Errors
    ///
    /// X の要求が失敗した。
    pub fn intern<C: Connection>(connection: &C) -> Result<Self, AdapterFailure> {
        Ok(Self {
            clipboard: intern(connection, b"CLIPBOARD")?,
            targets: intern(connection, b"TARGETS")?,
            utf8_string: intern(connection, b"UTF8_STRING")?,
            text: intern(connection, b"TEXT")?,
        })
    }
}

/// 1 つの atom を intern する。
fn intern<C: Connection>(connection: &C, name: &[u8]) -> Result<Atom, AdapterFailure> {
    Ok(connection.intern_atom(false, name)?.reply()?.atom)
}
