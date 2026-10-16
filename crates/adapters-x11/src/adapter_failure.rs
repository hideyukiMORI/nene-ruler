//! アダプタが起動時に失敗した理由。合成ルートが 1 行で示す（ARC-006）。

use core::fmt;

use x11rb::errors::{ConnectError, ConnectionError, ReplyError, ReplyOrIdError};

/// アダプタの X 接続の失敗。閉じた集合（ARC-010）。
#[derive(Debug)]
pub enum AdapterFailure {
    /// X サーバに繋げなかった。
    Connect(ConnectError),
    /// 要求を送れなかった。
    Connection(ConnectionError),
    /// 要求が X のエラーで返った。
    Reply(ReplyError),
    /// 要求が X のエラーで返った、または ID を使い切った。
    ReplyOrId(ReplyOrIdError),
}

impl fmt::Display for AdapterFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(error) => write!(formatter, "cannot connect to the X server: {error}"),
            Self::Connection(error) => write!(formatter, "X connection failed: {error}"),
            Self::Reply(error) => write!(formatter, "X request failed: {error}"),
            Self::ReplyOrId(error) => write!(formatter, "X request failed: {error}"),
        }
    }
}

impl From<ConnectError> for AdapterFailure {
    fn from(error: ConnectError) -> Self {
        Self::Connect(error)
    }
}

impl From<ConnectionError> for AdapterFailure {
    fn from(error: ConnectionError) -> Self {
        Self::Connection(error)
    }
}

impl From<ReplyError> for AdapterFailure {
    fn from(error: ReplyError) -> Self {
        Self::Reply(error)
    }
}

impl From<ReplyOrIdError> for AdapterFailure {
    fn from(error: ReplyOrIdError) -> Self {
        Self::ReplyOrId(error)
    }
}
