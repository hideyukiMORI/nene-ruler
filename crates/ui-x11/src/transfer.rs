//! `PutImage` の宛先。行ごとの転送で変わらないもの。

use x11rb::protocol::xproto::{ConnectionExt as _, Gcontext, ImageFormat, Window};
use x11rb::rust_connection::RustConnection;

use crate::ui_failure::UiFailure;

/// 転送先の窓と GC、1 行の幅、depth。
#[derive(Debug, Clone, Copy)]
pub(crate) struct Transfer {
    window: Window,
    gc: Gcontext,
    width: u16,
    depth: u8,
}

impl Transfer {
    /// 宛先を束ねる。
    pub(crate) const fn new(destination: (Window, Gcontext), width: u16, depth: u8) -> Self {
        Self {
            window: destination.0,
            gc: destination.1,
            width,
            depth,
        }
    }

    /// `rows.0` 行目から `rows.1` 行分の画素を転送する。
    pub(crate) fn put_rows(
        self,
        connection: &RustConnection,
        rows: (u16, u16),
        bytes: &[u8],
    ) -> Result<(), UiFailure> {
        let (first_row, count) = rows;
        if count == 0 {
            return Ok(());
        }
        let dst_y = i16::try_from(first_row).map_err(|_error| UiFailure::GeometryOutOfRange)?;
        connection.put_image(
            ImageFormat::Z_PIXMAP,
            self.window,
            self.gc,
            self.width,
            count,
            0,
            dst_y,
            0,
            self.depth,
            bytes,
        )?;
        Ok(())
    }
}
