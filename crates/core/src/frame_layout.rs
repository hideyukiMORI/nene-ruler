//! 枠の幾何。線・目盛・鉤括弧・ギア・数字の置き場と当たり判定を、物理画素で計算する。
//!
//! UI はここが返す矩形と点を cairo と X に写すだけで、位置の判断を持たない（ARC-011 / ADR 0005 の 3）。
//! 描く矩形（`ink_rects`）と触れる矩形（`hit_regions`）が同じ `FrameLayout` から出ることで、
//! 「見えるものと触れるもの」の経路を 1 本にする（ARC-001）。
//!
//! 値はすべて SPECIFICATION 第 2 節の「決め打ちしない」初期値である。

use alloc::vec::Vec;

use crate::anchor::Anchor;
use crate::axis::Axis;
use crate::edge::Edge;
use crate::frame_size::FrameSize;
use crate::hit_region::HitRegion;
use crate::hit_target::HitTarget;
use crate::pixel_point::PixelPoint;
use crate::pixel_rect::PixelRect;
use crate::tick_kind::TickKind;
use crate::tick_label::TickLabel;
use crate::tick_mark::TickMark;

/// 線の両側に広げる当たり判定の帯。窓の左上から枠の左上までの余白でもある。
const GRAB_PAD_PX: u16 = 4;
/// 線の太さ。
const LINE_PX: u16 = 1;
/// 鉤括弧の腕の長さ。
const ANCHOR_ARM_PX: u16 = 12;
/// 鉤括弧の太さ。
const ANCHOR_THICKNESS_PX: u16 = 2;
/// ギアの直径。
const GEAR_DIAMETER_PX: u16 = 16;
/// 右下の鉤括弧からギアまでの隙間。
const GEAR_GAP_PX: u16 = 4;
/// 枠の右下の外側に要る余白。ギアが収まる分。
const BEYOND_PX: u16 = GEAR_GAP_PX + GEAR_DIAMETER_PX;
/// 主目盛の数字の高さ。
const TICK_LABEL_PX: u16 = 9;
/// 幅・高さの数字の高さ。
const LABEL_PX: u16 = 11;
/// 幅・高さの数字の 1 文字あたりの幅の見積り（当たり判定だけに使う）。
const LABEL_ADVANCE_PX: u16 = 7;
/// 目盛の先端から文字までの隙間。
const TEXT_GAP_PX: u16 = 2;
/// 通知の文字の高さと行の間隔。
const NOTICE_PX: u16 = 9;
const NOTICE_LINE_PX: u16 = 12;

/// 枠の大きさから決まる、窓の中の全ての置き場。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameLayout {
    size: FrameSize,
}

impl FrameLayout {
    /// 枠の大きさから作る。
    #[must_use]
    pub const fn new(size: FrameSize) -> Self {
        Self { size }
    }

    /// 枠の大きさ。
    #[must_use]
    pub const fn size(self) -> FrameSize {
        self.size
    }

    /// 窓全体。枠の周りの帯とギアを含む。
    #[must_use]
    pub fn window(self) -> PixelRect {
        PixelRect::new(
            0,
            0,
            GRAB_PAD_PX + self.size.width().px() + BEYOND_PX,
            GRAB_PAD_PX + self.size.height().px() + BEYOND_PX,
        )
    }

    /// 窓の大きさから枠の大きさへ戻す（WM の `ConfigureNotify` を受けたとき）。余白を引く。
    #[must_use]
    pub const fn frame_px_for_window(window_px: u16) -> u16 {
        window_px.saturating_sub(GRAB_PAD_PX + BEYOND_PX)
    }

    /// 枠の矩形。線はこの矩形の外周の画素に乗る。幅 400 なら左の線と右の線の間が 400 画素（両端を含む）。
    #[must_use]
    pub fn frame(self) -> PixelRect {
        PixelRect::new(
            GRAB_PAD_PX,
            GRAB_PAD_PX,
            self.size.width().px(),
            self.size.height().px(),
        )
    }

