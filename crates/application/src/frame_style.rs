//! 枠の見た目のうち、幾何でも文字でもないもの。

use nene_ruler_core::rgb::Rgb;
use nene_ruler_core::window_layer::WindowLayer;

use crate::copy_mark::CopyMark;
use crate::frame_notices::FrameNotices;
use crate::transparency::Transparency;

/// インク色・重なり順・透明・コピーの印・通知。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameStyle {
    ink: Rgb,
    layer: WindowLayer,
    transparency: Transparency,
    copy_mark: Option<CopyMark>,
    notices: FrameNotices,
}

impl FrameStyle {
    /// 値を束ねる。
    #[must_use]
    pub const fn new(
        ink: Rgb,
        layer: WindowLayer,
        transparency: Transparency,
        copy_mark: Option<CopyMark>,
    ) -> Self {
        Self {
            ink,
            layer,
            transparency,
            copy_mark,
            notices: FrameNotices::NONE,
        }
    }

    /// 通知を付けた見た目。
    #[must_use]
    pub const fn with_notices(self, notices: FrameNotices) -> Self {
        Self { notices, ..self }
    }

    /// 通知。
    #[must_use]
    pub const fn notices(self) -> FrameNotices {
        self.notices
    }

    /// インク色。
    #[must_use]
    pub const fn ink(self) -> Rgb {
        self.ink
    }

    /// 重なり順。
    #[must_use]
    pub const fn layer(self) -> WindowLayer {
        self.layer
    }

    /// 透明に描けるか。
    #[must_use]
    pub const fn transparency(self) -> Transparency {
        self.transparency
    }

    /// コピーの印。
    #[must_use]
    pub const fn copy_mark(self) -> Option<CopyMark> {
        self.copy_mark
    }
}
