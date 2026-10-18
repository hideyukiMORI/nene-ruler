//! アプリのアイコン（案 A・ADR 0020 / 0023）。絵の正本で、X11 の `_NET_WM_ICON` も Windows の `.ico` もこの画素を写す。
//!
//! 暗い角丸の地に、左上の生成りの鉤括弧と、右下の赤い鉤括弧。右下は掴んで伸ばす方で、
//! `NeNe Clock` の「動く方の点だけ色を変える」と同じ考え方で 2 つの要素の色を分ける。
//! 座標は 128 の格子の中の整数で、1 画素を 8 × 8 の標本で塗る。浮動小数を使わないので、
//! ビルドの機械（`build.rs`）でも実行の機械でも同じ画素になる。16 px まで縮めても腕（太さ 14）が 1.75 px 残る。

use alloc::vec::Vec;

use crate::rgb::Rgb;

/// 設計した格子の一辺。
const GRID: u64 = 128;
/// 地の角の半径。
const CORNER: u64 = 26;
/// 1 画素の一辺あたりの標本の数。
const SAMPLES: u16 = 8;
/// 地（NeNe Clock と同じ）。
const GROUND: Rgb = Rgb::new(0x1A, 0x19, 0x17);
/// 左上の鉤括弧（NeNe Clock の上の点と同じ生成り）。
const STILL: Rgb = Rgb::new(0xF5, 0xF2, 0xEB);
/// 右下の鉤括弧。既定のインク色に固定する（利用者のインク色には追従しない）。
const MOVING: Rgb = Rgb::INITIAL_INK;
/// 左上の鉤括弧。腕の長さ 44・太さ 14 の 2 本の矩形（格子の座標で左・上・右・下。右と下は含まない）。
const TOP_LEFT: [[u64; 4]; 2] = [[28, 28, 72, 42], [28, 28, 42, 72]];
/// 右下の鉤括弧。左上と点対称。
const BOTTOM_RIGHT: [[u64; 4]; 2] = [[56, 86, 100, 100], [86, 56, 100, 100]];

/// 一辺 `size` で塗ったアイコン。画素は上の行から、非 premultiplied の RGBA。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppIcon {
    size: u16,
    pixels: Vec<[u8; 4]>,
}

impl AppIcon {
    /// X の窓マネージャへ渡す大きさ。256 は `_NET_WM_ICON` が X の 1 要求の上限に近づくので入れない。
    pub const X11_SIZES: [u16; 6] = [16, 24, 32, 48, 64, 128];
    /// Linux のメニュー用の hicolor の PNG の大きさ（ADR 0024）。
    pub const LINUX_SIZES: [u16; 7] = [16, 24, 32, 48, 64, 128, 256];
    /// Windows の `.ico` に入れる大きさ。小さいアイコン・大きいアイコンの 100〜250% と、エクスプローラーの特大。
    pub const WINDOWS_SIZES: [u16; 8] = [16, 20, 24, 32, 40, 48, 64, 256];

    /// 一辺 `size` で塗る。
    #[must_use]
    pub fn render(size: u16) -> Self {
        let scale = 2 * u64::from(size) * u64::from(SAMPLES);
        let pixels = (0..size)
            .flat_map(|y| (0..size).map(move |x| pixel(x, y, scale)))
            .collect();
        Self { size, pixels }
    }

    /// 一辺の画素数。
    #[must_use]
    pub const fn size(&self) -> u16 {
        self.size
    }

    /// 全画素。上の行から、行の中は左から。
    #[must_use]
    pub fn pixels(&self) -> &[[u8; 4]] {
        &self.pixels
    }

    /// 1 画素（RGBA）。範囲の外は `None`。
    #[must_use]
    pub fn pixel(&self, x: u16, y: u16) -> Option<[u8; 4]> {
        if x >= self.size {
            return None;
        }
        let index = usize::from(y) * usize::from(self.size) + usize::from(x);
        self.pixels.get(index).copied()
    }
}