    /// 1 辺の線（太さ 1 px）。
    #[must_use]
    pub fn line(self, edge: Edge) -> PixelRect {
        let frame = self.frame();
        match edge {
            Edge::Top => PixelRect::new(frame.x(), frame.y(), frame.width(), LINE_PX),
            Edge::Bottom => PixelRect::new(frame.x(), frame.bottom(), frame.width(), LINE_PX),
            Edge::Left => PixelRect::new(frame.x(), frame.y(), LINE_PX, frame.height()),
            Edge::Right => PixelRect::new(frame.right(), frame.y(), LINE_PX, frame.height()),
        }
    }

    /// 1 辺の当たり判定の帯。線の両側に `GRAB_PAD_PX`。
    #[must_use]
    pub fn line_band(self, edge: Edge) -> PixelRect {
        let line = self.line(edge);
        PixelRect::new(
            line.x() - GRAB_PAD_PX,
            line.y() - GRAB_PAD_PX,
            line.width() + 2 * GRAB_PAD_PX,
            line.height() + 2 * GRAB_PAD_PX,
        )
    }

    /// 1 辺の目盛の矩形。線から内側へ伸びる。
    pub fn ticks(self, edge: Edge) -> impl Iterator<Item = PixelRect> {
        let frame = self.frame();
        TickMark::along(self.size.along(edge.axis())).map(move |mark| {
            let reach = mark.kind().length_px();
            match edge {
                Edge::Top => {
                    PixelRect::new(frame.x() + mark.offset(), frame.y() + LINE_PX, 1, reach)
                }
                Edge::Bottom => {
                    PixelRect::new(frame.x() + mark.offset(), frame.bottom() - reach, 1, reach)
                }
                Edge::Left => {
                    PixelRect::new(frame.x() + LINE_PX, frame.y() + mark.offset(), reach, 1)
                }
                Edge::Right => {
                    PixelRect::new(frame.right() - reach, frame.y() + mark.offset(), reach, 1)
                }
            }
        })
    }

    /// 上辺と左辺の主目盛に添える数字。下辺と右辺には添えない（SPECIFICATION 第 2 節）。
    pub fn tick_labels(self, edge: Edge) -> impl Iterator<Item = TickLabel> {
        let frame = self.frame();
        let inside = LINE_PX + TickKind::Major.length_px() + TEXT_GAP_PX;
        TickMark::along(self.size.along(edge.axis()))
            .filter(move |mark| mark.kind() == TickKind::Major)
            .filter_map(move |mark| {
                let baseline = match edge {
                    Edge::Top => PixelPoint::new(
                        frame.x() + mark.offset() + TEXT_GAP_PX,
                        frame.y() + inside + TICK_LABEL_PX,
                    ),
                    Edge::Left => PixelPoint::new(
                        frame.x() + inside,
                        frame.y() + mark.offset() + TICK_LABEL_PX / 2,
                    ),
                    Edge::Bottom | Edge::Right => return None,
                };
                Some(TickLabel::new(mark.offset(), baseline))
            })
    }

    /// 鉤括弧を作る 2 本の腕。
    #[must_use]
    pub fn anchor_arms(self, anchor: Anchor) -> [PixelRect; 2] {
        let frame = self.frame();
        match anchor {
            Anchor::TopLeft => [
                PixelRect::new(frame.x(), frame.y(), ANCHOR_ARM_PX, ANCHOR_THICKNESS_PX),
                PixelRect::new(frame.x(), frame.y(), ANCHOR_THICKNESS_PX, ANCHOR_ARM_PX),
            ],
            Anchor::BottomRight => [
                PixelRect::new(
                    frame.right() + 1 - ANCHOR_ARM_PX,
                    frame.bottom() + 1 - ANCHOR_THICKNESS_PX,
                    ANCHOR_ARM_PX,
                    ANCHOR_THICKNESS_PX,
                ),
                PixelRect::new(
                    frame.right() + 1 - ANCHOR_THICKNESS_PX,
                    frame.bottom() + 1 - ANCHOR_ARM_PX,
                    ANCHOR_THICKNESS_PX,
                    ANCHOR_ARM_PX,
                ),
            ],
        }
    }

