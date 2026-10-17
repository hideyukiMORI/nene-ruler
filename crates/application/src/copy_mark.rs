//! コピーの結果の印。どの数字を、成功か失敗か。

use nene_ruler_core::length_label::LengthLabel;

use crate::clipboard_failure::ClipboardFailure;

/// ポインタが数字から離れるまで描く印（ADR 0007 の 5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyMark {
    label: LengthLabel,
    outcome: Result<(), ClipboardFailure>,
}

impl CopyMark {
    /// 数字の種類と結果から作る。
    #[must_use]
    pub const fn new(label: LengthLabel, outcome: Result<(), ClipboardFailure>) -> Self {
        Self { label, outcome }
    }

    /// どの数字か。
    #[must_use]
    pub const fn label(self) -> LengthLabel {
        self.label
    }

    /// 成功したか。
    #[must_use]
    pub const fn succeeded(self) -> bool {
        self.outcome.is_ok()
    }

    /// 失敗の理由。
    #[must_use]
    pub const fn failure(self) -> Option<ClipboardFailure> {
        match self.outcome {
            Ok(()) => None,
            Err(failure) => Some(failure),
        }
    }
}