/// 画素 (x, y) を 8 × 8 の標本で塗る。色は覆った標本の平均、アルファは覆った割合。
fn pixel(x: u16, y: u16, scale: u64) -> [u8; 4] {
    let fine = |at: u16, sample: u16| {
        (2 * (u64::from(at) * u64::from(SAMPLES) + u64::from(sample)) + 1) * GRID
    };
    let inks: Vec<Rgb> = (0..SAMPLES)
        .flat_map(|row| (0..SAMPLES).map(move |column| (column, row)))
        .filter_map(|(column, row)| ink_at(fine(x, column), fine(y, row), scale))
        .collect();
    let covered = u64::try_from(inks.len()).unwrap_or(u64::MAX);
    if covered == 0 {
        return [0, 0, 0, 0];
    }
    let mean = |channel: fn(Rgb) -> u8| {
        let sum: u64 = inks.iter().map(|ink| u64::from(channel(*ink))).sum();
        u8::try_from((sum + covered / 2) / covered).unwrap_or(u8::MAX)
    };
    let total = u64::from(SAMPLES) * u64::from(SAMPLES);
    let alpha = u8::try_from((covered * 255 + total / 2) / total).unwrap_or(u8::MAX);
    [mean(Rgb::red), mean(Rgb::green), mean(Rgb::blue), alpha]
}

/// 標本の色。座標は格子の値に `scale` を掛けたもの。鉤括弧 → 地 → 無し の順に見る。
fn ink_at(x: u64, y: u64, scale: u64) -> Option<Rgb> {
    let inside = |rects: &[[u64; 4]]| {
        rects.iter().any(|&[left, top, right, bottom]| {
            left * scale <= x && x < right * scale && top * scale <= y && y < bottom * scale
        })
    };
    if inside(&TOP_LEFT) {
        Some(STILL)
    } else if inside(&BOTTOM_RIGHT) {
        Some(MOVING)
    } else if in_rounded_square(x, y, scale) {
        Some(GROUND)
    } else {
        None
    }
}

/// 格子いっぱいの角丸の正方形の中か。角の円の中心へ寄せた点からの距離で測る。
fn in_rounded_square(x: u64, y: u64, scale: u64) -> bool {
    let (near, far, radius) = (CORNER * scale, (GRID - CORNER) * scale, CORNER * scale);
    if x >= GRID * scale || y >= GRID * scale {
        return false;
    }
    let dx = x.abs_diff(x.clamp(near, far));
    let dy = y.abs_diff(y.clamp(near, far));
    dx * dx + dy * dy <= radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_the_ground_and_both_brackets() {
        let icon = AppIcon::render(128);
        assert_eq!(icon.pixels().len(), 128 * 128);
        // 角は透明、中央は地、左上の腕は生成り、右下の腕は赤。
        assert_eq!(icon.pixel(0, 0), Some([0, 0, 0, 0]));
        assert_eq!(icon.pixel(64, 64), Some([0x1A, 0x19, 0x17, 0xFF]));
        assert_eq!(icon.pixel(50, 34), Some([0xF5, 0xF2, 0xEB, 0xFF]));
        assert_eq!(icon.pixel(78, 93), Some([0xE5, 0x39, 0x35, 0xFF]));
        assert_eq!(icon.pixel(128, 0), None);
        assert_eq!(icon.pixel(0, 128), None);
    }

    #[test]
    fn the_rounded_corner_is_partly_covered() {
        let icon = AppIcon::render(128);
        // 角の円の縁の画素は、地の色で半端なアルファになる。
        let edge = icon.pixel(5, 10).unwrap();
        assert_eq!(&edge[..3], &[0x1A, 0x19, 0x17]);
        assert!(edge[3] > 0 && edge[3] < 0xFF, "{edge:?}");
        assert_eq!(icon.pixel(0, 64), Some([0x1A, 0x19, 0x17, 0xFF]));
    }

    #[test]
    fn small_sizes_keep_both_brackets() {
        let icon = AppIcon::render(16);
        // 16 px でも、左上の腕と右下の腕の上に覆いきった画素が残る（太さ 1.75 px）。
        assert_eq!(icon.pixel(5, 4), Some([0xF5, 0xF2, 0xEB, 0xFF]));
        assert_eq!(icon.pixel(10, 11), Some([0xE5, 0x39, 0x35, 0xFF]));
    }

    #[test]
    fn rendering_is_deterministic() {
        assert_eq!(AppIcon::render(48), AppIcon::render(48));
    }
}
