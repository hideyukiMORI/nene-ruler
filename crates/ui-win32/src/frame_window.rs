//! 枠の窓（Windows）。窓を作り、メッセージを状態へ渡し、終わるまで回す（ADR 0021）。
//!
//! 移動と伸縮は Windows に任せる（`WM_NCHITTEST` で `HTCAPTION` / `HTTOPLEFT` / `HTBOTTOMRIGHT` を返す）。
//! 窓の手続きは、窓に結んだ `FrameSession` へのポインタ（`GWLP_USERDATA`）で状態に辿り着く。

use core::ffi::c_void;
use core::ptr;

use nene_ruler_application::ruler_controller::RulerController;
use nene_ruler_core::nudge_direction::NudgeDirection;
use nene_ruler_core::window_layer::WindowLayer;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent, VK_DOWN, VK_ESCAPE, VK_LEFT,
    VK_RIGHT, VK_SHIFT, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GWLP_USERDATA,
    GetMessageW, GetSystemMetrics, GetWindowLongPtrW, HTBOTTOMRIGHT, HTCAPTION, HTCLIENT,
    HTTOPLEFT, HTTRANSPARENT, IDC_HAND, IDC_SIZEALL, IDC_SIZENWSE, LoadCursorW, MINMAXINFO, MSG,
    PostQuitMessage, RegisterClassW, SM_CXSCREEN, SM_CYSCREEN, SW_SHOW, SetCursor,
    SetWindowLongPtrW, ShowWindow, TranslateMessage, WINDOW_EX_STYLE, WM_DESTROY, WM_GETMINMAXINFO,
    WM_KEYDOWN, WM_LBUTTONDOWN, WM_MOUSEMOVE, WM_NCCALCSIZE, WM_NCCREATE, WM_NCHITTEST,
    WM_SETCURSOR, WM_SIZE, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOPMOST, WS_MINIMIZEBOX, WS_POPUP,
    WS_SYSMENU, WS_THICKFRAME,
};
use windows::core::{PCWSTR, w};

use crate::frame_painter::FramePainter;
use crate::frame_session::FrameSession;
use crate::non_client_hit::NonClientHit;
use crate::win32_failure::Win32Failure;

/// 窓の類の名と、窓の題（タスクバーと Alt+Tab に出る）。
const CLASS: PCWSTR = w!("NeNeRulerFrame");
const TITLE: PCWSTR = w!("NeNe Ruler");
/// `WM_MOUSELEAVE`。windows クレートでは Controls の機能にあるので、値だけをここに置く（winuser.h の 0x02A3）。
const WM_MOUSELEAVE: u32 = 0x02A3;

/// 窓を作る前に、プロセスを per-monitor v2 の dpi 対応にする。Windows に拡大させず、物理画素で描く（FR-016）。
/// 画面の大きさと dpi を読む前に呼ぶ。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の SetProcessDpiAwarenessContext を呼ぶ（ADR 0021）"
)]
pub fn declare_dpi_awareness() {
    // SAFETY: 値だけを渡す。既に決まっていれば失敗するが、そのときは既定の対応のまま続ける。
    let _declared: Result<(), _> =
        unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
}