    /// 鉤括弧の当たり判定。腕を囲む正方形に、外側の帯を足したもの。
    #[must_use]
    pub fn anchor_hit(self, anchor: Anchor) -> PixelRect {
        let frame = self.frame();
        let side = ANCHOR_ARM_PX + GRAB_PAD_PX;
        match anchor {
            Anchor::TopLeft => {
                PixelRect::new(frame.x() - GRAB_PAD_PX, frame.y() - GRAB_PAD_PX, side, side)
            }
            Anchor::BottomRight => PixelRect::new(
                frame.right() + 1 - ANCHOR_ARM_PX,
                frame.bottom() + 1 - ANCHOR_ARM_PX,
                side,
                side,
            ),
        }
    }

    /// ギアの外接矩形。右下の鉤括弧のすぐ外側（FR-009）。
    #[must_use]
    pub fn gear(self) -> PixelRect {
        let frame = self.frame();
        PixelRect::new(
            frame.right() + 1 + GEAR_GAP_PX,
            frame.bottom() + 1 + GEAR_GAP_PX,
            GEAR_DIAMETER_PX,
            GEAR_DIAMETER_PX,
        )
    }

    /// 幅・高さの数字の置き場。横軸は上辺の中央の下、縦軸は左辺の中央の右に 90 度回して置く。
    /// 返す点は、文字の中央のベースライン。
    #[must_use]
    pub fn label_baseline(self, axis: Axis) -> PixelPoint {
        let frame = self.frame();
        let inside = LINE_PX
            + TickKind::Major.length_px()
            + TEXT_GAP_PX
            + TICK_LABEL_PX
            + TEXT_GAP_PX
            + LABEL_PX;
        match axis {
            Axis::Horizontal => PixelPoint::new(frame.x() + frame.width() / 2, frame.y() + inside),
            Axis::Vertical => PixelPoint::new(frame.x() + inside, frame.y() + frame.height() / 2),
        }
    }

    /// 枠の中に出す通知（FR-014）の `index` 行目のベースライン。下辺の目盛の上に、下から上へ積む。
    #[must_use]
    pub fn notice_baseline(self, index: u16) -> PixelPoint {
        let frame = self.frame();
        let inside = LINE_PX + TickKind::Major.length_px() + TEXT_GAP_PX;
        PixelPoint::new(
            frame.x() + ANCHOR_ARM_PX + TEXT_GAP_PX,
            frame
                .bottom()
                .saturating_sub(inside + index * NOTICE_LINE_PX),
        )
    }

    /// 通知の文字の高さ。
    #[must_use]
    pub const fn notice_px() -> u16 {
        NOTICE_PX
    }

    /// 幅・高さの数字の当たり判定。文字数から見積もる。
    #[must_use]
    pub fn label_hit(self, axis: Axis, chars: u16) -> PixelRect {
        let baseline = self.label_baseline(axis);
        let span = chars * LABEL_ADVANCE_PX;
        let half = span / 2;
        match axis {
            Axis::Horizontal => PixelRect::new(
                baseline.x().saturating_sub(half),
                baseline.y().saturating_sub(LABEL_PX),
                span,
                LABEL_PX + TEXT_GAP_PX,
            ),
            Axis::Vertical => PixelRect::new(
                baseline.x().saturating_sub(LABEL_PX),
                baseline.y().saturating_sub(half),
                LABEL_PX + TEXT_GAP_PX,
                span,
            ),
        }
    }

