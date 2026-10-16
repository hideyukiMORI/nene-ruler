//! 設定モーダルが開いている間の編集状態。

use nene_ruler_core::hex_entry::HexEntry;
use nene_ruler_core::rgb::Rgb;
use nene_ruler_core::rgb_failure::RgbFailure;

/// 直接入力の文字列と、直前の送信が拒まれた理由（FR-014 の「見える形で示す」）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalSession {
    entry: HexEntry,
    rejection: Option<RgbFailure>,
}

impl ModalSession {
    /// いまのインク色を入力欄に出した状態で開く。
    #[must_use]
    pub fn open(ink: Rgb) -> Self {
        Self {
            entry: HexEntry::showing(ink),
            rejection: None,
        }
    }

    /// 入力欄。
    #[must_use]
    pub const fn entry(&self) -> &HexEntry {
        &self.entry
    }

    /// 直前の送信が拒まれた理由。
    #[must_use]
    pub const fn rejection(&self) -> Option<RgbFailure> {
        self.rejection
    }

    /// 入力欄を差し替えた状態。拒否の表示は消す。
    #[must_use]
    pub fn with_entry(self, entry: HexEntry) -> Self {
        Self {
            entry,
            rejection: None,
        }
    }

    /// 送信が拒まれた状態。
    #[must_use]
    pub fn rejected(self, failure: RgbFailure) -> Self {
        Self {
            rejection: Some(failure),
            ..self
        }
    }
}