/// 枠の窓を作り、閉じられるまでメッセージを回す。
///
/// # Errors
///
/// 窓を作れない・描けない。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の窓の登録・作成とメッセージループ（ADR 0021）"
)]
pub fn run(controller: &mut RulerController<'_>) -> Result<(), Win32Failure> {
    let frame = controller.frame();
    let window = frame.layout().window();
    let topmost = frame.layer() == WindowLayer::AlwaysOnTop;
    let mut session = FrameSession::new(controller, FramePainter::new()?);
    // SAFETY: 自分のモジュールのハンドルを読むだけ。
    let instance = unsafe { GetModuleHandleW(None) }?;
    let class = WNDCLASSW {
        lpfnWndProc: Some(procedure),
        hInstance: instance.into(),
        lpszClassName: CLASS,
        ..WNDCLASSW::default()
    };
    // SAFETY: class はこの関数の局所で、登録の間だけ読まれる。2 度目の登録の失敗は無視する（同じ類）。
    let _registered: u16 = unsafe { RegisterClassW(&raw const class) };
    let (width, height) = (i32::from(window.width()), i32::from(window.height()));
    // SAFETY: 主モニタの大きさを読むだけ（dpi 対応の後なので物理画素）。
    let (screen_width, screen_height) =
        unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    let extended = if topmost {
        WS_EX_LAYERED | WS_EX_TOPMOST
    } else {
        WS_EX_LAYERED | WINDOW_EX_STYLE(0)
    };
    let session_pointer: *mut FrameSession<'_, '_> = &raw mut session;
    // SAFETY: 作成の引数に session のポインタを渡し、WM_NCCREATE で窓に結ぶ。session はこの関数の終わりまで生き、
    // 窓はメッセージループが終わる前（WM_DESTROY）に壊れるので、窓の手続きがポインタを辿るのは session の生存中だけ。
    let created = unsafe {
        CreateWindowExW(
            extended,
            CLASS,
            TITLE,
            WS_POPUP | WS_THICKFRAME | WS_SYSMENU | WS_MINIMIZEBOX,
            (screen_width - width) / 2,
            (screen_height - height) / 2,
            width,
            height,
            None,
            None,
            Some(instance.into()),
            Some(session_pointer.cast::<c_void>()),
        )
    }?;
    session.attach(created);
    session.present()?;
    // SAFETY: 作った窓を見せる。戻り値は前に見えていたかどうかで、失敗ではない。
    let _was_visible = unsafe { ShowWindow(created, SW_SHOW) };
    let mut message = MSG::default();
    // SAFETY: このスレッドのメッセージを、WM_QUIT（WM_DESTROY で積む）まで取り出して配る。
    while unsafe { GetMessageW(&raw mut message, None, 0, 0) }.as_bool() {
        // SAFETY: 取り出したばかりのメッセージを訳して配る。
        unsafe {
            let _translated = TranslateMessage(&raw const message);
            DispatchMessageW(&raw const message);
        }
    }
    Ok(())
}

