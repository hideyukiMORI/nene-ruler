//! 枠の窓 1 つ分の状態と、窓の手続きから呼ばれる処理（ADR 0021）。状態の正本は application（ARC-004）。

use nene_ruler_application::ruler_controller::RulerController;
use nene_ruler_application::ruler_intent::RulerIntent;
use nene_ruler_core::frame_layout::FrameLayout;
use nene_ruler_core::hit_region;
use nene_ruler_core::hit_target::HitTarget;
use nene_ruler_core::nudge::Nudge;
use nene_ruler_core::nudge_direction::NudgeDirection;
use nene_ruler_core::pixel_point::PixelPoint;
use windows::Win32::Foundation::{COLORREF, HWND, POINT, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{AC_SRC_ALPHA, AC_SRC_OVER, BLENDFUNCTION};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos, ULW_ALPHA,
    UpdateLayeredWindow,
};

use crate::frame_painter::FramePainter;
use crate::non_client_hit::NonClientHit;
use crate::win32_failure::Win32Failure;

/// 窓と、状態の持ち主と、描き手。窓の手続きはこれを通してだけ状態に触れる。
#[derive(Debug)]
pub struct FrameSession<'c, 'p> {
    window: HWND,
    controller: &'c mut RulerController<'p>,
    painter: FramePainter,
}

impl<'c, 'p> FrameSession<'c, 'p> {
    /// 状態の持ち主と描き手から作る。窓はまだ無い（`attach` で結ぶ）。
    #[must_use]
    pub const fn new(controller: &'c mut RulerController<'p>, painter: FramePainter) -> Self {
        Self {
            window: HWND(core::ptr::null_mut()),
            controller,
            painter,
        }
    }

    /// 作った窓を結ぶ。
    pub const fn attach(&mut self, window: HWND) {
        self.window = window;
    }

    /// いまの状態を描いて窓へ出す。
    ///
    /// # Errors
    ///
    /// 描画か `UpdateLayeredWindow` が失敗した。
    #[expect(
        unsafe_code,
        reason = "RS-010: Win32 の UpdateLayeredWindow を呼ぶ（ADR 0021）"
    )]
    pub fn present(&self) -> Result<(), Win32Failure> {
        let bitmap = self.painter.paint(&self.controller.frame())?;
        let size = SIZE {
            cx: i32::from(bitmap.width()),
            cy: i32::from(bitmap.height()),
        };
        let origin = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            BlendOp: u8::try_from(AC_SRC_OVER)
                .map_err(|_error| Win32Failure::GeometryOutOfRange)?,
            BlendFlags: 0,
            SourceConstantAlpha: u8::MAX,
            AlphaFormat: u8::try_from(AC_SRC_ALPHA)
                .map_err(|_error| Win32Failure::GeometryOutOfRange)?,
        };
        // SAFETY: 窓は attach で結んだもので、bitmap の DC はこの関数の間生きている。位置は変えない（None）。
        unsafe {
            UpdateLayeredWindow(
                self.window,
                None,
                None,
                Some(&raw const size),
                Some(bitmap.dc()),
                Some(&raw const origin),
                COLORREF(0),
                Some(&raw const blend),
                ULW_ALPHA,
            )
        }?;
        Ok(())
    }

    /// 窓の中の点の当たり判定。
    #[must_use]
    pub fn hit(&self, client: (i32, i32)) -> NonClientHit {
        NonClientHit::of(self.target_at(client))
    }

    /// 枠が最小のときの窓の一辺（`WM_GETMINMAXINFO`・Issue #52）。
    #[must_use]
    pub fn smallest_window(&self) -> i32 {
        i32::from(FrameLayout::smallest_window(
            self.controller.frame().layout().scale(),
        ))
    }

    /// 窓の大きさが変わった（`WM_SIZE`）。枠を合わせて描き直す。
    ///
    /// # Errors
    ///
    /// 描き直しに失敗した。
    pub fn resized(&mut self, width: u16, height: u16) -> Result<(), Win32Failure> {
        self.controller.apply(RulerIntent::WindowResized {
            width_px: width,
            height_px: height,
        });
        self.present()
    }

    /// 左ボタンで押した（窓の中の点）。数字ならコピーする。ギアは設定モーダルを開く（次の Issue）。
    ///
    /// # Errors
    ///
    /// 描き直しに失敗した。
    pub fn pressed(&mut self, client: (i32, i32)) -> Result<(), Win32Failure> {
        let Some(HitTarget::Copy(label)) = self.target_at(client) else {
            return Ok(());
        };
        self.controller.apply(RulerIntent::CopyText(label));
        self.present()
    }

    /// ポインタが動いた・窓から出た。コピーの印があり、数字から離れたら消す（ADR 0007 の 5）。
    ///
    /// # Errors
    ///
    /// 描き直しに失敗した。
    pub fn pointer_moved(&mut self, client: Option<(i32, i32)>) -> Result<(), Win32Failure> {
        let frame = self.controller.frame();
        let Some(mark) = frame.copy_mark() else {
            return Ok(());
        };
        let hit = frame
            .layout()
            .label_hit(mark.label(), frame.shown(mark.label()).chars());
        let still_over = client
            .and_then(point_in_window)
            .is_some_and(|point| hit.contains(point));
        if still_over {
            return Ok(());
        }
        self.controller.apply(RulerIntent::ClearCopyMark);
        self.present()
    }

    /// 矢印で 1 px（Shift で 10 px）動かす（FR-015）。
    ///
    /// # Errors
    ///
    /// 窓の位置を読めない・変えられない。
    #[expect(
        unsafe_code,
        reason = "RS-010: Win32 の GetWindowRect / SetWindowPos を呼ぶ（ADR 0021）"
    )]
    pub fn nudge(&self, direction: NudgeDirection, fast: bool) -> Result<(), Win32Failure> {
        let nudge = Nudge::new(direction, fast);
        let mut rect = RECT::default();
        // SAFETY: 窓は attach で結んだもの。rect はこの関数の局所。大きさと重なり順は変えない。
        unsafe {
            GetWindowRect(self.window, &raw mut rect)?;
            SetWindowPos(
                self.window,
                None,
                rect.left + nudge.dx(),
                rect.top + nudge.dy(),
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
        }
        Ok(())
    }

    /// 窓の中の点にある当たり判定の意味。
    fn target_at(&self, client: (i32, i32)) -> Option<HitTarget> {
        let point = point_in_window(client)?;
        hit_region::target_at(&self.controller.frame().hit_regions(), point)
    }
}

/// 窓の中の `i32` の点を、core の点に。窓の外（負）は `None`。
fn point_in_window((x, y): (i32, i32)) -> Option<PixelPoint> {
    Some(PixelPoint::new(
        u16::try_from(x).ok()?,
        u16::try_from(y).ok()?,
    ))
}
