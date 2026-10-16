//! コピーの結果の印。どの数字を、成功か失敗か。

use nene_ruler_core::axis::Axis;

use crate::clipboard_failure::ClipboardFailure;

/// ポインタが数字から離れるまで描く印（ADR 0007 の 5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyMark {
    axis: Axis,
    outcome: Result<(), ClipboardFailure>,
}

impl CopyMark {
    /// 軸と結果から作る。
    #[must_use]
    pub const fn new(axis: Axis, outcome: Result<(), ClipboardFailure>) -> Self {
        Self { axis, outcome }
    }

    /// どの数字か。
    #[must_use]
    pub const fn axis(self) -> Axis {
        self.axis
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