/// 窓の手続き。状態に触れる処理は `FrameSession` に渡し、それ以外は Windows の既定に任せる。
#[expect(
    unsafe_code,
    reason = "RS-010: 窓の手続きは Win32 から呼ばれ、窓に結んだポインタを辿る（ADR 0021）"
)]
unsafe extern "system" fn procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        // SAFETY: WM_NCCREATE の lparam は CREATESTRUCTW を指し、lpCreateParams は run が渡した session。
        unsafe {
            let create = ptr::with_exposed_provenance::<CREATESTRUCTW>(lparam.0.cast_unsigned());
            if let Some(create) = create.as_ref() {
                let address = create.lpCreateParams.expose_provenance().cast_signed();
                let _previous = SetWindowLongPtrW(window, GWLP_USERDATA, address);
            }
        }
    }
    if message == WM_DESTROY {
        // SAFETY: このスレッドのループに WM_QUIT を積むだけ。
        unsafe {
            PostQuitMessage(0);
        }
        return LRESULT(0);
    }
    // SAFETY: GWLP_USERDATA は WM_NCCREATE で結んだ session か 0。session は run の終わりまで生きる。
    // 1 つのメッセージを処理する間だけ借りる。処理の中で同じ窓へ同期的に送られるメッセージ（SetWindowPos の
    // WM_MOVE など）は、ここで扱わない種類なので借りずに既定へ流れる。
    let session = unsafe {
        ptr::with_exposed_provenance_mut::<FrameSession<'static, 'static>>(
            GetWindowLongPtrW(window, GWLP_USERDATA).cast_unsigned(),
        )
        .as_mut()
    };
    let Some(session) = session else {
        // SAFETY: 既定の手続きにそのまま渡す。
        return unsafe { DefWindowProcW(window, message, wparam, lparam) };
    };
    match handled(session, window, (message, wparam, lparam)) {
        Some(result) => result,
        // SAFETY: 既定の手続きにそのまま渡す。
        None => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

/// 扱うメッセージだけを処理する。`None` は既定の手続きへ渡す印。
fn handled(
    session: &mut FrameSession<'_, '_>,
    window: HWND,
    (message, wparam, lparam): (u32, WPARAM, LPARAM),
) -> Option<LRESULT> {
    match message {
        // 窓全体を中身にする（枠の飾りの領域を持たない）。作成時の最初の 1 回（wparam = 0）も含めて答える。
        // 答えないと WS_THICKFRAME の太さが差し引かれ、枠が 16 px 小さく始まる（実測: 400 が 384）。
        WM_NCCALCSIZE => Some(LRESULT(0)),
        WM_NCHITTEST => Some(hit_code(session.hit(client_point(window, lparam)?))),
        WM_SETCURSOR => set_cursor(lparam),
        WM_GETMINMAXINFO => {
            limit_size(session, lparam);
            Some(LRESULT(0))
        }
        WM_SIZE => {
            let (width, height) = packed_words(lparam);
            report(session.resized(width, height));
            Some(LRESULT(0))
        }
        WM_LBUTTONDOWN => {
            report(session.pressed(signed_words(lparam)));
            Some(LRESULT(0))
        }
        WM_MOUSEMOVE => {
            track_leave(window);
            report(session.pointer_moved(Some(signed_words(lparam))));
            Some(LRESULT(0))
        }
        WM_MOUSELEAVE => {
            report(session.pointer_moved(None));
            Some(LRESULT(0))
        }
        WM_KEYDOWN => key_down(session, window, wparam),
        _other => None,
    }
}

/// 失敗を握る。窓の手続きは失敗を返せないので、次の描き直しに任せる（ARC-009 の例外は ADR 0021 に記録）。
fn report(result: Result<(), Win32Failure>) {
    let _ignored: Result<(), Win32Failure> = result;
}

/// Esc で閉じ、矢印で動かす（FR-015）。Alt+F4 は Windows の既定（`WM_SYSKEYDOWN` → `WM_CLOSE`）に任せる。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の DestroyWindow / GetKeyState を呼ぶ（ADR 0021）"
)]
fn key_down(session: &FrameSession<'_, '_>, window: HWND, wparam: WPARAM) -> Option<LRESULT> {
    let key = u16::try_from(wparam.0).ok()?;
    if key == VK_ESCAPE.0 {
        // SAFETY: 自分の窓を壊す。WM_DESTROY でループが終わる。
        let _destroyed: Result<(), _> = unsafe { DestroyWindow(window) };
        return Some(LRESULT(0));
    }
    let direction = [
        (VK_LEFT.0, NudgeDirection::Left),
        (VK_RIGHT.0, NudgeDirection::Right),
        (VK_UP.0, NudgeDirection::Up),
        (VK_DOWN.0, NudgeDirection::Down),
    ]
    .into_iter()
    .find_map(|(code, direction)| (code == key).then_some(direction))?;
    // SAFETY: このスレッドの鍵の状態を読むだけ。負なら押されている。
    let fast = unsafe { GetKeyState(i32::from(VK_SHIFT.0)) } < 0;
    report(session.nudge(direction, fast));
    Some(LRESULT(0))
}

/// 当たり判定から `WM_NCHITTEST` の答えへ。
fn hit_code(hit: NonClientHit) -> LRESULT {
    let code = match hit {
        NonClientHit::Caption => isize::try_from(HTCAPTION),
        NonClientHit::TopLeft => isize::try_from(HTTOPLEFT),
        NonClientHit::BottomRight => isize::try_from(HTBOTTOMRIGHT),
        NonClientHit::Client => isize::try_from(HTCLIENT),
        NonClientHit::Transparent => isize::try_from(HTTRANSPARENT),
    };
    LRESULT(code.unwrap_or(0))
}

