//! 線の上の目盛 1 つ。

use crate::length::Length;
use crate::tick_kind::TickKind;

/// 原点からの距離と種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickMark {
    offset: u16,
    kind: TickKind,
}

impl TickMark {
    /// 一辺の長さに沿った目盛を原点側から順に返す。
    ///
    /// 原点（0）は鉤括弧の下に隠れるので含めない。終端の画素は線の反対側の角なので含めない。
    pub fn along(length: Length) -> impl Iterator<Item = Self> {
        (0_u16..length.px())
            .step_by(usize::from(TickKind::SPACING_PX))
            .skip(1_usize)
            .map(|offset| Self {
                offset,
                kind: TickKind::at(offset),
            })
    }

    /// 原点からの距離。
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
    use alloc::vec::Vec;

    #[test]
    fn walks_from_five_to_just_before_the_end() {
        let length = Length::new(60).unwrap();
        let offsets: Vec<u16> = TickMark::along(length).map(TickMark::offset).collect();
        assert_eq!(offsets, [5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55]);
    }

    #[test]
    fn carries_the_kind() {
        let length = Length::new(60).unwrap();
        let major: Vec<u16> = TickMark::along(length)
            .filter(|mark| mark.kind() == TickKind::Major)
            .map(TickMark::offset)
            .collect();
        assert_eq!(major, [50]);
    }
}
