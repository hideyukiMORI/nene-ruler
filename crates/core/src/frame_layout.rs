//! 枠の幾何。線・目盛・鉤括弧・ギア・数字の置き場と当たり判定を、物理画素で計算する。
//!
//! UI はここが返す矩形と点を cairo と X に写すだけで、位置の判断を持たない（ARC-011 / ADR 0005 の 3）。
//! 描く矩形（`ink_rects`）と触れる矩形（`hit_regions`）が同じ `FrameLayout` から出ることで、
//! 「見えるものと触れるもの」の経路を 1 本にする（ARC-001）。
//!
//! 値はすべて SPECIFICATION 第 2 節の「決め打ちしない」1x の初期値である。飾り（帯・鉤括弧・ギア・目盛の長さ）には
//! UI の整数の倍率を掛け、測るもの（枠の大きさ・線と目盛の太さ）には掛けない（FR-016・ADR 0011）。
//! 字の大きさは論理画素で定め、単位に依らず比（dpi / 96）で物理画素に丸める。目盛の間隔は選んだ単位で刻む（ADR 0028）。

use alloc::string::ToString as _;
use alloc::vec::Vec;

use crate::anchor::Anchor;
use crate::edge::Edge;
use crate::frame_detail::FrameDetail;
use crate::frame_size::FrameSize;
use crate::hit_region::HitRegion;
use crate::hit_target::HitTarget;
use crate::length::Length;
use crate::length_label::LengthLabel;
use crate::measure::Measure;
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
/// 線を消した描き方（`Bare`）の鉤括弧の腕の長さと太さ（ADR 0018）。
const BARE_ANCHOR_ARM_PX: u16 = 6;
const BARE_ANCHOR_THICKNESS_PX: u16 = 1;
/// ギアの直径。
const GEAR_DIAMETER_PX: u16 = 16;
/// 右下の鉤括弧からギアまでの隙間。
const GEAR_GAP_PX: u16 = 4;
/// 枠の右下の外側に要る余白。ギアが収まる分。
const BEYOND_PX: u16 = GEAR_GAP_PX + GEAR_DIAMETER_PX;
/// 主目盛の数字の高さ。論理画素で、単位に依らず比で物理画素にする（施主指示 2026-10-20・ADR 0028）。
const TICK_LABEL_LOGICAL_PX: u16 = 9;
/// 幅・高さの数字の高さ。論理画素（ADR 0028）。
const LABEL_LOGICAL_PX: u16 = 11;
/// 幅・高さの数字の 1 文字あたりの幅の見積り（当たり判定だけに使う）。論理画素。
const LABEL_ADVANCE_LOGICAL_PX: u16 = 7;
/// 目盛の先端から文字までの隙間。
const TEXT_GAP_PX: u16 = 2;
/// 通知の文字の高さと行の間隔。論理画素（ADR 0028）。
const NOTICE_LOGICAL_PX: u16 = 9;
const NOTICE_LINE_LOGICAL_PX: u16 = 12;
/// 主目盛の数字の 1 桁あたりの幅の見積り（9 論理画素の字）。左辺の数字を避けるのに使う。論理画素。
const TICK_DIGIT_LOGICAL_PX: u16 = 6;

/// 枠の大きさから決まる、窓の中の全ての置き場。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameLayout {
    size: FrameSize,
    scale: UiScale,
    measure: Measure,
    detail: FrameDetail,
}

impl FrameLayout {
    /// 枠の大きさ・UI の倍率・単位と比から作る。
    #[must_use]
    pub const fn new(size: FrameSize, scale: UiScale, measure: Measure) -> Self {
        Self {
            size,
            scale,
            measure,
            detail: FrameDetail::Full,
        }
    }

    /// 幅と高さの数字が、目盛と目盛の数字を避けた内側に収まるかで描き方を決める（ADR 0017）。
    /// `chars` は数字の種類ごとの文字数（当たり判定と同じ見積り）。
    #[must_use]
    pub fn fitted<F: Fn(LengthLabel) -> u16>(self, chars: F) -> Self {
        let full = Self {
            detail: FrameDetail::Full,
            ..self
        };
        let compact = Self {
            detail: FrameDetail::Compact,
            ..self
        };
        let size_width = compact
            .label_hit(LengthLabel::Size, chars(LengthLabel::Size))
            .width();
        let detail = if full.labels_fit(&chars) {
            FrameDetail::Full
        } else if size_width + 2 <= compact.frame().width() {
            FrameDetail::Compact
        } else {
            FrameDetail::Bare
        };
        Self { detail, ..self }
    }

