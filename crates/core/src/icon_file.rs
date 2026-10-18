//! Windows の `.ico` のバイト列（ADR 0023）。`build.rs` がこれを `OUT_DIR` に書き、exe の資源に埋める。
//!
//! 各大きさを 32 ビットの BMP（`BITMAPINFOHEADER`・下の行から・BGRA）と 1 ビットの AND マスクで入れる。
//! PNG で入れれば小さくなるが、圧縮の実装が要るので BMP にした。数値はすべて小さい側から（little-endian）。

use alloc::vec;
use alloc::vec::Vec;

use crate::app_icon::AppIcon;

/// `ICONDIR` の大きさ。
const DIRECTORY_BYTES: u32 = 6;
/// `ICONDIRENTRY` の大きさ。
const ENTRY_BYTES: u32 = 16;
/// `BITMAPINFOHEADER` の大きさ。
const HEADER_BYTES: u32 = 40;

/// `icons` を並べた `.ico` のバイト列。大きさは 256 まで（256 は項目の幅・高さに 0 と書く）。
#[must_use]
pub fn icon_file(icons: &[AppIcon]) -> Vec<u8> {
    let images: Vec<Vec<u8>> = icons.iter().map(bitmap).collect();
    let count = u16::try_from(icons.len()).unwrap_or(u16::MAX);
    let mut bytes = Vec::new();
    for word in [0_u16, 1_u16, count] {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    let mut offset = DIRECTORY_BYTES + ENTRY_BYTES * u32::from(count);
    for (icon, image) in icons.iter().zip(&images) {
        let side = u8::try_from(icon.size()).unwrap_or(0);
        let length = u32::try_from(image.len()).unwrap_or(u32::MAX);
        bytes.extend_from_slice(&[side, side, 0, 0]);
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&32_u16.to_le_bytes());
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(&offset.to_le_bytes());
        offset = offset.saturating_add(length);
    }
    for image in images {
        bytes.extend_from_slice(&image);
    }
    bytes
}

/// 1 つの大きさの画像。頭・色（下の行から BGRA）・AND マスク（透明の画素に 1）。
fn bitmap(icon: &AppIcon) -> Vec<u8> {
    let side = usize::from(icon.size());
    let mask_row = side.div_ceil(32) * 4;
    let colour_bytes = u32::try_from(side * side * 4 + mask_row * side).unwrap_or(u32::MAX);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&HEADER_BYTES.to_le_bytes());
    bytes.extend_from_slice(&i32::from(icon.size()).to_le_bytes());
    // 高さは色と AND マスクの 2 枚分。
    bytes.extend_from_slice(&(2 * i32::from(icon.size())).to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&32_u16.to_le_bytes());
    for field in [0_u32, colour_bytes, 0, 0, 0, 0] {
        bytes.extend_from_slice(&field.to_le_bytes());
    }
    let rows: Vec<&[[u8; 4]]> = icon.pixels().chunks(side.max(1)).rev().collect();
    for row in &rows {
        for &[red, green, blue, alpha] in *row {
            bytes.extend_from_slice(&[blue, green, red, alpha]);
        }
    }
    for row in &rows {
        let mut mask = vec![0_u8; mask_row];
        for (column, byte) in mask.iter_mut().enumerate() {
            *byte = (0..8)
                .filter(|bit| {
                    row.get(column * 8 + bit)
                        .is_some_and(|&[_, _, _, alpha]| alpha == 0)
                })
                .fold(0_u8, |acc, bit| acc | (0x80 >> bit));
        }
        bytes.extend_from_slice(&mask);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(bytes: &[u8], at: usize) -> u16 {
        u16::from_le_bytes([bytes[at], bytes[at + 1]])
    }

    fn double(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
    }

    #[test]
    fn the_directory_lists_every_size() {
        let icons = [AppIcon::render(16), AppIcon::render(256)];
        let bytes = icon_file(&icons);
        assert_eq!(
            (word(&bytes, 0), word(&bytes, 2), word(&bytes, 4)),
            (0, 1, 2)
        );
        // 16 px の項目: 幅・高さ 16、32 ビット、画像は 6 + 16 × 2 = 38 バイト目から。
        assert_eq!((bytes[6], bytes[7]), (16, 16));
        assert_eq!(word(&bytes, 12), 32);
        let first = 40 + 16 * 16 * 4 + 4 * 16;
        assert_eq!(double(&bytes, 14), first);
        assert_eq!(double(&bytes, 18), 38);
        // 256 px の項目は幅・高さを 0 と書き、16 px の画像の後に続く。
        assert_eq!((bytes[22], bytes[23]), (0, 0));
        assert_eq!(double(&bytes, 34), 38 + first);
        let second = 40 + 256 * 256 * 4 + 32 * 256;
        assert_eq!(bytes.len(), 38 + usize::try_from(first + second).unwrap());
    }

    #[test]
    fn the_bitmap_is_bottom_up_bgra_with_a_mask() {
        let icon = AppIcon::render(16);
        let bytes = icon_file(core::slice::from_ref(&icon));
        let image = 22;
        assert_eq!(double(&bytes, image), 40);
        assert_eq!(double(&bytes, image + 4), 16);
        assert_eq!(double(&bytes, image + 8), 32);
        // 色の最初の行は絵の一番下の行。その左端は透明な角。
        let colour = image + 40;
        assert_eq!(&bytes[colour..colour + 4], &[0, 0, 0, 0]);
        // 下から 5 行目（絵の 11 行目）の 10 列目は右下の赤い腕（BGRA）。
        let red = colour + (4 * 16 + 10) * 4;
        assert_eq!(&bytes[red..red + 4], &[0x35, 0x39, 0xE5, 0xFF]);
        // AND マスクの最初の行（絵の一番下）は、透明な左端の画素のビットが立つ。
        let mask = colour + 16 * 16 * 4;
        assert_eq!(bytes[mask] & 0x80, 0x80);
        assert_eq!(bytes.len(), mask + 4 * 16);
    }
}
