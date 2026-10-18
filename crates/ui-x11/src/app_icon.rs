//! アプリの印を窓に付ける。アイコン（`_NET_WM_ICON`・ADR 0020 / 0023）と、メニューの項目と窓を結ぶ `WM_CLASS`（ADR 0024）。
//!
//! 絵の正本は core の `AppIcon`。ここは画素を X の値の並びに写すだけ。

use nene_ruler_core::app_icon::AppIcon;
use x11rb::protocol::xproto::{AtomEnum, PropMode, Window};
use x11rb::wrapper::ConnectionExt as _;

use crate::display::Display;
use crate::ui_failure::UiFailure;

/// `_NET_WM_ICON` の値。大きさごとに幅・高さ・画素（行ごと、非 premultiplied の `0xAARRGGBB`）を並べる。
#[must_use]
pub fn net_wm_icon() -> Vec<u32> {
    let mut words = Vec::new();
    for size in AppIcon::X11_SIZES {
        let icon = AppIcon::render(size);
        words.push(u32::from(size));
        words.push(u32::from(size));
        words.extend(icon.pixels().iter().map(|&pixel| argb(pixel)));
    }
    words
}

/// `WM_CLASS` の値（インスタンス名と類名・NUL 区切り）。`packaging/linux/nene-ruler.desktop` の
/// `StartupWMClass` と同じ名前で、デスクトップがメニューの項目（とそのアイコン）を窓に結ぶ。
const WM_CLASS: &[u8] = b"nene-ruler\0nene-ruler\0";

/// 窓に `_NET_WM_ICON` と `WM_CLASS` を付ける。
///
/// # Errors
///
/// X の要求が失敗した。
pub(crate) fn declare(display: &Display, window: Window) -> Result<(), UiFailure> {
    display.connection().change_property32(
        PropMode::REPLACE,
        window,
        display.atoms().net_wm_icon,
        AtomEnum::CARDINAL,
        &net_wm_icon(),
    )?;
    display.connection().change_property8(
        PropMode::REPLACE,
        window,
        AtomEnum::WM_CLASS,
        AtomEnum::STRING,
        WM_CLASS,
    )?;
    Ok(())
}

/// RGBA を `0xAARRGGBB` にする。
fn argb([red, green, blue, alpha]: [u8; 4]) -> u32 {
    (u32::from(alpha) << 24) | (u32::from(red) << 16) | (u32::from(green) << 8) | u32::from(blue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_property_lists_every_size_with_straight_alpha() {
        let words = net_wm_icon();
        let expected: usize = AppIcon::X11_SIZES
            .iter()
            .map(|size| 2 + usize::from(*size) * usize::from(*size))
            .sum();
        assert_eq!(words.len(), expected);
        assert_eq!(words.first(), Some(&16));
        assert_eq!(words.get(1), Some(&16));
        // 16 px の左上の角は透明。
        assert_eq!(words.get(2), Some(&0));
        assert_eq!(argb([0xE5, 0x39, 0x35, 0xFF]), 0xFFE5_3935);
        assert_eq!(argb([0xFF, 0xFF, 0xFF, 0x80]), 0x80FF_FFFF);
    }
}