    /// インク色で塗る矩形のすべて。線・目盛・鉤括弧。順は決定的（RS-016）。
    #[must_use]
    pub fn ink_rects(self) -> Vec<PixelRect> {
        let mut rects = Vec::new();
        for edge in Edge::ALL {
            rects.push(self.line(edge));
            rects.extend(self.ticks(edge));
        }
        rects.extend(self.anchor_arms(Anchor::TopLeft));
        rects.extend(self.anchor_arms(Anchor::BottomRight));
        rects
    }

    /// 当たり判定を優先順に。鉤括弧 → ギア → 数字 → 線の帯。
    #[must_use]
    pub fn hit_regions(self, width_chars: u16, height_chars: u16) -> Vec<HitRegion> {
        let mut regions = Vec::new();
        for anchor in [Anchor::TopLeft, Anchor::BottomRight] {
            regions.push(HitRegion::new(
                self.anchor_hit(anchor),
                HitTarget::Resize(anchor),
            ));
        }
        regions.push(HitRegion::new(self.gear(), HitTarget::Gear));
        regions.push(HitRegion::new(
            self.label_hit(Axis::Horizontal, width_chars),
            HitTarget::Copy(Axis::Horizontal),
        ));
        regions.push(HitRegion::new(
            self.label_hit(Axis::Vertical, height_chars),
            HitTarget::Copy(Axis::Vertical),
        ));
        for edge in Edge::ALL {
            regions.push(HitRegion::new(self.line_band(edge), HitTarget::Move));
        }
        regions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hit_region::target_at;

    fn layout() -> FrameLayout {
        FrameLayout::new(FrameSize::INITIAL)
    }

    #[test]
    fn window_wraps_the_frame_and_the_gear() {
        let window = layout().window();
        assert_eq!((window.width(), window.height()), (424, 264));
        assert_eq!(FrameLayout::frame_px_for_window(424), 400);
        assert_eq!(FrameLayout::frame_px_for_window(3), 0);
        let gear = layout().gear();
        assert_eq!(gear.right(), window.right());
        assert_eq!(gear.bottom(), window.bottom());
    }

    #[test]
    fn lines_sit_on_the_frame_border() {
        let frame = layout().frame();
        assert_eq!(frame, PixelRect::new(4, 4, 400, 240));
        assert_eq!(layout().line(Edge::Top), PixelRect::new(4, 4, 400, 1));
        assert_eq!(layout().line(Edge::Bottom), PixelRect::new(4, 243, 400, 1));
        assert_eq!(layout().line(Edge::Left), PixelRect::new(4, 4, 1, 240));
        assert_eq!(layout().line(Edge::Right), PixelRect::new(403, 4, 1, 240));
        assert_eq!(layout().line_band(Edge::Top), PixelRect::new(0, 0, 408, 9));
    }

    #[test]
    fn ticks_point_inward() {
        let top: Vec<PixelRect> = layout().ticks(Edge::Top).collect();
        assert_eq!(top.first(), Some(&PixelRect::new(9, 5, 1, 4)));
        assert!(top.iter().any(|rect| *rect == PixelRect::new(54, 5, 1, 12)));
        let bottom: Vec<PixelRect> = layout().ticks(Edge::Bottom).collect();
        assert_eq!(bottom.first(), Some(&PixelRect::new(9, 239, 1, 4)));
        let left: Vec<PixelRect> = layout().ticks(Edge::Left).collect();
        assert_eq!(left.first(), Some(&PixelRect::new(5, 9, 4, 1)));
        let right: Vec<PixelRect> = layout().ticks(Edge::Right).collect();
        assert_eq!(right.first(), Some(&PixelRect::new(399, 9, 4, 1)));
    }

    #[test]
    fn tick_labels_only_on_top_and_left() {
        let top: Vec<TickLabel> = layout().tick_labels(Edge::Top).collect();
        assert_eq!(top.len(), 7);
        assert_eq!(top.first().map(|label| label.value()), Some(50));
        assert_eq!(
            top.first().map(|label| label.baseline()),
            Some(PixelPoint::new(56, 28))
        );
        let left: Vec<TickLabel> = layout().tick_labels(Edge::Left).collect();
        assert_eq!(left.len(), 4);
        assert_eq!(
            left.first().map(|label| label.baseline()),
            Some(PixelPoint::new(19, 58))
        );
        assert_eq!(layout().tick_labels(Edge::Bottom).count(), 0);
        assert_eq!(layout().tick_labels(Edge::Right).count(), 0);
    }

    #[test]
    fn anchors_hug_the_corners() {
        let [horizontal, vertical] = layout().anchor_arms(Anchor::TopLeft);
        assert_eq!(horizontal, PixelRect::new(4, 4, 12, 2));
        assert_eq!(vertical, PixelRect::new(4, 4, 2, 12));
        let [lower, right] = layout().anchor_arms(Anchor::BottomRight);
        assert_eq!(lower, PixelRect::new(392, 242, 12, 2));
        assert_eq!(right, PixelRect::new(402, 232, 2, 12));
        assert_eq!(
            layout().anchor_hit(Anchor::TopLeft),
            PixelRect::new(0, 0, 16, 16)
        );
        assert_eq!(
            layout().anchor_hit(Anchor::BottomRight),
            PixelRect::new(392, 232, 16, 16)
        );
    }

    #[test]
    fn labels_sit_inside_past_the_tick_numbers() {
        assert_eq!(
            layout().label_baseline(Axis::Horizontal),
            PixelPoint::new(204, 41)
        );
        assert_eq!(
            layout().label_baseline(Axis::Vertical),
            PixelPoint::new(41, 124)
        );
        assert_eq!(
            layout().label_hit(Axis::Horizontal, 6),
            PixelRect::new(183, 30, 42, 13)
        );
        assert_eq!(
            layout().label_hit(Axis::Vertical, 6),
            PixelRect::new(30, 103, 13, 42)
        );
    }

    #[test]
    fn notices_stack_upward_above_the_bottom_ticks() {
        let first = layout().notice_baseline(0);
        let second = layout().notice_baseline(1);
        assert_eq!(first, PixelPoint::new(18, 228));
        assert_eq!(second.y(), first.y() - 12);
        assert!(first.y() < layout().line(Edge::Bottom).y() - TickKind::Major.length_px());
    }

    #[test]
    fn hit_regions_follow_the_priority() {
        let regions = layout().hit_regions(6, 6);
        assert_eq!(
            target_at(&regions, PixelPoint::new(2, 2)),
            Some(HitTarget::Resize(Anchor::TopLeft))
        );
        assert_eq!(
            target_at(&regions, PixelPoint::new(400, 240)),
            Some(HitTarget::Resize(Anchor::BottomRight))
        );
        assert_eq!(
            target_at(&regions, PixelPoint::new(415, 255)),
            Some(HitTarget::Gear)
        );
        assert_eq!(
            target_at(&regions, PixelPoint::new(204, 36)),
            Some(HitTarget::Copy(Axis::Horizontal))
        );
        assert_eq!(
            target_at(&regions, PixelPoint::new(36, 124)),
            Some(HitTarget::Copy(Axis::Vertical))
        );
        assert_eq!(
            target_at(&regions, PixelPoint::new(200, 7)),
            Some(HitTarget::Move)
        );
        assert_eq!(target_at(&regions, PixelPoint::new(200, 120)), None);
    }

    #[test]
    fn ink_rects_cover_lines_ticks_and_anchors() {
        let rects = layout().ink_rects();
        let ticks: usize = Edge::ALL
            .iter()
            .map(|edge| layout().ticks(*edge).count())
            .sum();
        assert_eq!(rects.len(), 4 + ticks + 4);
        assert!(rects.contains(&layout().line(Edge::Right)));
    }
}
