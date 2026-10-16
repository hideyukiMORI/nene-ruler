//! 矢印で枠を動かす量（FR-015）。1 px、Shift で 10 px。

use crate::nudge_direction::NudgeDirection;

/// 向きと歩幅。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nudge {
    direction: NudgeDirection,
    step_px: i32,
}

impl Nudge {
    /// 矢印だけの歩幅。
    const SMALL_PX: i32 = 1;
    /// Shift ＋ 矢印の歩幅。
    const LARGE_PX: i32 = 10;

    /// 向きと、Shift が押されていたかから作る。
    #[must_use]
    pub const fn new(direction: NudgeDirection, shifted: bool) -> Self {
        Self {
            direction,
            step_px: if shifted {
                Self::LARGE_PX
            } else {
                Self::SMALL_PX
            },
        }
    }

    /// 横の移動量。右が正。
    #[must_use]
    pub const fn dx(self) -> i32 {
        match self.direction {
            NudgeDirection::Left => -self.step_px,
            NudgeDirection::Right => self.step_px,
            NudgeDirection::Up | NudgeDirection::Down => 0,
        }
    }

    /// 縦の移動量。下が正。
    #[must_use]
    pub const fn dy(self) -> i32 {
        match self.direction {
            NudgeDirection::Up => -self.step_px,
            NudgeDirection::Down => self.step_px,
            NudgeDirection::Left | NudgeDirection::Right => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_pixel_without_shift_ten_with() {
        assert_eq!(Nudge::new(NudgeDirection::Right, false).dx(), 1_i32);
        assert_eq!(Nudge::new(NudgeDirection::Left, true).dx(), -10_i32);
        assert_eq!(Nudge::new(NudgeDirection::Down, true).dy(), 10_i32);
        assert_eq!(Nudge::new(NudgeDirection::Up, false).dy(), -1_i32);
        assert_eq!(Nudge::new(NudgeDirection::Up, false).dx(), 0_i32);
        assert_eq!(Nudge::new(NudgeDirection::Left, false).dy(), 0_i32);
    }
}