    /// 描き方。
    #[must_use]
    pub const fn detail(self) -> FrameDetail {
        self.detail
    }

    /// いまの描き方で出す数字。
    #[must_use]
    pub const fn labels(self) -> &'static [LengthLabel] {
        match self.detail {
            FrameDetail::Full => &[LengthLabel::Width, LengthLabel::Height],
            FrameDetail::Compact | FrameDetail::Bare => &[LengthLabel::Size],
        }
    }

    /// 全部描いたとき、幅と高さの数字が、左辺と上辺の数字・右辺と下辺の主目盛を避けた内側に収まり、互いに重ならないか。
    /// 高さの数字は上辺の数字の帯の下から始まればよく、幅の数字とは矩形が交わらなければよい（横に離れていれば重ならない・ADR 0028）。
    fn labels_fit(self, chars: &impl Fn(LengthLabel) -> u16) -> bool {
        let frame = self.frame();
        let left = frame.x() + self.left_inset();
        let reach = self.px(TickKind::Major.length_px());
        let right = frame.right().saturating_sub(reach);
        let bottom = frame.bottom().saturating_sub(reach);
        let top_numbers = self
            .tick_band(Edge::Top)
            .map_or(frame.y(), PixelRect::bottom);
        let width = self.label_hit(LengthLabel::Width, chars(LengthLabel::Width));
        let height = self.label_hit(LengthLabel::Height, chars(LengthLabel::Height));
        width.x() >= left
            && width.right() < right
            && width.bottom() < bottom
            && height.y() > top_numbers
            && !overlaps(width, height)
            && height.right() < right
            && height.bottom() < bottom
    }

    /// UI の倍率。飾りの太さを決めるのに UI が読む。
    #[must_use]
    pub const fn scale(self) -> UiScale {
        self.scale
    }

    /// 単位と比。
    #[must_use]
    pub const fn measure(self) -> Measure {
        self.measure
    }

    /// 1x の寸法に倍率を掛ける（飾り）。
    const fn px(self, base: u16) -> u16 {
        base * self.scale.factor()
    }

    /// 論理画素の寸法を比で物理画素にする（字）。単位の選択には依らない（ADR 0028）。
    fn text_px(self, logical: u16) -> u16 {
        self.measure.ratio().physical(logical)
    }

    /// 主目盛の数字の字の高さ（物理画素）。UI はこれで字の大きさを決める。
    #[must_use]
    pub fn tick_label_px(self) -> u16 {
        self.text_px(TICK_LABEL_LOGICAL_PX)
    }

    /// 幅・高さの数字の字の高さ（物理画素）。
    #[must_use]
    pub fn label_px(self) -> u16 {
        self.text_px(LABEL_LOGICAL_PX)
    }

    /// 幅・高さの数字の 1 文字あたりの幅の見積り（物理画素）。
    fn label_advance_px(self) -> u16 {
        self.text_px(LABEL_ADVANCE_LOGICAL_PX)
    }

    /// 主目盛の数字の 1 桁あたりの幅の見積り（物理画素）。
    fn tick_digit_px(self) -> u16 {
        self.text_px(TICK_DIGIT_LOGICAL_PX)
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

    /// 枠が最小（`Length::MIN_PX`）のときの窓の一辺。WM に伝える最小の大きさと、自前のドラッグの下限（Issue #52）。
    #[must_use]
    pub const fn smallest_window(scale: UiScale) -> u16 {
        (GRAB_PAD_PX + BEYOND_PX) * scale.factor() + Length::MIN_PX
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

    /// 1 辺の目盛の帯。線から主目盛の先まで、上辺と左辺は目盛の数字まで。目盛の上ならどこでも掴める（ADR 0022）。
    /// 目盛を描かない描き方では無い。
    #[must_use]
    pub fn tick_band(self, edge: Edge) -> Option<PixelRect> {
        if self.detail != FrameDetail::Full {
            return None;
        }
        let frame = self.frame();
        let reach = self.px(TickKind::Major.length_px());
        let numbered = LINE_PX + self.px(TickKind::Major.length_px() + TEXT_GAP_PX);
        Some(match edge {
            Edge::Top => PixelRect::new(
                frame.x(),
                frame.y(),
                frame.width(),
                numbered + self.tick_label_px(),
            ),
            Edge::Bottom => PixelRect::new(frame.x(), frame.bottom() - reach, frame.width(), reach),
            Edge::Left => PixelRect::new(
                frame.x(),
                frame.y(),
                numbered + self.left_numbers_width(),
                frame.height(),
            ),
            Edge::Right => PixelRect::new(frame.right() - reach, frame.y(), reach, frame.height()),
        })
    }

    /// 1 辺の目盛の矩形。線から内側へ伸びる。
    pub fn ticks(self, edge: Edge) -> impl Iterator<Item = PixelRect> {
        let frame = self.frame();
        let drawn = self.detail == FrameDetail::Full;
        TickMark::along(self.size.along(edge.axis()), self.measure)
            .filter(move |_| drawn)
            .map(move |mark| {
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
        let label_px = self.tick_label_px();
        let digit_px = self.tick_digit_px();
        let drawn = self.detail == FrameDetail::Full;
        TickMark::along(self.size.along(edge.axis()), self.measure)
            .filter(move |mark| drawn && mark.kind() == TickKind::Major)
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
                // 枠の外へはみ出す数字は置かない（Issue #46）。上辺は字の右端、左辺はベースライン。
                let inside_frame = match edge {
                    Edge::Top => baseline.x() + digits(mark.value()) * digit_px <= frame.right(),
                    Edge::Left | Edge::Bottom | Edge::Right => baseline.y() < frame.bottom(),
                };
                inside_frame.then(|| TickLabel::new(mark.value(), baseline))
            })
    }

    /// 鉤括弧を作る 2 本の腕。
    #[must_use]
    pub fn anchor_arms(self, anchor: Anchor) -> [PixelRect; 2] {
        let frame = self.frame();
        let (arm, thickness) = match self.detail {
            FrameDetail::Full | FrameDetail::Compact => {
                (self.px(ANCHOR_ARM_PX), self.px(ANCHOR_THICKNESS_PX))
            }
            // 細く描くだけで、当たり判定（`anchor_hit`）は変えない（ADR 0018）。
            FrameDetail::Bare => (
                self.px(BARE_ANCHOR_ARM_PX),
                self.px(BARE_ANCHOR_THICKNESS_PX),
            ),
        };
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
    /// 桁は選んだ単位での高さの値で数える（論理画素なら 150% の高さ 240 は 160）。
    #[must_use]
    pub fn left_numbers_width(self) -> u16 {
        let largest = self.measure.tenths(self.size.height()) / 10;
        digits(u16::try_from(largest).unwrap_or(u16::MAX)) * self.tick_digit_px()
    }

    /// 左辺の目盛と数字の右端から、枠の左端までの距離。
    fn left_inset(self) -> u16 {
        LINE_PX
            + self.px(TickKind::Major.length_px() + TEXT_GAP_PX)
            + self.left_numbers_width()
            + self.px(TEXT_GAP_PX)
    }

    /// 数字の置き場。幅は上辺の目盛の数字の下の中央、高さは左辺の目盛の数字の右の中央に 90 度回して、
    /// 両方（詰めた表示）は枠の中央に置く。
    /// 返す点は、文字の中央のベースライン。
    #[must_use]
    pub fn label_baseline(self, label: LengthLabel) -> PixelPoint {
        let frame = self.frame();
        match label {
            LengthLabel::Width => {
                let inside = LINE_PX
                    + self.tick_label_px()
                    + self.label_px()
                    + self.px(TickKind::Major.length_px() + TEXT_GAP_PX + TEXT_GAP_PX);
                PixelPoint::new(frame.x() + frame.width() / 2, frame.y() + inside)
            }
            LengthLabel::Height => PixelPoint::new(
                frame.x() + self.left_inset() + self.label_px(),
                frame.y() + frame.height() / 2,
            ),
            LengthLabel::Size => {
                // 線を消した描き方では、枠より広い文字を窓の横の中央に寄せて余白へはみ出させる（ADR 0018）。
                let centre_x = match self.detail {
                    FrameDetail::Full | FrameDetail::Compact => frame.x() + frame.width() / 2,
                    FrameDetail::Bare => self.window().width() / 2,
                };
                PixelPoint::new(
                    centre_x,
                    frame.y() + frame.height() / 2 + self.label_px() / 2,
                )
            }
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
                .saturating_sub(inside + index * self.text_px(NOTICE_LINE_LOGICAL_PX)),
        )
    }

    /// 通知の文字の高さ（物理画素）。
    #[must_use]
    pub fn notice_px(self) -> u16 {
        self.text_px(NOTICE_LOGICAL_PX)
    }

    /// 数字の当たり判定。文字数から見積もる。
    #[must_use]
    pub fn label_hit(self, label: LengthLabel, chars: u16) -> PixelRect {
        let baseline = self.label_baseline(label);
        let estimate = chars * self.label_advance_px();
        // 線を消した描き方では、見積りが窓より広くても窓の中に収める（ADR 0018）。
        let span = match self.detail {
            FrameDetail::Full | FrameDetail::Compact => estimate,
            FrameDetail::Bare => estimate.min(self.window().width()),
        };
        let half = span / 2;
        let label_px = self.label_px();
        let depth = label_px + self.px(TEXT_GAP_PX);
        match label {
            LengthLabel::Width | LengthLabel::Size => PixelRect::new(
                baseline.x().saturating_sub(half),
                baseline.y().saturating_sub(label_px),
                span,
                depth,
            ),
            LengthLabel::Height => PixelRect::new(
                baseline.x().saturating_sub(label_px),
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
            if self.detail != FrameDetail::Bare {
                rects.push(self.line(edge));
            }
            rects.extend(self.ticks(edge));
        }
        for anchor in [Anchor::TopLeft, Anchor::BottomRight] {
            let [horizontal, vertical] = self.anchor_arms(anchor);
            rects.push(horizontal);
            // 線を消した表示では縦の腕を描かない。短い形の字に重なるので（施主指示 2026-10-17・Issue #56）。
            if self.detail != FrameDetail::Bare {
                rects.push(vertical);
            }
        }
        rects
    }

    /// 当たり判定を優先順に。鉤括弧 → ギア → 数字 → 線の帯 → 目盛の帯。
    #[must_use]
    pub fn hit_regions<F: Fn(LengthLabel) -> u16>(self, chars: F) -> Vec<HitRegion> {
        let mut regions = Vec::new();
        for anchor in [Anchor::TopLeft, Anchor::BottomRight] {
            regions.push(HitRegion::new(
                self.anchor_hit(anchor),
                HitTarget::Resize(anchor),
            ));
        }
        regions.push(HitRegion::new(self.gear(), HitTarget::Gear));
        for label in self.labels() {
            regions.push(HitRegion::new(
                self.label_hit(*label, chars(*label)),
                HitTarget::Copy(*label),
            ));
        }
        for edge in Edge::ALL {
            regions.push(HitRegion::new(self.line_band(edge), HitTarget::Move));
        }
        for band in Edge::ALL
            .into_iter()
            .filter_map(|edge| self.tick_band(edge))
        {
            regions.push(HitRegion::new(band, HitTarget::Move));
        }
        regions
    }
}

/// 2 つの矩形が 1 画素でも共有するか。
fn overlaps(a: PixelRect, b: PixelRect) -> bool {
    a.x() <= b.right() && b.x() <= a.right() && a.y() <= b.bottom() && b.y() <= a.bottom()
}

/// 10 進の桁数。
fn digits(value: u16) -> u16 {
    u16::try_from(value.to_string().len()).unwrap_or(5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_pixel_ratio::DevicePixelRatio;
    use crate::hit_region::target_at;
    use crate::length::Length;
    use crate::length_text::LengthText;
    use crate::unit::Unit;

    fn layout() -> FrameLayout {
        FrameLayout::new(FrameSize::INITIAL, UiScale::Single, Measure::PHYSICAL)
    }

    /// 192 dpi の 2x。飾りは 2 倍、字も比で 2 倍。
    fn doubled() -> FrameLayout {
        FrameLayout::new(
            FrameSize::INITIAL,
            UiScale::Double,
            at_dpi(Unit::Physical, 192),
        )
    }

    fn at_dpi(unit: Unit, dpi: u32) -> Measure {
        Measure::new(unit, DevicePixelRatio::from_dpi(Some(dpi)))
    }

    /// 144 dpi（150%）の 2x。字は 1.5 倍で、論理画素なら目盛も 1.5 倍の間隔。
    fn half_again(unit: Unit) -> FrameLayout {
        FrameLayout::new(FrameSize::INITIAL, UiScale::Double, at_dpi(unit, 144))
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
            layout().label_baseline(LengthLabel::Width),
            PixelPoint::new(204, 41)
        );
        assert_eq!(
            layout().label_baseline(LengthLabel::Height),
            PixelPoint::new(50, 124)
        );
        assert_eq!(layout().left_numbers_width(), 18);
        assert_eq!(
            layout().label_hit(LengthLabel::Width, 6),
            PixelRect::new(183, 30, 42, 13)
        );
        assert_eq!(
            layout().label_hit(LengthLabel::Height, 6),
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
        let regions = layout().hit_regions(|_label| 6);
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
            Some(HitTarget::Copy(LengthLabel::Width))
        );
        assert_eq!(
            target_at(&regions, PixelPoint::new(45, 124)),
            Some(HitTarget::Copy(LengthLabel::Height))
        );
        assert_eq!(
            target_at(&regions, PixelPoint::new(200, 7)),
            Some(HitTarget::Move)
        );
        assert_eq!(target_at(&regions, PixelPoint::new(200, 120)), None);
    }

    #[test]
    fn tick_bands_cover_the_ticks_and_their_numbers() {
        assert_eq!(
            layout().tick_band(Edge::Top),
            Some(PixelRect::new(4, 4, 400, 24))
        );
        assert_eq!(
            layout().tick_band(Edge::Bottom),
            Some(PixelRect::new(4, 231, 400, 12))
        );
        assert_eq!(
            layout().tick_band(Edge::Left),
            Some(PixelRect::new(4, 4, 33, 240))
        );
        assert_eq!(
            layout().tick_band(Edge::Right),
            Some(PixelRect::new(391, 4, 12, 240))
        );
        assert_eq!(
            doubled().tick_band(Edge::Top),
            Some(PixelRect::new(8, 8, 400, 47))
        );
        for edge in Edge::ALL {
            let band = layout().tick_band(edge);
            assert!(layout().ticks(edge).all(|tick| band.is_some_and(|band| {
                band.contains(PixelPoint::new(tick.x(), tick.y()))
                    && band.contains(PixelPoint::new(tick.right(), tick.bottom()))
            })));
        }
    }

    #[test]
    fn tick_bands_move_between_the_ticks_but_yield_to_the_rest() {
        let regions = layout().hit_regions(|_label| 6);
        // 上辺の目盛の隙間と、目盛の数字の隙間。
        for point in [PixelPoint::new(61, 12), PixelPoint::new(61, 24)] {
            assert_eq!(target_at(&regions, point), Some(HitTarget::Move));
        }
        assert_eq!(
            target_at(&regions, PixelPoint::new(20, 130)),
            Some(HitTarget::Move)
        );
        assert_eq!(
            target_at(&regions, PixelPoint::new(395, 130)),
            Some(HitTarget::Move)
        );
        assert_eq!(
            target_at(&regions, PixelPoint::new(200, 236)),
            Some(HitTarget::Move)
        );
        assert_eq!(
            target_at(&regions, PixelPoint::new(10, 10)),
            Some(HitTarget::Resize(Anchor::TopLeft))
        );
        assert_eq!(target_at(&regions, PixelPoint::new(200, 29)), None);
        assert_eq!(target_at(&regions, PixelPoint::new(38, 130)), None);
    }

    /// 左辺の主目盛の数字が占める矩形の見積り（字の高さは 9 論理画素）。
    fn left_number_rects(layout: FrameLayout) -> Vec<PixelRect> {
        let height = layout.tick_label_px();
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

    #[test]
    fn vertical_label_and_notices_clear_the_left_tick_numbers() {
        let setups = [
            (UiScale::Single, Measure::PHYSICAL),
            (UiScale::Double, at_dpi(Unit::Physical, 192)),
            (UiScale::Double, at_dpi(Unit::Physical, 144)),
            (UiScale::Double, at_dpi(Unit::Logical, 144)),
            (UiScale::Single, at_dpi(Unit::Logical, 120)),
        ];
        for (height, (scale, measure)) in [240_u16, 270, 1_200]
            .into_iter()
            .flat_map(|height| setups.into_iter().map(move |setup| (height, setup)))
        {
            let size = FrameSize::new(Length::INITIAL_WIDTH, Length::new(height).unwrap());
            let layout = FrameLayout::new(size, scale, measure);
            let label = layout.label_hit(LengthLabel::Height, 7);
            let notice = {
                let base = layout.notice_baseline(0);
                let line = layout.notice_px();
                PixelRect::new(base.x(), base.y() - line, 120, line)
            };
            for number in left_number_rects(layout) {
                assert!(
                    !overlaps(label, number),
                    "height {height}: {label:?} vs {number:?}"
                );
                assert!(
                    !overlaps(notice, number),
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
        let gear = double.gear();
        assert_eq!(gear.right(), double.window().right());
    }

    #[test]
    fn at_192_dpi_the_text_is_twice_as_tall() {
        let (single, double) = (layout(), doubled());
        // 字は 192 dpi の比で 2 倍（ADR 0028）。
        assert_eq!(double.tick_label_px(), 18);
        assert_eq!(double.label_px(), 22);
        assert_eq!(double.left_numbers_width(), 2 * single.left_numbers_width());
        let first = double
            .tick_labels(Edge::Top)
            .next()
            .map(TickLabel::baseline);
        // 目盛の長さと隙間は 2x、字の高さは 18: y = 8 + 1 + 2 × (12 + 2) + 18。
        assert_eq!(first, Some(PixelPoint::new(62, 55)));
        // 幅と高さの数字は 22: y = 8 + 1 + 18 + 22 + 2 × (12 + 2 + 2)。当たり判定も 1 文字 14 で見積もる。
        assert_eq!(
            double.label_baseline(LengthLabel::Width),
            PixelPoint::new(208, 81)
        );
        assert_eq!(
            double.label_hit(LengthLabel::Width, 6),
            PixelRect::new(166, 59, 84, 26)
        );
    }

    #[test]
    fn text_follows_the_ratio_whichever_unit_is_chosen() {
        for unit in Unit::ALL {
            let layout = half_again(unit);
            assert_eq!(layout.tick_label_px(), 14, "{unit:?}");
            assert_eq!(layout.label_px(), 17, "{unit:?}");
            assert_eq!(layout.notice_px(), 14, "{unit:?}");
            assert_eq!(layout.label_hit(LengthLabel::Width, 6).width(), 66);
            let plain = FrameLayout::new(FrameSize::INITIAL, UiScale::Single, at_dpi(unit, 96));
            assert_eq!(
                (plain.tick_label_px(), plain.label_px(), plain.notice_px()),
                (9, 11, 9)
            );
        }
        assert_eq!(layout().measure(), Measure::PHYSICAL);
    }

    #[test]
    fn logical_pixels_space_the_ticks_by_the_ratio() {
        let logical = half_again(Unit::Logical);
        // 5 論理画素 = 7.5 物理画素。最初の目盛は 8、主目盛は 75 ごと。長さは 2x の飾り。
        let top: Vec<PixelRect> = logical.ticks(Edge::Top).collect();
        assert_eq!(top.first(), Some(&PixelRect::new(16, 9, 1, 8)));
        assert!(top.contains(&PixelRect::new(83, 9, 1, 24)));
        assert!(!top.contains(&PixelRect::new(58, 9, 1, 24)));
        // 数字は論理画素の値。幅 400 は 266.7 論理画素で、250 は字が右の線を越えるので置かない。
        let values: Vec<(u16, PixelPoint)> = logical
            .tick_labels(Edge::Top)
            .map(|label| (label.value(), label.baseline()))
            .collect();
        assert_eq!(
            values.iter().map(|(value, _)| *value).collect::<Vec<_>>(),
            [50, 100, 150, 200]
        );
        // x = 8 + 75 + 4、y = 8 + 1 + 2 × (12 + 2) + 14。
        assert_eq!(
            values.first().map(|(_, at)| *at),
            Some(PixelPoint::new(87, 51))
        );
        let left: Vec<u16> = logical
            .tick_labels(Edge::Left)
            .map(TickLabel::value)
            .collect();
        assert_eq!(left, [50, 100, 150]);
        // 左辺の数字の幅は 160（高さ 240 の論理画素）の 3 桁 × 9。
        assert_eq!(logical.left_numbers_width(), 27);
    }

    #[test]
    fn physical_pixels_keep_the_ticks_at_their_places_whatever_the_dpi() {
        // 物理画素を選べば、同じ dpi でも目盛は 5 物理画素ごとで値は位置そのもの。
        let physical = half_again(Unit::Physical);
        assert_eq!(
            physical.ticks(Edge::Top).next(),
            Some(PixelRect::new(13, 9, 1, 8))
        );
        assert!(
            physical
                .tick_labels(Edge::Top)
                .all(|label| label.value() % 50 == 0)
        );
        assert_eq!(physical.tick_labels(Edge::Top).count(), 7);
    }

    #[test]
    fn tick_numbers_that_would_leave_the_frame_are_dropped() {
        let numbers = |width: u16, height: u16, edge: Edge| -> Vec<u16> {
            let size = FrameSize::new(Length::new(width).unwrap(), Length::new(height).unwrap());
            FrameLayout::new(size, UiScale::Single, Measure::PHYSICAL)
                .tick_labels(edge)
                .map(TickLabel::value)
                .collect()
        };
        // 幅 360: 350 は x = 4 + 350 + 2 から 3 桁（18 px）で右の線（363）を越える。
        assert_eq!(numbers(360, 240, Edge::Top).last(), Some(&300));
        assert_eq!(numbers(400, 240, Edge::Top).last(), Some(&350));
        // 高さ 252: 250 のベースラインは y = 4 + 250 + 4 で下の線（255）を越える。
        assert_eq!(numbers(400, 252, Edge::Left).last(), Some(&200));
        assert_eq!(numbers(400, 260, Edge::Left).last(), Some(&250));
    }

    /// 文字数の見積りを、表示する文字列（`LengthText`）の長さで作る。
    fn chars_for(width: u16, height: u16) -> impl Fn(LengthLabel) -> u16 {
        let (width, height) = (Length::new(width).unwrap(), Length::new(height).unwrap());
        move |label| match label {
            LengthLabel::Width => LengthText::single(width, Measure::PHYSICAL).chars(),
            LengthLabel::Height => LengthText::single(height, Measure::PHYSICAL).chars(),
            LengthLabel::Size => LengthText::size(width, height, Measure::PHYSICAL).chars(),
        }
    }

    fn fitted(width: u16, height: u16) -> FrameLayout {
        let size = FrameSize::new(Length::new(width).unwrap(), Length::new(height).unwrap());
        FrameLayout::new(size, UiScale::Single, Measure::PHYSICAL).fitted(chars_for(width, height))
    }

    /// 144 dpi の論理画素で、表示する文字列の長さから描き方を決める。
    fn fitted_logical(width: u16, height: u16) -> FrameLayout {
        let (w, h) = (Length::new(width).unwrap(), Length::new(height).unwrap());
        let measure = at_dpi(Unit::Logical, 144);
        FrameLayout::new(FrameSize::new(w, h), UiScale::Double, measure).fitted(move |label| {
            match label {
                LengthLabel::Width => LengthText::single(w, measure).chars(),
                LengthLabel::Height => LengthText::single(h, measure).chars(),
                LengthLabel::Size => LengthText::size(w, h, measure).chars(),
            }
        })
    }

    #[test]
    fn logical_labels_at_150_percent_still_fit_the_initial_frame() {
        // `160 logical px`（14 文字 × 11）は縦 154 px で、上辺の数字の帯（y ≦ 50）のすぐ下の 51 から始まる。
        // 幅の数字（x 120〜295）とは横に離れているので重ならない。
        let initial = fitted_logical(400, 240);
        assert_eq!(initial.detail(), FrameDetail::Full);
        let height = initial.label_hit(LengthLabel::Height, 14);
        assert_eq!(height.y(), 51);
        assert_eq!(
            initial.tick_band(Edge::Top).map(PixelRect::bottom),
            Some(50)
        );
        assert!(!overlaps(height, initial.label_hit(LengthLabel::Width, 16)));
        assert_eq!(fitted_logical(500, 280).detail(), FrameDetail::Full);
        // 1 px 低いと上辺の数字に触れるので詰める。幅 300 では幅の数字（x 81〜234）が高さの数字（x 68〜88・y 51〜）に重なるので詰める。
        assert_eq!(fitted_logical(400, 239).detail(), FrameDetail::Compact);
        assert_eq!(fitted_logical(300, 240).detail(), FrameDetail::Compact);
    }

    #[test]
    fn small_frames_switch_to_the_compact_detail() {
        assert_eq!(fitted(400, 240).detail(), FrameDetail::Full);
        assert_eq!(fitted(100, 240).detail(), FrameDetail::Compact);
        assert_eq!(fitted(400, 60).detail(), FrameDetail::Compact);
        // 境目: 幅を 1 px ずつ縮めると、全部 → 詰める → 線を消す の順に一度ずつ変わり、戻らない。
        let changes = (16..=400_u16)
            .map(|width| fitted(width, 240).detail())
            .collect::<Vec<_>>()
            .windows(2)
            .filter(|pair| pair.first() != pair.last())
            .count();
        assert_eq!(changes, 2);
    }

    #[test]
    fn the_compact_detail_draws_only_lines_anchors_and_the_size() {
        let compact = fitted(100, 40);
        assert_eq!(compact.labels(), &[LengthLabel::Size]);
        for edge in Edge::ALL {
            assert_eq!(compact.ticks(edge).count(), 0);
            assert_eq!(compact.tick_labels(edge).count(), 0);
        }
        assert_eq!(compact.ink_rects().len(), 4 + 4);
        assert!(
            Edge::ALL
                .iter()
                .all(|edge| compact.tick_band(*edge).is_none())
        );
        let regions = compact.hit_regions(chars_for(100, 40));
        let centre = PixelPoint::new(4 + 50, 4 + 20);
        assert_eq!(
            target_at(&regions, centre),
            Some(HitTarget::Copy(LengthLabel::Size))
        );
        assert!(
            regions
                .iter()
                .all(|region| region.target() != HitTarget::Copy(LengthLabel::Width))
        );
    }

    #[test]
    fn frames_too_small_for_the_size_drop_their_lines() {
        assert_eq!(fitted(102, 52).detail(), FrameDetail::Compact);
        assert_eq!(fitted(16, 16).detail(), FrameDetail::Bare);
        assert_eq!(fitted(60, 40).detail(), FrameDetail::Bare);
        let bare = fitted(16, 16);
        let compact = fitted(102, 52);
        // 線のインクが無く、鉤括弧は横の細い腕 2 本だけ（Issue #56）。
        let [top_left_arm, _] = bare.anchor_arms(Anchor::TopLeft);
        let [bottom_right_arm, _] = bare.anchor_arms(Anchor::BottomRight);
        assert_eq!(bare.ink_rects(), [top_left_arm, bottom_right_arm]);
        let [arm, _] = bare.anchor_arms(Anchor::TopLeft);
        assert_eq!((arm.width(), arm.height()), (6, 1));
        // 掴める範囲は変えない。
        assert_eq!(
            bare.anchor_hit(Anchor::TopLeft),
            compact.anchor_hit(Anchor::TopLeft)
        );
    }

    #[test]
    fn the_short_size_fits_the_smallest_window() {
        let bare = fitted(16, 16);
        // 文字は窓の横の中央に置く。短い形（`16×16`・5 文字）は 1x の最小の窓（40 px）に収まる。
        let short = LengthText::short_size(
            Length::new(16).unwrap(),
            Length::new(16).unwrap(),
            Measure::PHYSICAL,
        );
        let hit = bare.label_hit(LengthLabel::Size, short.chars());
        assert_eq!(hit.width(), 35);
        assert_eq!(
            bare.label_baseline(LengthLabel::Size).x(),
            bare.window().width() / 2
        );
        assert!(hit.right() <= bare.window().right(), "{hit:?}");
        assert_eq!(bare.labels(), &[LengthLabel::Size]);
    }

    #[test]
    fn the_smallest_window_holds_the_smallest_frame() {
        assert_eq!(FrameLayout::smallest_window(UiScale::Single), 40);
        assert_eq!(FrameLayout::smallest_window(UiScale::Double), 64);
        assert_eq!(
            FrameLayout::frame_px_for_window(
                FrameLayout::smallest_window(UiScale::Double),
                UiScale::Double
            ),
            Length::MIN_PX
        );
    }
}