/// `WM_SETCURSOR`: 当たり判定の答え（lparam の下位語）に合わせてポインタを選ぶ（Issue #36 の Windows 版）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の LoadCursorW / SetCursor を呼ぶ（ADR 0021）"
)]
fn set_cursor(lparam: LPARAM) -> Option<LRESULT> {
    let (code, _mouse) = packed_words(lparam);
    let code = u32::from(code);
    let shape = if code == HTCAPTION {
        IDC_SIZEALL
    } else if code == HTTOPLEFT || code == HTBOTTOMRIGHT {
        IDC_SIZENWSE
    } else if code == HTCLIENT {
        IDC_HAND
    } else {
        return None;
    };
    // SAFETY: 系統の共有のポインタを読み、いまの形にするだけ。
    let cursor = unsafe { LoadCursorW(None, shape) }.ok()?;
    // SAFETY: 同上。
    let _previous = unsafe { SetCursor(Some(cursor)) };
    Some(LRESULT(1))
}

/// `WM_GETMINMAXINFO`: 枠が最小のときの窓より小さくさせない（Issue #52）。
#[expect(
    unsafe_code,
    reason = "RS-010: WM_GETMINMAXINFO の lparam が指す MINMAXINFO を書く（ADR 0021）"
)]
fn limit_size(session: &FrameSession<'_, '_>, lparam: LPARAM) {
    let smallest = session.smallest_window();
    // SAFETY: WM_GETMINMAXINFO の lparam は Windows が用意した MINMAXINFO を指し、この処理の間だけ書ける。
    let info = unsafe {
        ptr::with_exposed_provenance_mut::<MINMAXINFO>(lparam.0.cast_unsigned()).as_mut()
    };
    if let Some(info) = info {
        info.ptMinTrackSize = POINT {
            x: smallest,
            y: smallest,
        };
    }
}

/// ポインタが窓から出たら `WM_MOUSELEAVE` を受け取る。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の TrackMouseEvent を呼ぶ（ADR 0021）"
)]
fn track_leave(window: HWND) {
    let Ok(size) = u32::try_from(size_of::<TRACKMOUSEEVENT>()) else {
        return;
    };
    let mut request = TRACKMOUSEEVENT {
        cbSize: size,
        dwFlags: TME_LEAVE,
        hwndTrack: window,
        dwHoverTime: 0,
    };
    // SAFETY: request はこの関数の局所で、cbSize を正しく入れてある。
    let _tracked: Result<(), _> = unsafe { TrackMouseEvent(&raw mut request) };
}

/// 画面の座標（lparam）を窓の中の座標へ。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の ScreenToClient を呼ぶ（ADR 0021）"
)]
fn client_point(window: HWND, lparam: LPARAM) -> Option<(i32, i32)> {
    let (x, y) = signed_words(lparam);
    let mut point = POINT { x, y };
    // SAFETY: point はこの関数の局所。
    unsafe { ScreenToClient(window, &raw mut point) }
        .as_bool()
        .then_some((point.x, point.y))
}

/// lparam の下位語と上位語（符号なし）。
fn packed_words(lparam: LPARAM) -> (u16, u16) {
    let bits = lparam.0.cast_unsigned();
    let low = u16::try_from(bits & 0xFFFF).unwrap_or(0);
    let high = u16::try_from((bits >> 16_u32) & 0xFFFF).unwrap_or(0);
    (low, high)
}

/// lparam の下位語と上位語（符号付き・座標）。複数のモニタでは負になりうる。
fn signed_words(lparam: LPARAM) -> (i32, i32) {
    let (low, high) = packed_words(lparam);
    (i32::from(low.cast_signed()), i32::from(high.cast_signed()))
}
