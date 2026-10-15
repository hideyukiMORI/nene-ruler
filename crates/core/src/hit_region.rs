//! 当たり判定の矩形と意味の対。

use crate::hit_target::HitTarget;
use crate::pixel_point::PixelPoint;
use crate::pixel_rect::PixelRect;

/// クリックを受ける矩形と、そこを掴んだ意味。`XFixes` の入力 shape にもこの矩形を使う（FR-017）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitRegion {
    rect: PixelRect,
    target: HitTarget,
}

impl HitRegion {
    /// 矩形と意味から作る。
    #[must_use]
    pub const fn new(rect: PixelRect, target: HitTarget) -> Self {
        Self { rect, target }
    }

    /// 矩形。
    #[must_use]
    pub const fn rect(self) -> PixelRect {
        self.rect
    }

    /// 意味。
    #[must_use]
    pub const fn target(self) -> HitTarget {
        self.target
    }
}

/// 優先順に並んだ領域から、点を含む最初の意味を返す。どれにも当たらなければ `None`（素通し）。
#[must_use]
pub fn target_at(regions: &[HitRegion], point: PixelPoint) -> Option<HitTarget> {
    regions
        .iter()
        .find(|region| region.rect.contains(point))
        .map(|region| region.target())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_match_wins() {
        let regions = [
            HitRegion::new(PixelRect::new(0, 0, 10, 10), HitTarget::Gear),
            HitRegion::new(PixelRect::new(0, 0, 100, 100), HitTarget::Move),
        ];
        assert_eq!(
            target_at(&regions, PixelPoint::new(5, 5)),
            Some(HitTarget::Gear)
        );
        assert_eq!(
            target_at(&regions, PixelPoint::new(50, 50)),
            Some(HitTarget::Move)
        );
        assert_eq!(target_at(&regions, PixelPoint::new(150, 50)), None);
    }
}
