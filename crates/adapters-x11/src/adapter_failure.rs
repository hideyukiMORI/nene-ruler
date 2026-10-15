//! アダプタが起動時に失敗した理由。合成ルートが 1 行で示す（ARC-006）。

use core::fmt;

use x11rb::errors::ConnectError;

/// アダプタの X 接続の失敗。閉じた集合（ARC-010）。
#[derive(Debug)]
pub enum AdapterFailure {
    /// X サーバに繋げなかった。
    Connect(ConnectError),
}

impl fmt::Display for AdapterFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(error) => write!(formatter, "cannot connect to the X server: {error}"),
        }
    }
}

impl From<ConnectError> for AdapterFailure {
    fn from(error: ConnectError) -> Self {
        Self::Connect(error)
    }
}
