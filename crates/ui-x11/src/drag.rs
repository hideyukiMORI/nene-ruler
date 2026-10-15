//! WM が `_NET_WM_MOVERESIZE` に対応しないときの、自前のドラッグ（ADR 0005 の 4）。

use nene_ruler_core::anchor::Anchor;
use nene_ruler_core::hit_target::HitTarget;

use crate::window_geometry::WindowGeometry;

/// ポインタの動きを窓の幾何に写すドラッグ 1 回分。UI 区画の可変状態（ARC-005）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Drag {
    target: HitTarget,
    origin_x: i32,
    origin_y: i32,
    start: WindowGeometry,
}

impl Drag {
    /// 掴んだ場所と、そのときの根座標と窓の幾何から始める。
    #[must_use]
    pub fn begin(target: HitTarget, origin: (i16, i16), start: WindowGeometry) -> Self {
        Self {
            target,
            origin_x: i32::from(origin.0),
            origin_y: i32::from(origin.1),
            start,
        }
    }

    /// ポインタが根座標 `(root_x, root_y)` にあるときの窓の幾何。
    /// 掴んだのが移動でも伸縮でもなければ、幾何は変わらない。
    #[must_use]
    pub fn geometry_at(self, root: (i16, i16), minimum: u32) -> WindowGeometry {
        let dx = i32::from(root.0) - self.origin_x;
        let dy = i32::from(root.1) - self.origin_y;
        match self.target {
            HitTarget::Move => WindowGeometry::new(
                self.start.x() + dx,
                self.start.y() + dy,
                self.start.width(),
                self.start.height(),
            ),
            HitTarget::Resize(Anchor::TopLeft) => {
                let width = shrink(self.start.width(), dx, minimum);
                let height = shrink(self.start.height(), dy, minimum);
                WindowGeometry::new(
                    self.start.x() + used(self.start.width(), width),
                    self.start.y() + used(self.start.height(), height),
                    width,
                    height,
                )
            }
            HitTarget::Resize(Anchor::BottomRight) => WindowGeometry::new(
                self.start.x(),
                self.start.y(),
                grow(self.start.width(), dx, minimum),
                grow(self.start.height(), dy, minimum),
            ),
            HitTarget::Gear | HitTarget::Copy(_) => self.start,
        }
    }
}

/// 原点側の角を `delta` だけ動かしたときの大きさ。`minimum` より小さくしない。
fn shrink(extent: u32, delta: i32, minimum: u32) -> u32 {
    grow(extent, delta.saturating_neg(), minimum)
}

/// 反対側の角を `delta` だけ動かしたときの大きさ。`minimum` より小さくしない。
fn grow(extent: u32, delta: i32, minimum: u32) -> u32 {
    let signed = i64::from(extent) + i64::from(delta);
    u32::try_from(signed.max(i64::from(minimum))).unwrap_or(u32::MAX)
}

/// 大きさが `before` から `after` になったとき、原点をどれだけ動かすか。
fn used(before: u32, after: u32) -> i32 {
    i32::try_from(i64::from(before) - i64::from(after)).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start() -> WindowGeometry {
        WindowGeometry::new(100, 200, 424, 264)
    }

    #[test]
    fn moving_shifts_the_origin_only() {
        let drag = Drag::begin(HitTarget::Move, (10, 10), start());
        assert_eq!(
            drag.geometry_at((15, 7), 64),
            WindowGeometry::new(105, 197, 424, 264)
        );
    }

    #[test]
    fn bottom_right_grows_the_extent() {
        let drag = Drag::begin(HitTarget::Resize(Anchor::BottomRight), (0, 0), start());
        assert_eq!(
            drag.geometry_at((20, -10), 64),
            WindowGeometry::new(100, 200, 444, 254)
        );
        assert_eq!(drag.geometry_at((-1000, 0), 64).width(), 64);
    }

    #[test]
    fn top_left_moves_the_origin_and_shrinks() {
        let drag = Drag::begin(HitTarget::Resize(Anchor::TopLeft), (0, 0), start());
        assert_eq!(
            drag.geometry_at((20, 10), 64),
            WindowGeometry::new(120, 210, 404, 254)
        );
        let clamped = drag.geometry_at((1000, 0), 64);
        assert_eq!((clamped.x(), clamped.width()), (460_i32, 64_u32));
    }

    #[test]
    fn other_targets_keep_the_geometry() {
        let drag = Drag::begin(HitTarget::Gear, (0, 0), start());
        assert_eq!(drag.geometry_at((50, 50), 64), start());
    }
}
