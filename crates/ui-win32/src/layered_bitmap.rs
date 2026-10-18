//! 透明な窓へ出す 1 フレーム分の画素。メモリ DC に選んだ 32 ビットの DIB（premultiplied の BGRA）。

use core::ptr;

use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, HBITMAP, HDC, HGDIOBJ, SelectObject,
};

use crate::win32_failure::Win32Failure;

/// `UpdateLayeredWindow` に渡すメモリ DC と、その DC に選んだ DIB。`Drop` で手放す。
#[derive(Debug)]
pub struct LayeredBitmap {
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    width: u16,
    height: u16,
}

impl LayeredBitmap {
    /// 幅と高さの、上から下へ並ぶ 32 ビットの DIB を作り、新しいメモリ DC に選ぶ。
    ///
    /// # Errors
    ///
    /// DC か DIB を作れなかった。
    #[expect(
        unsafe_code,
        reason = "RS-010: Win32 の CreateCompatibleDC / CreateDIBSection / SelectObject を呼ぶ（ADR 0021）"
    )]
    pub fn new(width: u16, height: u16) -> Result<Self, Win32Failure> {
        let header = BITMAPINFOHEADER {
            biSize: u32::try_from(size_of::<BITMAPINFOHEADER>())
                .map_err(|_error| Win32Failure::GeometryOutOfRange)?,
            biWidth: i32::from(width),
            // 負の高さは上から下へ並ぶ DIB（Direct2D と同じ向き）。
            biHeight: -i32::from(height),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };
        let info = BITMAPINFO {
            bmiHeader: header,
            bmiColors: [windows::Win32::Graphics::Gdi::RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }],
        };
        // SAFETY: 画面と互換のメモリ DC を作るだけ。失敗は null で返り、下で確かめる。
        let dc = unsafe { CreateCompatibleDC(None) };
        if dc.is_invalid() {
            return Err(Win32Failure::Api(windows::core::Error::from_thread()));
        }
        let mut bits = ptr::null_mut();
        // SAFETY: info はこの関数の局所で、bits は DIB の画素の先頭を受け取るだけ（使わない）。
        let created = unsafe {
            CreateDIBSection(
                Some(dc),
                &raw const info,
                DIB_RGB_COLORS,
                &raw mut bits,
                None,
                0,
            )
        };
        let bitmap = match created {
            Ok(bitmap) => bitmap,
            Err(error) => {
                // SAFETY: dc はこの関数で作り、まだ何も選んでいない。
                let _deleted: bool = unsafe { DeleteDC(dc) }.as_bool();
                return Err(Win32Failure::Api(error));
            }
        };
        // SAFETY: dc と bitmap はこの関数で作った。前に選ばれていたものは Drop で戻す。
        let previous = unsafe { SelectObject(dc, bitmap.into()) };
        Ok(Self {
            dc,
            bitmap,
            previous,
            width,
            height,
        })
    }

    /// メモリ DC。Direct2D の DC レンダーターゲットを結び、`UpdateLayeredWindow` の源にする。
    #[must_use]
    pub const fn dc(&self) -> HDC {
        self.dc
    }

    /// 幅。
    #[must_use]
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// 高さ。
    #[must_use]
    pub const fn height(&self) -> u16 {
        self.height
    }
}

impl Drop for LayeredBitmap {
    #[expect(
        unsafe_code,
        reason = "RS-010: Win32 の SelectObject / DeleteObject / DeleteDC を呼ぶ（ADR 0021）"
    )]
    fn drop(&mut self) {
        // SAFETY: new で選んだ前のものを戻してから、作った DIB と DC を手放す。どれもこの値だけが持つ。
        unsafe {
            SelectObject(self.dc, self.previous);
            let _bitmap_deleted: bool = DeleteObject(self.bitmap.into()).as_bool();
            let _dc_deleted: bool = DeleteDC(self.dc).as_bool();
        }
    }
}
