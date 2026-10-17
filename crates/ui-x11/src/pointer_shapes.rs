//! 枠の窓のポインタの形。当たり判定の種類ごとに、X の標準カーソルフォント（`cursor`）の字形を使う（Issue #36）。
//!
//! カーソルのテーマ（Xcursor）は読まない。x11rb の `cursor` 機能が要り、施主の `WSLg` にはテーマが入っていない。

use nene_ruler_core::anchor::Anchor;
use nene_ruler_core::hit_target::HitTarget;
use x11rb::protocol::xproto::{Cursor, CursorWrapper, FontWrapper};
use x11rb::rust_connection::RustConnection;

use crate::display::Display;
use crate::ui_failure::UiFailure;

/// `X11/cursorfont.h` の字形。マスクは次の番号。
const XC_BOTTOM_RIGHT_CORNER: u16 = 14;
const XC_FLEUR: u16 = 52;
const XC_HAND2: u16 = 60;
const XC_TOP_LEFT_CORNER: u16 = 134;
/// 前景（黒）と背景（白）。16 ビットの成分。
const BLACK: u16 = 0;
const WHITE: u16 = u16::MAX;

/// 4 つの形を起動時に一度作って持つ。`Drop` で解放する。
#[derive(Debug)]
pub struct PointerShapes<'d> {
    moving: CursorWrapper<&'d RustConnection>,
    top_left: CursorWrapper<&'d RustConnection>,
    bottom_right: CursorWrapper<&'d RustConnection>,
    hand: CursorWrapper<&'d RustConnection>,
}

impl<'d> PointerShapes<'d> {
    /// カーソルフォントを開き、4 つの形を作る。
    ///
    /// # Errors
    ///
    /// X の要求が失敗した。
    pub fn load(display: &'d Display) -> Result<Self, UiFailure> {
        let connection = display.connection();
        let font = FontWrapper::open_font(connection, b"cursor")?;
        let glyph = |source: u16| {
            CursorWrapper::create_glyph_cursor(
                connection,
                font.font(),
                font.font(),
                source,
                source + 1,
                BLACK,
                BLACK,
                BLACK,
                WHITE,
                WHITE,
                WHITE,
            )
        };
        Ok(Self {
            moving: glyph(glyph_of(HitTarget::Move))?,
            top_left: glyph(glyph_of(HitTarget::Resize(Anchor::TopLeft)))?,
            bottom_right: glyph(glyph_of(HitTarget::Resize(Anchor::BottomRight)))?,
            hand: glyph(glyph_of(HitTarget::Gear))?,
        })
    }

    /// 当たり判定の種類に対応する形。どれにも当たらなければ親の窓の形を継ぐ（`NONE`）。
    #[must_use]
    pub fn cursor_for(&self, target: Option<HitTarget>) -> Cursor {
        match target {
            None => x11rb::NONE,
            Some(HitTarget::Move) => self.moving.cursor(),
            Some(HitTarget::Resize(Anchor::TopLeft)) => self.top_left.cursor(),
            Some(HitTarget::Resize(Anchor::BottomRight)) => self.bottom_right.cursor(),
            Some(HitTarget::Gear | HitTarget::Copy(_)) => self.hand.cursor(),
        }
    }
}

/// 当たり判定の種類 → カーソルフォントの字形。
const fn glyph_of(target: HitTarget) -> u16 {
    match target {
        HitTarget::Move => XC_FLEUR,
        HitTarget::Resize(Anchor::TopLeft) => XC_TOP_LEFT_CORNER,
        HitTarget::Resize(Anchor::BottomRight) => XC_BOTTOM_RIGHT_CORNER,
        HitTarget::Gear | HitTarget::Copy(_) => XC_HAND2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ruler_core::axis::Axis;

    #[test]
    fn each_target_has_its_glyph() {
        assert_eq!(glyph_of(HitTarget::Move), XC_FLEUR);
        assert_eq!(
            glyph_of(HitTarget::Resize(Anchor::TopLeft)),
            XC_TOP_LEFT_CORNER
        );
        assert_eq!(
            glyph_of(HitTarget::Resize(Anchor::BottomRight)),
            XC_BOTTOM_RIGHT_CORNER
        );
        assert_eq!(glyph_of(HitTarget::Gear), XC_HAND2);
        assert_eq!(glyph_of(HitTarget::Copy(Axis::Vertical)), XC_HAND2);
    }
}
