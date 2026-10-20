//! 枠とモーダルの 2 つの窓が共有する状態（ADR 0021）。状態の正本は application（ARC-004）。
//!
//! 窓の手続きは `RefCell` に入れたこれを借りて処理する。借りている最中に入れ子で届いたメッセージは
//! 既定の処理へ流れる（`frame_window.rs`）。窓を壊すのは `WM_CLOSE` を投げて後で行う。

use nene_ruler_application::ruler_controller::RulerController;
use nene_ruler_application::ruler_intent::RulerIntent;
use nene_ruler_core::frame_layout::FrameLayout;
use nene_ruler_core::hit_region;
use nene_ruler_core::hit_target::HitTarget;
use nene_ruler_core::nudge::Nudge;
use nene_ruler_core::nudge_direction::NudgeDirection;
use nene_ruler_core::pixel_point::PixelPoint;
use nene_ruler_core::window_layer::WindowLayer;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{AC_SRC_ALPHA, AC_SRC_OVER, BLENDFUNCTION};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, HWND_NOTOPMOST, HWND_TOPMOST, PostMessageW, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOZORDER, SetWindowPos, ULW_ALPHA, UpdateLayeredWindow, WM_CLOSE,
};

use crate::frame_painter::paint_frame;
use crate::layered_bitmap::LayeredBitmap;
use crate::modal_focus::ModalFocus;
use crate::modal_key::ModalKey;
use crate::modal_reply::ModalReply;
use crate::modal_window::ModalWindow;
use crate::non_client_hit::NonClientHit;
use crate::pen::Pen;
use crate::settings_painter::paint_settings;
use crate::win32_failure::Win32Failure;

/// 状態の持ち主・描く道具・枠の窓・開いていればモーダル。
#[derive(Debug)]
pub struct WindowsApp<'c, 'p> {
    controller: &'c mut RulerController<'p>,
    pen: Pen,
    frame: HWND,
    modal: Option<ModalWindow>,
}

impl<'c, 'p> WindowsApp<'c, 'p> {
    /// 状態の持ち主と描く道具から。窓はまだ無い（`attach_frame` で結ぶ）。
    #[must_use]
    pub const fn new(controller: &'c mut RulerController<'p>, pen: Pen) -> Self {
        Self {
            controller,
            pen,
            frame: HWND(core::ptr::null_mut()),
            modal: None,
        }
    }

    /// 作った枠の窓を結ぶ。
    pub const fn attach_frame(&mut self, window: HWND) {
        self.frame = window;
    }

    /// 枠の窓。
    #[must_use]
    pub const fn frame(&self) -> HWND {
        self.frame
    }

    /// 開いていればモーダルの窓。
    #[must_use]
    pub fn modal(&self) -> Option<HWND> {
        self.modal.as_ref().map(ModalWindow::window)
    }

