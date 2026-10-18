//! 当たり判定の意味から、Windows のヒットテストの答えとポインタの形を決める（ADR 0021）。
//!
//! Win32 を呼ばない純粋な対応なので、Linux でもテストする。

use nene_ruler_core::anchor::Anchor;
use nene_ruler_core::hit_target::HitTarget;

/// `WM_NCHITTEST` で返すもの。移動と伸縮は Windows に任せる（ADR 0005 の 4 を Windows で）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonClientHit {
    /// `HTCAPTION`: 掴むと窓が動く。ポインタは十字の矢印（`IDC_SIZEALL`）。
    Caption,
    /// `HTTOPLEFT`: 左上の鉤括弧。ポインタは斜めの両矢印（`IDC_SIZENWSE`）。
    TopLeft,
    /// `HTBOTTOMRIGHT`: 右下の鉤括弧。ポインタは同じ斜めの両矢印。
    BottomRight,
    /// `HTCLIENT`: ギアと数字。押すと窓の手続きに届く。ポインタは指（`IDC_HAND`）。
    Client,
    /// `HTTRANSPARENT`: どれにも当たらない。下の窓へ素通しする。
    Transparent,
}

impl NonClientHit {
    /// 当たり判定の意味から。
    #[must_use]
    pub const fn of(target: Option<HitTarget>) -> Self {
        match target {
            None => Self::Transparent,
            Some(HitTarget::Move) => Self::Caption,
            Some(HitTarget::Resize(Anchor::TopLeft)) => Self::TopLeft,
            Some(HitTarget::Resize(Anchor::BottomRight)) => Self::BottomRight,
            Some(HitTarget::Gear | HitTarget::Copy(_)) => Self::Client,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ruler_core::length_label::LengthLabel;

    #[test]
    fn each_target_maps_to_a_hit_code() {
        assert_eq!(NonClientHit::of(None), NonClientHit::Transparent);
        assert_eq!(
            NonClientHit::of(Some(HitTarget::Move)),
            NonClientHit::Caption
        );
        assert_eq!(
            NonClientHit::of(Some(HitTarget::Resize(Anchor::TopLeft))),
            NonClientHit::TopLeft
        );
        assert_eq!(
            NonClientHit::of(Some(HitTarget::Resize(Anchor::BottomRight))),
            NonClientHit::BottomRight
        );
        assert_eq!(
            NonClientHit::of(Some(HitTarget::Gear)),
            NonClientHit::Client
        );
        assert_eq!(
            NonClientHit::of(Some(HitTarget::Copy(LengthLabel::Size))),
            NonClientHit::Client
        );
    }
}
