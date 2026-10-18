//! Linux のメニュー用のアイコンの PNG のバイト列（ADR 0024）。`build.rs` がこれを `OUT_DIR` に書く。
//!
//! RGBA（色の型 6・8 ビット）を、行ごとに濾過なし（0）で並べ、zlib の無圧縮の区切り（stored）で包む。
//! 圧縮しないので大きくなるが、deflate の実装が要らない。CRC32 と Adler-32 はここで計算する（依存 0）。
//! 数値は大きい側から（big-endian）。

use alloc::vec;
use alloc::vec::Vec;

use crate::app_icon::AppIcon;

/// PNG の署名。
const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
/// 無圧縮の区切り 1 つに入る最大のバイト数。
const STORED_BLOCK: usize = 65_535;
/// CRC32 の多項式（反転した形）。
const CRC_POLYNOMIAL: u32 = 0xEDB8_8320;
/// Adler-32 の法。
const ADLER_MODULUS: u32 = 65_521;

/// `icon` の PNG のバイト列。
#[must_use]
pub fn png_file(icon: &AppIcon) -> Vec<u8> {
    let side = u32::from(icon.size());
    let mut header = Vec::new();
    header.extend_from_slice(&side.to_be_bytes());
    header.extend_from_slice(&side.to_be_bytes());
    // 8 ビット・RGBA・deflate・濾過の方式 0・飛び越しなし。
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    let mut bytes = SIGNATURE.to_vec();
    chunk(&mut bytes, *b"IHDR", &header);
    chunk(&mut bytes, *b"IDAT", &zlib_stored(&scanlines(icon)));
    chunk(&mut bytes, *b"IEND", &[]);
    bytes
}

/// 行ごとに濾過の種類（0）を先頭に付けた画素の列。
fn scanlines(icon: &AppIcon) -> Vec<u8> {
    let side = usize::from(icon.size()).max(1);
    let mut raw = Vec::new();
    for row in icon.pixels().chunks(side) {
        raw.push(0);
        for pixel in row {
            raw.extend_from_slice(pixel);
        }
    }
    raw
}

/// 長さ・種類・中身・CRC32（種類と中身）の 1 つの塊を足す。
fn chunk(bytes: &mut Vec<u8>, kind: [u8; 4], body: &[u8]) {
    let length = u32::try_from(body.len()).unwrap_or(u32::MAX);
    bytes.extend_from_slice(&length.to_be_bytes());
    let start = bytes.len();
    bytes.extend_from_slice(&kind);
    bytes.extend_from_slice(body);
    let crc = crc32(bytes.get(start..).unwrap_or_default());
    bytes.extend_from_slice(&crc.to_be_bytes());
}

/// zlib の頭・無圧縮の区切りの列・Adler-32。
fn zlib_stored(raw: &[u8]) -> Vec<u8> {
    // CMF 0x78（deflate・窓 32 KiB）と FLG 0x01（0x7801 が 31 で割り切れる）。
    let mut bytes = vec![0x78, 0x01];
    let mut blocks = raw.chunks(STORED_BLOCK).peekable();
    if blocks.peek().is_none() {
        bytes.extend_from_slice(&[1, 0, 0, 0xFF, 0xFF]);
    }
    while let Some(block) = blocks.next() {
        let last = u8::from(blocks.peek().is_none());
        let length = u16::try_from(block.len()).unwrap_or(u16::MAX);
        bytes.push(last);
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(&(!length).to_le_bytes());
        bytes.extend_from_slice(block);
    }
    bytes.extend_from_slice(&adler32(raw).to_be_bytes());
    bytes
}

/// CRC32（ISO-HDLC。PNG と zip と同じ）。
fn crc32(bytes: &[u8]) -> u32 {
    !bytes.iter().fold(u32::MAX, |crc, &byte| {
        (0_u8..8_u8).fold(crc ^ u32::from(byte), |value, _bit| {
            if value & 1 == 1 {
                (value >> 1_u32) ^ CRC_POLYNOMIAL
            } else {
                value >> 1_u32
            }
        })
    })
}

/// Adler-32。
fn adler32(bytes: &[u8]) -> u32 {
    let (low, high) = bytes.iter().fold((1_u32, 0_u32), |(low, high), &byte| {
        let next = (low + u32::from(byte)) % ADLER_MODULUS;
        (next, (high + next) % ADLER_MODULUS)
    });
    (high << 16) | low
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksums_match_the_known_values() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
        assert_eq!(adler32(b""), 1);
    }

    #[test]
    fn the_file_has_a_signature_header_and_end() {
        let bytes = png_file(&AppIcon::render(16));
        assert_eq!(&bytes[..8], &SIGNATURE);
        // IHDR: 長さ 13・幅 16・高さ 16・8 ビット・RGBA。
        assert_eq!(&bytes[8..16], &[0, 0, 0, 13, b'I', b'H', b'D', b'R']);
        assert_eq!(&bytes[16..24], &[0, 0, 0, 16, 0, 0, 0, 16]);
        assert_eq!(&bytes[24..26], &[8, 6]);
        // IEND は中身が空で、CRC は決まった値。
        let end = bytes.len() - 12;
        assert_eq!(
            &bytes[end..],
            &[0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xAE, 0x42, 0x60, 0x82]
        );
    }

    #[test]
    fn large_images_are_split_into_stored_blocks() {
        let raw = vec![7_u8; STORED_BLOCK + 10];
        let stream = zlib_stored(&raw);
        assert_eq!(&stream[..2], &[0x78, 0x01]);
        // 1 つ目は最後でない区切りで 65535 バイト、2 つ目が最後の区切りで 10 バイト。
        assert_eq!(&stream[2..7], &[0, 0xFF, 0xFF, 0, 0]);
        let second = 7 + STORED_BLOCK;
        assert_eq!(&stream[second..second + 5], &[1, 10, 0, 0xF5, 0xFF]);
        assert_eq!(stream.len(), 2 + 2 * 5 + raw.len() + 4);
        assert_eq!(
            zlib_stored(&[]),
            vec![0x78, 0x01, 1, 0, 0, 0xFF, 0xFF, 0, 0, 0, 1]
        );
    }

    #[test]
    fn scanlines_start_each_row_with_no_filter() {
        let icon = AppIcon::render(16);
        let raw = scanlines(&icon);
        assert_eq!(raw.len(), 16 * (1 + 16 * 4));
        assert!(raw.chunks(1 + 16 * 4).all(|row| row[0] == 0));
    }
}
