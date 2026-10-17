//! 枠の幾何。線・目盛・鉤括弧・ギア・数字の置き場と当たり判定を、物理画素で計算する。
//!
//! UI はここが返す矩形と点を cairo と X に写すだけで、位置の判断を持たない（ARC-011 / ADR 0005 の 3）。
//! 描く矩形（`ink_rects`）と触れる矩形（`hit_regions`）が同じ `FrameLayout` から出ることで、
//! 「見えるものと触れるもの」の経路を 1 本にする（ARC-001）。
//!
//! 値はすべて SPECIFICATION 第 2 節の「決め打ちしない」1x の初期値である。飾り（帯・鉤括弧・ギア・目盛の長さ・文字）には
//! UI の倍率を掛け、測るもの（枠の大きさ・線と目盛の太さ・目盛の間隔）には掛けない（FR-016・ADR 0011）。

use alloc::string::ToString as _;
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
use crate::ui_scale::UiScale;

/// 線の両側に広げる当たり判定の帯。窓の左上から枠の左上までの余白でもある。
const GRAB_PAD_PX: u16 = 4;
/// 線の太さ。倍率を掛けない。
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
/// 主目盛の数字の高さ。倍率を掛けない（施主指示 2026-10-17・ADR 0013）。
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
/// 主目盛の数字の 1 桁あたりの幅の見積り（9 px の字）。左辺の数字を避けるのに使う。
const TICK_DIGIT_PX: u16 = 6;

/// 枠の大きさから決まる、窓の中の全ての置き場。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameLayout {
    size: FrameSize,
    scale: UiScale,
}

impl FrameLayout {
    /// 枠の大きさと UI の倍率から作る。
    #[must_use]
    pub const fn new(size: FrameSize, scale: UiScale) -> Self {
        Self { size, scale }
    }

    /// UI の倍率。字の大きさを決めるのに UI が読む。
    #[must_use]
    pub const fn scale(self) -> UiScale {
        self.scale
    }

