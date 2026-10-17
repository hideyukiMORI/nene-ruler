//! 枠の窓に出す通知の組。

use alloc::vec::Vec;

use crate::frame_notice::FrameNotice;

/// いま当てはまる通知。順は固定（RS-016）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameNotices {
    opaque: bool,
    no_screen_metrics: bool,
    no_always_on_top: bool,
}

impl FrameNotices {
    /// 当てはまるものを立てる。
    #[must_use]
    pub const fn new(opaque: bool, no_screen_metrics: bool, no_always_on_top: bool) -> Self {
        Self {
            opaque,
            no_screen_metrics,
            no_always_on_top,
        }
    }

    /// 何も無い。
    pub const NONE: Self = Self::new(false, false, false);

    /// 描く順に並べる。
    #[must_use]
    pub fn list(self) -> Vec<FrameNotice> {
        let mut notices = Vec::new();
        if self.opaque {
            notices.push(FrameNotice::Opaque);
        }
        if self.no_screen_metrics {
            notices.push(FrameNotice::NoScreenMetrics);
        }
        if self.no_always_on_top {
            notices.push(FrameNotice::NoAlwaysOnTop);
        }
        notices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_in_a_fixed_order() {
        assert!(FrameNotices::NONE.list().is_empty());
        assert_eq!(
            FrameNotices::new(true, true, true).list(),
            [
                FrameNotice::Opaque,
                FrameNotice::NoScreenMetrics,
                FrameNotice::NoAlwaysOnTop
            ]
        );
        assert_eq!(
            FrameNotices::new(false, true, false).list(),
            [FrameNotice::NoScreenMetrics]
        );
    }
}
