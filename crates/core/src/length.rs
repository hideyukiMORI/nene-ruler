//! 枠の一辺が測っている物理画素の数（FR-005 の下限と上限を守る）。

use crate::length_failure::LengthFailure;

/// 枠の一辺の長さ。幅も高さもこれ。物理画素。
///
/// 🔴 フィールドは非公開で、生成経路は `new` と `clamped` だけ（RS-001）。
/// `u16` なのは X の窓の大きさが `u16` だからで、`MAX_PX` はそれより十分小さい。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Length(u16);

impl Length {
    /// 下限（施主指定 2026-10-17・Issue #54）。線を消した表示の細い鉤括弧（腕 6 px）が重ならない最小。
    pub const MIN_PX: u16 = 16;
    /// 上限。窓の座標を `u16` / `i16` で持てる範囲に収める（`FrameLayout` の余白を足しても `i16::MAX` 未満）。
    pub const MAX_PX: u16 = 16_384;
    /// 初期の幅（SPECIFICATION 第 2 節。決め打ちしない）。
    pub const INITIAL_WIDTH: Self = Self(400);
    /// 初期の高さ（同上）。
    pub const INITIAL_HEIGHT: Self = Self(240);

    /// 範囲内の値だけを受け入れる唯一のファクトリ。
    ///
    /// # Errors
    ///
    /// `MIN_PX` 未満なら `BelowMinimum`、`MAX_PX` 超なら `AboveMaximum`。
    pub const fn new(px: u16) -> Result<Self, LengthFailure> {
        if px < Self::MIN_PX {
            Err(LengthFailure::BelowMinimum)
        } else if px > Self::MAX_PX {
            Err(LengthFailure::AboveMaximum)
        } else {
            Ok(Self(px))
        }
    }

    /// 外から来た大きさ（WM の `ConfigureNotify` など）を `MIN_PX..=max` に収める。
    /// `max` は画面の大きさなど、その時点の上限。
    #[must_use]
    pub fn clamped(px: u16, max: Self) -> Self {
        Self(px.clamp(Self::MIN_PX, max.0))
    }

    /// 物理画素の数。
    #[must_use]
    pub const fn px(self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_bounds_inclusive() {
        assert_eq!(Length::new(16).map(Length::px), Ok(16));
        assert_eq!(Length::new(16_384).map(Length::px), Ok(16_384));
    }

    #[test]
    fn rejects_outside_the_bounds() {
        assert_eq!(Length::new(15), Err(LengthFailure::BelowMinimum));
        assert_eq!(Length::new(16_385), Err(LengthFailure::AboveMaximum));
    }

    #[test]
    fn clamps_into_the_range() {
        let max = Length::new(1_000).unwrap();
        assert_eq!(Length::clamped(10, max).px(), 16);
        assert_eq!(Length::clamped(5_000, max).px(), 1_000);
        assert_eq!(Length::clamped(300, max).px(), 300);
    }

    #[test]
    fn initial_values_are_within_the_range() {
        assert!(Length::new(Length::INITIAL_WIDTH.px()).is_ok());
        assert!(Length::new(Length::INITIAL_HEIGHT.px()).is_ok());
    }
}
