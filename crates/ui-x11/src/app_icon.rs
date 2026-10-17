//! アプリのアイコン（案 A・ADR 0020）。画像ファイルを同梱せず、要る大きさをその場で描く。
//!
//! 暗い角丸の地に、左上の生成りの鉤括弧と、右下の赤い鉤括弧。右下は掴んで伸ばす方で、
//! `NeNe Clock` の「動く方の点だけ色を変える」と同じ考え方で 2 つの要素の色を分ける。
//! 座標は 128 の格子の中の値で、16 px まで縮めても腕（太さ 14）が 1.75 px 残る。

use core::f64::consts::{FRAC_PI_2, PI};

use cairo::{Context, Format};
use nene_ruler_core::pixel_rect::PixelRect;
use nene_ruler_core::rgb::Rgb;
use x11rb::protocol::xproto::{AtomEnum, PropMode, Window};
use x11rb::wrapper::ConnectionExt as _;

use crate::canvas::{Canvas, set_ink};
use crate::display::Display;
use crate::ui_failure::UiFailure;

/// 設計した格子の一辺。
const GRID: f64 = 128.0;
/// 地の角の半径。
const CORNER: f64 = 26.0;
/// 地（NeNe Clock と同じ）。
const GROUND: Rgb = Rgb::new(0x1A, 0x19, 0x17);
/// 左上の鉤括弧（NeNe Clock の上の点と同じ生成り）。
const STILL: Rgb = Rgb::new(0xF5, 0xF2, 0xEB);
/// 右下の鉤括弧。既定のインク色に固定する（利用者のインク色には追従しない）。
const MOVING: Rgb = Rgb::INITIAL_INK;
/// 左上の鉤括弧の輪郭（格子の座標）。腕の長さ 44・太さ 14。
const TOP_LEFT: [(f64, f64); 6] = [
    (28.0, 28.0),
    (72.0, 28.0),
    (72.0, 42.0),
    (42.0, 42.0),
    (42.0, 72.0),
    (28.0, 72.0),
];
/// 右下の鉤括弧の輪郭。左上と点対称。
const BOTTOM_RIGHT: [(f64, f64); 6] = [
    (100.0, 100.0),
    (56.0, 100.0),
    (56.0, 86.0),
    (86.0, 86.0),
    (86.0, 56.0),
    (100.0, 56.0),
];
/// 窓マネージャへ渡す大きさ。小さいほうから。256 は `_NET_WM_ICON` が X の 1 要求の上限に近づくので入れない。
pub const SIZES: [u16; 6] = [16, 24, 32, 48, 64, 128];

/// 一辺 `size` で描く。
///
/// # Errors
///
/// cairo が失敗した。
pub fn render(size: u16) -> Result<Canvas, UiFailure> {
    Canvas::paint(
        PixelRect::new(0, 0, size, size),
        Format::ARgb32,
        |context| {
            let scale = f64::from(size) / GRID;
            context.scale(scale, scale);
            rounded_square(context);
            set_ink(context, GROUND);
            context.fill()?;
            for (outline, ink) in [(TOP_LEFT, STILL), (BOTTOM_RIGHT, MOVING)] {
                polygon(context, &outline);
                set_ink(context, ink);
                context.fill()?;
            }
            Ok(())
        },
    )
}

/// `_NET_WM_ICON` の値。大きさごとに幅・高さ・画素（行ごと、非 premultiplied の ARGB）を並べる。
///
/// # Errors
///
/// cairo が失敗した。
pub fn net_wm_icon() -> Result<Vec<u32>, UiFailure> {
    let mut words = Vec::new();
    for size in SIZES {
        let mut canvas = render(size)?;
        words.push(u32::from(size));
        words.push(u32::from(size));
        for y in 0..size {
            for x in 0..size {
                let [blue, green, red, alpha] = canvas.pixel(x, y)?.unwrap_or([0, 0, 0, 0]);
                words.push(argb(alpha, [red, green, blue]));
            }
        }
    }
    Ok(words)
}

/// 窓に `_NET_WM_ICON` を付ける。
///
/// # Errors
///
/// cairo か X の要求が失敗した。
pub(crate) fn declare(display: &Display, window: Window) -> Result<(), UiFailure> {
    display.connection().change_property32(
        PropMode::REPLACE,
        window,
        display.atoms().net_wm_icon,
        AtomEnum::CARDINAL,
        &net_wm_icon()?,
    )?;
    Ok(())
}

/// premultiplied の成分を戻して `0xAARRGGBB` にする。
fn argb(alpha: u8, premultiplied: [u8; 3]) -> u32 {
    let straight = premultiplied.map(|channel| {
        if alpha == 0 {
            0_u32
        } else {
            (u32::from(channel) * 255 + u32::from(alpha) / 2) / u32::from(alpha)
        }
    });
    let [red, green, blue] = straight.map(|channel| channel.min(255));
    (u32::from(alpha) << 24) | (red << 16) | (green << 8) | blue
}

/// 格子いっぱいの角丸の正方形の輪郭。
fn rounded_square(context: &Context) {
    let far = GRID - CORNER;
    context.new_path();
    context.arc(far, CORNER, CORNER, -FRAC_PI_2, 0.0_f64);
    context.arc(far, far, CORNER, 0.0_f64, FRAC_PI_2);
    context.arc(CORNER, far, CORNER, FRAC_PI_2, PI);
    context.arc(CORNER, CORNER, CORNER, PI, PI + FRAC_PI_2);
    context.close_path();
}

/// 点の列を閉じた多角形にする。
fn polygon(context: &Context, outline: &[(f64, f64)]) {
    context.new_path();
    for &(x, y) in outline {
        context.line_to(x, y);
    }
    context.close_path();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_the_ground_and_both_brackets() {
        let mut canvas = render(128).unwrap();
        // 角は透明、中央は地、左上の腕は生成り、右下の腕は赤（並びは BGRA）。
        assert_eq!(canvas.pixel(0, 0).unwrap(), Some([0, 0, 0, 0]));
        assert_eq!(
            canvas.pixel(64, 64).unwrap(),
            Some([0x17, 0x19, 0x1A, 0xFF])
        );
        assert_eq!(
            canvas.pixel(50, 34).unwrap(),
            Some([0xEB, 0xF2, 0xF5, 0xFF])
        );
        assert_eq!(
            canvas.pixel(78, 93).unwrap(),
            Some([0x35, 0x39, 0xE5, 0xFF])
        );
    }

    #[test]
    fn the_property_lists_every_size_with_straight_alpha() {
        let words = net_wm_icon().unwrap();
        let expected: usize = SIZES
            .iter()
            .map(|size| 2 + usize::from(*size) * usize::from(*size))
            .sum();
        assert_eq!(words.len(), expected);
        assert_eq!(words.first(), Some(&16));
        assert_eq!(words.get(1), Some(&16));
        // premultiplied の半透明の白（128, 128, 128, 128）は、白（255）の半透明に戻る。
        assert_eq!(argb(128, [128, 128, 128]), 0x80FF_FFFF);
        assert_eq!(argb(0, [0, 0, 0]), 0);
        assert_eq!(argb(255, [0xE5, 0x39, 0x35]), 0xFFE5_3935);
    }
}