    /// 1x の寸法に倍率を掛ける。
    const fn px(self, base: u16) -> u16 {
        base * self.scale.factor()
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
            self.px(GRAB_PAD_PX) + self.size.width().px() + self.px(BEYOND_PX),
            self.px(GRAB_PAD_PX) + self.size.height().px() + self.px(BEYOND_PX),
        )
    }

    /// 窓の大きさから枠の大きさへ戻す（WM の `ConfigureNotify` を受けたとき）。余白を引く。
    #[must_use]
    pub const fn frame_px_for_window(window_px: u16, scale: UiScale) -> u16 {
        window_px.saturating_sub((GRAB_PAD_PX + BEYOND_PX) * scale.factor())
    }

    /// 枠の矩形。線はこの矩形の外周の画素に乗る。幅 400 なら左の線と右の線の間が 400 画素（両端を含む）。
    #[must_use]
    pub fn frame(self) -> PixelRect {
        PixelRect::new(
            self.px(GRAB_PAD_PX),
            self.px(GRAB_PAD_PX),
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
        let pad = self.px(GRAB_PAD_PX);
        PixelRect::new(
            line.x() - pad,
            line.y() - pad,
            line.width() + 2 * pad,
            line.height() + 2 * pad,
        )
    }

    /// 1 辺の目盛の矩形。線から内側へ伸びる。
    pub fn ticks(self, edge: Edge) -> impl Iterator<Item = PixelRect> {
        let frame = self.frame();
        TickMark::along(self.size.along(edge.axis())).map(move |mark| {
            let reach = self.px(mark.kind().length_px());
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
        let inside = LINE_PX + self.px(TickKind::Major.length_px() + TEXT_GAP_PX);
        let gap = self.px(TEXT_GAP_PX);
        let label_px = TICK_LABEL_PX;
        TickMark::along(self.size.along(edge.axis()))
            .filter(move |mark| mark.kind() == TickKind::Major)
            .filter_map(move |mark| {
                let baseline = match edge {
                    Edge::Top => PixelPoint::new(
                        frame.x() + mark.offset() + gap,
                        frame.y() + inside + label_px,
                    ),
                    Edge::Left => PixelPoint::new(
                        frame.x() + inside,
                        frame.y() + mark.offset() + label_px / 2,
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
        let arm = self.px(ANCHOR_ARM_PX);
        let thickness = self.px(ANCHOR_THICKNESS_PX);
        match anchor {
            Anchor::TopLeft => [
                PixelRect::new(frame.x(), frame.y(), arm, thickness),
                PixelRect::new(frame.x(), frame.y(), thickness, arm),
            ],
            Anchor::BottomRight => [
                PixelRect::new(
                    frame.right() + 1 - arm,
                    frame.bottom() + 1 - thickness,
                    arm,
                    thickness,
                ),
                PixelRect::new(
                    frame.right() + 1 - thickness,
                    frame.bottom() + 1 - arm,
                    thickness,
                    arm,
                ),
            ],
        }
    }

    /// 鉤括弧の当たり判定。腕を囲む正方形に、外側の帯を足したもの。
    #[must_use]
    pub fn anchor_hit(self, anchor: Anchor) -> PixelRect {
        let frame = self.frame();
        let arm = self.px(ANCHOR_ARM_PX);
        let pad = self.px(GRAB_PAD_PX);
        let side = arm + pad;
        match anchor {
            Anchor::TopLeft => PixelRect::new(frame.x() - pad, frame.y() - pad, side, side),
            Anchor::BottomRight => PixelRect::new(
                frame.right() + 1 - arm,
                frame.bottom() + 1 - arm,
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
            frame.right() + 1 + self.px(GEAR_GAP_PX),
            frame.bottom() + 1 + self.px(GEAR_GAP_PX),
            self.px(GEAR_DIAMETER_PX),
            self.px(GEAR_DIAMETER_PX),
        )
    }

    /// 左辺の主目盛の数字が占める幅（最も桁の多い数字）。縦軸の数字と通知はこの右に置く。
    #[must_use]
    pub fn left_numbers_width(self) -> u16 {
        let digits = u16::try_from(self.size.height().px().to_string().len()).unwrap_or(5);
        digits * TICK_DIGIT_PX
    }

    /// 左辺の目盛と数字の右端から、枠の左端までの距離。
    fn left_inset(self) -> u16 {
        LINE_PX
            + self.px(TickKind::Major.length_px() + TEXT_GAP_PX)
            + self.left_numbers_width()
            + self.px(TEXT_GAP_PX)
    }

    /// 幅・高さの数字の置き場。横軸は上辺の目盛の数字の下の中央、縦軸は左辺の目盛の数字の右の中央に 90 度回して置く。
    /// 返す点は、文字の中央のベースライン。
    #[must_use]
    pub fn label_baseline(self, axis: Axis) -> PixelPoint {
        let frame = self.frame();
        match axis {
            Axis::Horizontal => {
                let inside = LINE_PX
                    + TICK_LABEL_PX
                    + self.px(TickKind::Major.length_px() + TEXT_GAP_PX + TEXT_GAP_PX + LABEL_PX);
                PixelPoint::new(frame.x() + frame.width() / 2, frame.y() + inside)
            }
            Axis::Vertical => PixelPoint::new(
                frame.x() + self.left_inset() + self.px(LABEL_PX),
                frame.y() + frame.height() / 2,
            ),
        }
    }

    /// 枠の中に出す通知（FR-014）の `index` 行目のベースライン。下辺の目盛の上に、下から上へ積む。
    #[must_use]
    pub fn notice_baseline(self, index: u16) -> PixelPoint {
        let frame = self.frame();
        let inside = LINE_PX + self.px(TickKind::Major.length_px() + TEXT_GAP_PX);
        PixelPoint::new(
            frame.x() + self.left_inset(),
            frame
                .bottom()
                .saturating_sub(inside + index * self.px(NOTICE_LINE_PX)),
        )
    }

    /// 通知の文字の高さ。
    #[must_use]
    pub const fn notice_px(self) -> u16 {
        self.px(NOTICE_PX)
    }

    /// 幅・高さの数字の当たり判定。文字数から見積もる。
    #[must_use]
    pub fn label_hit(self, axis: Axis, chars: u16) -> PixelRect {
        let baseline = self.label_baseline(axis);
        let span = chars * self.px(LABEL_ADVANCE_PX);
        let half = span / 2;
        let label = self.px(LABEL_PX);
        let depth = self.px(LABEL_PX + TEXT_GAP_PX);
        match axis {
            Axis::Horizontal => PixelRect::new(
                baseline.x().saturating_sub(half),
                baseline.y().saturating_sub(label),
                span,
                depth,
            ),
            Axis::Vertical => PixelRect::new(
                baseline.x().saturating_sub(label),
                baseline.y().saturating_sub(half),
                depth,
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
    use crate::length::Length;

    fn layout() -> FrameLayout {
        FrameLayout::new(FrameSize::INITIAL, UiScale::Single)
    }

    fn doubled() -> FrameLayout {
        FrameLayout::new(FrameSize::INITIAL, UiScale::Double)
    }

    #[test]
    fn window_wraps_the_frame_and_the_gear() {
        let window = layout().window();
        assert_eq!((window.width(), window.height()), (424, 264));
        assert_eq!(FrameLayout::frame_px_for_window(424, UiScale::Single), 400);
        assert_eq!(FrameLayout::frame_px_for_window(3, UiScale::Single), 0);
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
            PixelPoint::new(50, 124)
        );
        assert_eq!(layout().left_numbers_width(), 18);
        assert_eq!(
            layout().label_hit(Axis::Horizontal, 6),
            PixelRect::new(183, 30, 42, 13)
        );
        assert_eq!(
            layout().label_hit(Axis::Vertical, 6),
            PixelRect::new(39, 103, 13, 42)
        );
    }

    #[test]
    fn notices_stack_upward_above_the_bottom_ticks() {
        let first = layout().notice_baseline(0);
        let second = layout().notice_baseline(1);
        assert_eq!(first, PixelPoint::new(39, 228));
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
            target_at(&regions, PixelPoint::new(45, 124)),
            Some(HitTarget::Copy(Axis::Vertical))
        );
        assert_eq!(
            target_at(&regions, PixelPoint::new(200, 7)),
            Some(HitTarget::Move)
        );
        assert_eq!(target_at(&regions, PixelPoint::new(200, 120)), None);
    }

    /// 左辺の主目盛の数字が占める矩形の見積り（字の高さは 1x で 9 px）。
    fn left_number_rects(layout: FrameLayout) -> Vec<PixelRect> {
        let height = 9;
        layout
            .tick_labels(Edge::Left)
            .map(|label| {
                let base = label.baseline();
                PixelRect::new(
                    base.x(),
                    base.y() - height,
                    layout.left_numbers_width(),
                    height,
                )
            })
            .collect()
    }

    fn intersects(a: PixelRect, b: PixelRect) -> bool {
        a.x() <= b.right() && b.x() <= a.right() && a.y() <= b.bottom() && b.y() <= a.bottom()
    }

    #[test]
    fn vertical_label_and_notices_clear_the_left_tick_numbers() {
        for (height, scale) in [240_u16, 270, 1_200]
            .into_iter()
            .flat_map(|height| [(height, UiScale::Single), (height, UiScale::Double)])
        {
            let size = FrameSize::new(Length::INITIAL_WIDTH, Length::new(height).unwrap());
            let layout = FrameLayout::new(size, scale);
            let label = layout.label_hit(Axis::Vertical, 7);
            let notice = {
                let base = layout.notice_baseline(0);
                let line = layout.notice_px();
                PixelRect::new(base.x(), base.y() - line, 120, line)
            };
            for number in left_number_rects(layout) {
                assert!(
                    !intersects(label, number),
                    "height {height}: {label:?} vs {number:?}"
                );
                assert!(
                    !intersects(notice, number),
                    "height {height}: {notice:?} vs {number:?}"
                );
            }
        }
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

    #[test]
    fn double_keeps_the_measure() {
        let (single, double) = (layout(), doubled());
        // 測るもの: 枠の大きさ・線の太さ・目盛の間隔と太さ。
        assert_eq!(double.frame().width(), single.frame().width());
        assert_eq!(double.frame().height(), single.frame().height());
        assert_eq!(double.line(Edge::Top).height(), 1);
        let offsets = |layout: FrameLayout| -> Vec<(u16, u16)> {
            layout
                .ticks(Edge::Top)
                .map(|tick| (tick.x() - layout.frame().x(), tick.width()))
                .collect()
        };
        assert_eq!(offsets(double), offsets(single));
    }

    #[test]
    fn double_scales_the_decorations() {
        let (single, double) = (layout(), doubled());
        // 飾り: 帯・目盛の長さ・鉤括弧・ギア・文字。
        assert_eq!(double.frame(), PixelRect::new(8, 8, 400, 240));
        assert_eq!(double.line_band(Edge::Top), PixelRect::new(0, 0, 416, 17));
        assert_eq!(
            double.ticks(Edge::Top).next(),
            Some(PixelRect::new(13, 9, 1, 8))
        );
        let [arm, _] = double.anchor_arms(Anchor::TopLeft);
        assert_eq!(arm, PixelRect::new(8, 8, 24, 4));
        assert_eq!(double.gear().width(), 2 * single.gear().width());
    }

    #[test]
    fn double_grows_the_window_by_twice_the_margins() {
        let (single, double) = (layout(), doubled());
        assert_eq!(double.window(), PixelRect::new(0, 0, 448, 288));
        assert_eq!(FrameLayout::frame_px_for_window(448, UiScale::Double), 400);
        assert_eq!(double.notice_px(), 2 * single.notice_px());
        // 主目盛の数字は倍率を掛けない（ADR 0013）。
        assert_eq!(double.left_numbers_width(), single.left_numbers_width());
        let first = double
            .tick_labels(Edge::Top)
            .next()
            .map(TickLabel::baseline);
        // 目盛の長さと隙間は 2x、字の高さ 9 px はそのまま: y = 8 + 1 + 2 × (12 + 2) + 9。
        assert_eq!(first, Some(PixelPoint::new(62, 46)));
        let gear = double.gear();
        assert_eq!(gear.right(), double.window().right());
    }
}