    /// 状態の持ち主（窓を作るときに置き場を読むため）。
    #[must_use]
    pub const fn controller(&self) -> &RulerController<'p> {
        self.controller
    }

    // ── 枠の窓 ─────────────────────────────────────────────────────────────

    /// 枠を描いて窓へ出す。
    ///
    /// # Errors
    ///
    /// 描画か `UpdateLayeredWindow` が失敗した。
    pub fn present_frame(&self) -> Result<(), Win32Failure> {
        show(
            self.frame,
            &paint_frame(&self.pen, &self.controller.frame())?,
        )
    }

    /// 枠の窓の中の点の当たり判定。
    #[must_use]
    pub fn frame_hit(&self, client: (i32, i32)) -> NonClientHit {
        NonClientHit::of(self.target_at(client))
    }

    /// 枠が最小のときの窓の一辺（`WM_GETMINMAXINFO`・Issue #52）。
    #[must_use]
    pub fn smallest_window(&self) -> i32 {
        i32::from(FrameLayout::smallest_window(
            self.controller.frame().layout().scale(),
        ))
    }

    /// 定規の窓が乗っているモニタの dpi が分かった・変わった（`WM_DPICHANGED`・窓を作った直後）。
    /// 数字・字の大きさ・目盛と、開いていればモーダルの説明文を描き直す（ADR 0029）。
    ///
    /// # Errors
    ///
    /// 描き直しに失敗した。
    pub fn dpi_changed(&mut self, dpi: Option<u32>) -> Result<(), Win32Failure> {
        self.controller.apply(RulerIntent::DpiChanged(dpi));
        self.present_frame()?;
        if self.modal.is_some() {
            self.present_modal()?;
        }
        Ok(())
    }

    /// 枠の窓の大きさが変わった（`WM_SIZE`）。
    ///
    /// # Errors
    ///
    /// 描き直しに失敗した。
    pub fn resized(&mut self, width: u16, height: u16) -> Result<(), Win32Failure> {
        self.controller.apply(RulerIntent::WindowResized {
            width_px: width,
            height_px: height,
        });
        self.present_frame()
    }

    /// 枠の窓を左ボタンで押した。数字ならコピー、ギアなら設定モーダルを開いてよいかを返す。
    ///
    /// # Errors
    ///
    /// 描き直しに失敗した。
    pub fn frame_pressed(&mut self, client: (i32, i32)) -> Result<bool, Win32Failure> {
        match self.target_at(client) {
            Some(HitTarget::Copy(label)) => {
                self.controller.apply(RulerIntent::CopyText(label));
                self.present_frame()?;
                Ok(false)
            }
            Some(HitTarget::Gear) => Ok(self.modal.is_none()),
            Some(HitTarget::Move | HitTarget::Resize(_)) | None => Ok(false),
        }
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
        if client
            .and_then(point_in_window)
            .is_some_and(|point| hit.contains(point))
        {
            return Ok(());
        }
        self.controller.apply(RulerIntent::ClearCopyMark);
        self.present_frame()
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
        // SAFETY: 窓は attach_frame で結んだもの。rect はこの関数の局所。大きさと重なり順は変えない。
        unsafe {
            GetWindowRect(self.frame, &raw mut rect)?;
            SetWindowPos(
                self.frame,
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

    /// 閉じる（Esc）。`WM_CLOSE` を投げ、借りていない時に窓を壊す。
    pub fn close_frame(&self) {
        post_close(self.frame);
    }

    // ── 設定モーダル ───────────────────────────────────────────────────────

    /// 設定モーダルを開いた状態にする（`RulerIntent::OpenSettings`）。窓は呼び出し側が作る。
    pub fn open_settings(&mut self) {
        self.controller.apply(RulerIntent::OpenSettings);
    }

    /// モーダルを開いた（窓は作ってある）。状態を開き、描いて出す。
    ///
    /// # Errors
    ///
    /// 描画に失敗した。
    pub fn modal_opened(&mut self, window: HWND) -> Result<(), Win32Failure> {
        self.modal = Some(ModalWindow::new(window));
        self.present_modal()
    }

    /// モーダルの窓が壊れた。状態を閉じる。
    pub fn modal_destroyed(&mut self) {
        self.modal = None;
        self.controller.apply(RulerIntent::CloseSettings);
    }

    /// モーダルを描いて窓へ出す。開いていなければ何もしない。
    ///
    /// # Errors
    ///
    /// 描画か `UpdateLayeredWindow` が失敗した。
    pub fn present_modal(&self) -> Result<(), Win32Failure> {
        let (Some(modal), Some(frame)) = (&self.modal, self.controller.settings_frame()) else {
            return Ok(());
        };
        show(
            modal.window(),
            &paint_settings(&self.pen, &frame, modal.focus())?,
        )
    }

    /// モーダルの窓の中の点の当たり判定。
    #[must_use]
    pub fn modal_hit(&self, client: (i32, i32)) -> NonClientHit {
        ModalWindow::hit(self.controller, client)
    }

    /// モーダルを左ボタンで押した。
    ///
    /// # Errors
    ///
    /// 描き直しか重なり順の変更に失敗した。
    pub fn modal_pressed(&mut self, client: (i32, i32)) -> Result<ModalReply, Win32Failure> {
        let reply = match &mut self.modal {
            Some(modal) => modal.pressed(self.controller, client),
            None => ModalReply::Unchanged,
        };
        self.follow(reply)
    }

    /// モーダルの上でポインタが動いた。
    ///
    /// # Errors
    ///
    /// 描き直しに失敗した。
    pub fn modal_dragged(&mut self, client: (i32, i32)) -> Result<ModalReply, Win32Failure> {
        let reply = match &self.modal {
            Some(modal) => modal.dragged(self.controller, client),
            None => ModalReply::Unchanged,
        };
        self.follow(reply)
    }

    /// モーダルで左ボタンを離した。引いていたなら真。
    pub fn modal_released(&mut self) -> bool {
        self.modal.as_mut().is_some_and(ModalWindow::released)
    }

    /// モーダルで字を打った。
    ///
    /// # Errors
    ///
    /// 描き直しに失敗した。
    pub fn modal_typed(&mut self, character: char) -> Result<ModalReply, Win32Failure> {
        let reply = ModalWindow::typed(self.controller, character);
        self.follow(reply)
    }

    /// モーダルで鍵を押した。
    ///
    /// # Errors
    ///
    /// 描き直しに失敗した。
    pub fn modal_key(&mut self, key: ModalKey) -> Result<ModalReply, Win32Failure> {
        let reply = ModalWindow::key(self.controller, key);
        self.follow(reply)
    }

    /// モーダルの焦点が出入りした。
    ///
    /// # Errors
    ///
    /// 描き直しに失敗した。
    pub fn modal_refocused(&mut self, focus: ModalFocus) -> Result<(), Win32Failure> {
        if let Some(modal) = &mut self.modal {
            modal.refocused(focus);
        }
        self.present_modal()
    }

    /// 処理の後に窓の側ですること。
    fn follow(&self, reply: ModalReply) -> Result<ModalReply, Win32Failure> {
        match reply {
            ModalReply::Unchanged => {}
            ModalReply::Changed => {
                self.present_frame()?;
                self.present_modal()?;
            }
            ModalReply::LayerChanged => {
                self.set_layer(self.controller.frame().layer())?;
                self.present_frame()?;
                self.present_modal()?;
            }
            ModalReply::Close => {
                if let Some(window) = self.modal() {
                    post_close(window);
                }
            }
            ModalReply::Quit => post_close(self.frame),
        }
        Ok(reply)
    }

    /// 枠の窓の重なり順を設定に合わせる（FR-013）。
    #[expect(
        unsafe_code,
        reason = "RS-010: Win32 の SetWindowPos を呼ぶ（ADR 0021）"
    )]
    fn set_layer(&self, layer: WindowLayer) -> Result<(), Win32Failure> {
        let after = match layer {
            WindowLayer::AlwaysOnTop => HWND_TOPMOST,
            WindowLayer::Normal => HWND_NOTOPMOST,
        };
        // SAFETY: 窓は attach_frame で結んだもの。位置と大きさは変えない。
        unsafe {
            SetWindowPos(
                self.frame,
                Some(after),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )?;
        }
        Ok(())
    }

    /// 枠の窓の中の点にある当たり判定の意味。
    fn target_at(&self, client: (i32, i32)) -> Option<HitTarget> {
        let point = point_in_window(client)?;
        hit_region::target_at(&self.controller.frame().hit_regions(), point)
    }
}

