//! 線の上の目盛 1 つ。

use crate::length::Length;
use crate::measure::Measure;
use crate::tick_kind::TickKind;

/// 選んだ単位での値と、原点からの物理画素の距離と、種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickMark {
    value: u16,
    offset: u16,
    kind: TickKind,
}

impl TickMark {
    /// 一辺の長さに沿った目盛を原点側から順に返す。
    ///
    /// 目盛は選んだ単位で `TickKind::SPACING_PX` ごとに刻み、位置は物理画素に丸める（FR-003・ADR 0028）。
    /// 原点（0）は鉤括弧の下に隠れるので含めない。終端の画素は線の反対側の角なので含めない。
    pub fn along(length: Length, measure: Measure) -> impl Iterator<Item = Self> {
        (1_u16..)
            .map(move |step| {
                let value = step.saturating_mul(TickKind::SPACING_PX);
                Self {
                    value,
                    offset: measure.physical(value),
                    kind: TickKind::at(value),
                }
            })
            .take_while(move |mark| mark.offset < length.px())
    }

    /// 選んだ単位での原点からの距離。主目盛の数字はこれ。
    #[must_use]
    pub const fn value(self) -> u16 {
        self.value
    }

    /// 原点からの物理画素の距離。
    #[must_use]
    pub const fn offset(self) -> u16 {
        self.offset
    }

    /// 種類。
    #[must_use]
    pub const fn kind(self) -> TickKind {
        self.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_pixel_ratio::DevicePixelRatio;
    use crate::unit::Unit;
    use alloc::vec::Vec;

    #[test]
    fn walks_from_five_to_just_before_the_end() {
        let length = Length::new(60).unwrap();
        let offsets: Vec<u16> = TickMark::along(length, Measure::PHYSICAL)
            .map(TickMark::offset)
            .collect();
        assert_eq!(offsets, [5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55]);
        assert!(
            TickMark::along(length, Measure::PHYSICAL).all(|mark| mark.value() == mark.offset())
        );
    }

    #[test]
    fn carries_the_kind() {
        let length = Length::new(60).unwrap();
        let major: Vec<u16> = TickMark::along(length, Measure::PHYSICAL)
            .filter(|mark| mark.kind() == TickKind::Major)
            .map(TickMark::offset)
            .collect();
        assert_eq!(major, [50]);
    }

    #[test]
    fn logical_pixels_space_the_ticks_by_the_ratio_and_round_the_position() {
        let logical = Measure::new(Unit::Logical, DevicePixelRatio::from_dpi(Some(144)));
        let length = Length::new(60).unwrap();
        let marks: Vec<(u16, u16)> = TickMark::along(length, logical)
            .map(|mark| (mark.value(), mark.offset()))
            .collect();
        // 5 論理画素 = 7.5 物理画素。位置は四捨五入で 8 / 15 / 23 / 30 …、値は論理画素のまま。
        assert_eq!(
            marks,
            [
                (5, 8),
                (10, 15),
                (15, 23),
                (20, 30),
                (25, 38),
                (30, 45),
                (35, 53)
            ]
        );
        // 種類は論理画素の値で決まる: 150% の主目盛は物理 75 px ごと。
        let major: Vec<u16> = TickMark::along(Length::new(200).unwrap(), logical)
            .filter(|mark| mark.kind() == TickKind::Major)
            .map(TickMark::offset)
            .collect();
        assert_eq!(major, [75, 150]);
        // 物理画素を選べば比があっても今までどおり。
        let physical = Measure::new(Unit::Physical, DevicePixelRatio::from_dpi(Some(144)));
        assert_eq!(TickMark::along(length, physical).count(), 11);
    }
}
