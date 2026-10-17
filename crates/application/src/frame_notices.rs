//! 枠の窓に出す通知の組。

use alloc::vec::Vec;

use crate::frame_notice::FrameNotice;

/// いま当てはまる通知。順は固定（RS-016）。最前面と物理寸法の通知は外した（ADR 0012）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameNotices {
    opaque: bool,
}

impl FrameNotices {
    /// 当てはまるものを立てる。
    #[must_use]
    pub const fn new(opaque: bool) -> Self {
        Self { opaque }
    }

    /// 何も無い。
    pub const NONE: Self = Self::new(false);

    /// 描く順に並べる。
    #[must_use]
    pub fn list(self) -> Vec<FrameNotice> {
        let mut notices = Vec::new();
        if self.opaque {
            notices.push(FrameNotice::Opaque);
        }
        notices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_only_what_applies() {
        assert!(FrameNotices::NONE.list().is_empty());
        assert_eq!(FrameNotices::new(true).list(), [FrameNotice::Opaque]);
    }
}