/// 画素を透明な窓へ出す。位置は変えない。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の UpdateLayeredWindow を呼ぶ（ADR 0021）"
)]
fn show(window: HWND, bitmap: &LayeredBitmap) -> Result<(), Win32Failure> {
    let size = SIZE {
        cx: i32::from(bitmap.width()),
        cy: i32::from(bitmap.height()),
    };
    let origin = POINT { x: 0, y: 0 };
    let blend = BLENDFUNCTION {
        BlendOp: u8::try_from(AC_SRC_OVER).map_err(|_error| Win32Failure::GeometryOutOfRange)?,
        BlendFlags: 0,
        SourceConstantAlpha: u8::MAX,
        AlphaFormat: u8::try_from(AC_SRC_ALPHA)
            .map_err(|_error| Win32Failure::GeometryOutOfRange)?,
    };
    // SAFETY: 窓はこのアプリが作ったもの。bitmap の DC はこの関数の間生きている。
    unsafe {
        UpdateLayeredWindow(
            window,
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

/// 窓に `WM_CLOSE` を投げる。処理は借りていない時に既定の手続きが行う。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の PostMessageW を呼ぶ（ADR 0021）"
)]
fn post_close(window: HWND) {
    // SAFETY: 自分の窓へ値だけのメッセージを積む。
    let _posted: Result<(), _> =
        unsafe { PostMessageW(Some(window), WM_CLOSE, WPARAM(0), LPARAM(0)) };
}

/// 窓の中の `i32` の点を、core の点に。窓の外（負）は `None`。
fn point_in_window((x, y): (i32, i32)) -> Option<PixelPoint> {
    Some(PixelPoint::new(
        u16::try_from(x).ok()?,
        u16::try_from(y).ok()?,
    ))
}
