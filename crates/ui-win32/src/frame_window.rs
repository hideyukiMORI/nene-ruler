//! 枠の窓と設定モーダルの窓（Windows）。窓を作り、メッセージを状態へ渡し、終わるまで回す（ADR 0021）。
//!
//! 2 つの窓は同じ `WindowsApp` を `RefCell` で共有し、窓に結んだポインタ（`GWLP_USERDATA`）で辿る。
//! 処理の中で同じスレッドの窓へ同期的に届いたメッセージは、借りられないので既定の手続きへ流れる。
//! 移動と伸縮は Windows に任せる（`WM_NCHITTEST` で `HTCAPTION` / `HTTOPLEFT` / `HTBOTTOMRIGHT`）。

use core::cell::RefCell;
use core::ffi::c_void;
use core::ptr;

use nene_ruler_application::ruler_controller::RulerController;
use nene_ruler_core::nudge_direction::NudgeDirection;
use nene_ruler_core::window_layer::WindowLayer;
use windows::Win32::Foundation::{HMODULE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow, ScreenToClient,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, ReleaseCapture, SetCapture, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent, VK_BACK,
    VK_DOWN, VK_ESCAPE, VK_LEFT, VK_RETURN, VK_RIGHT, VK_SHIFT, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CreateWindowExW, DefWindowProcW, DispatchMessageW, GWLP_USERDATA, GetMessageW,
    GetSystemMetrics, GetWindowLongPtrW, GetWindowRect, HICON, HTBOTTOMRIGHT, HTCAPTION, HTCLIENT,
    HTTOPLEFT, HTTRANSPARENT, IDC_ARROW, IDC_HAND, IDC_SIZEALL, IDC_SIZENWSE, IMAGE_ICON,
    LR_DEFAULTCOLOR, LoadCursorW, LoadImageW, MINMAXINFO, MSG, PostQuitMessage, RegisterClassExW,
    SM_CXICON, SM_CXSCREEN, SM_CXSMICON, SM_CYICON, SM_CYSCREEN, SM_CYSMICON, SW_SHOW,
    SYSTEM_METRICS_INDEX, SetCursor, SetWindowLongPtrW, ShowWindow, TranslateMessage,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CHAR, WM_DESTROY, WM_GETMINMAXINFO, WM_KEYDOWN, WM_KILLFOCUS,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCCALCSIZE, WM_NCCREATE, WM_NCHITTEST,
    WM_SETCURSOR, WM_SETFOCUS, WM_SIZE, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
};
use windows::core::{PCWSTR, w};

use crate::modal_focus::ModalFocus;
use crate::modal_key::ModalKey;
use crate::non_client_hit::NonClientHit;
use crate::pen::Pen;
use crate::win32_failure::Win32Failure;
use crate::windows_app::WindowsApp;

/// 窓の類の名と、窓の題（枠はタスクバーと Alt+Tab に出る。モーダルは出さない）。
const FRAME_CLASS: PCWSTR = w!("NeNeRulerFrame");
const MODAL_CLASS: PCWSTR = w!("NeNeRulerSettings");
const FRAME_TITLE: PCWSTR = w!("NeNe Ruler");
const MODAL_TITLE: PCWSTR = w!("NeNe Ruler Settings");
/// `WM_MOUSELEAVE`。windows クレートでは Controls の機能にあるので、値だけをここに置く（winuser.h の 0x02A3）。
const WM_MOUSELEAVE: u32 = 0x02A3;
/// exe に埋めたアイコンの資源番号（`crates/app/build.rs` の `.rc` と同じ・ADR 0023）。
const ICON_RESOURCE: usize = 1;
/// 枠の窓とモーダルの間の隙間。
const MODAL_GAP: i32 = 8;

/// 2 つの窓が共有する状態。窓の手続きはこれを借りる。
type AppCell<'c, 'p> = RefCell<WindowsApp<'c, 'p>>;

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
    reason = "RS-010: Win32 の窓の作成とメッセージループ（ADR 0021）"
)]
pub fn run(controller: &mut RulerController<'_>) -> Result<(), Win32Failure> {
    let frame = controller.frame();
    let window = frame.layout().window();
    let topmost = frame.layer() == WindowLayer::AlwaysOnTop;
    register_classes()?;
    let cell: AppCell<'_, '_> = RefCell::new(WindowsApp::new(controller, Pen::new()?));
    let (width, height) = (i32::from(window.width()), i32::from(window.height()));
    // SAFETY: 主モニタの大きさを読むだけ（dpi 対応の後なので物理画素）。
    let (screen_width, screen_height) =
        unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    let extended = if topmost {
        WS_EX_LAYERED | WS_EX_TOPMOST
    } else {
        WS_EX_LAYERED | WINDOW_EX_STYLE(0)
    };
    let created = create(
        (FRAME_CLASS, FRAME_TITLE),
        (
            extended,
            WS_POPUP | WS_THICKFRAME | WS_SYSMENU | WS_MINIMIZEBOX,
        ),
        centred((screen_width, screen_height), (width, height)),
        (None, &raw const cell),
    )?;
    {
        let mut app = cell.borrow_mut();
        app.attach_frame(created);
        app.present_frame()?;
    }
    // SAFETY: 作った窓を見せる。借りていないので、入れ子のメッセージも状態に届く。
    let _was_visible = unsafe { ShowWindow(created, SW_SHOW) };
    let mut message = MSG::default();
    // SAFETY: このスレッドのメッセージを、WM_QUIT（枠の WM_DESTROY で積む）まで取り出して配る。
    // cell はこの関数の終わりまで生き、窓はループが終わる前に壊れるので、ポインタは生存中だけ辿られる。
    while unsafe { GetMessageW(&raw mut message, None, 0, 0) }.as_bool() {
        // SAFETY: 取り出したばかりのメッセージを訳して配る。
        unsafe {
            let _translated = TranslateMessage(&raw const message);
            DispatchMessageW(&raw const message);
        }
    }
    Ok(())
}

/// 枠とモーダルの窓の類を登録する。アイコンは exe に埋めた資源（タスクバー・Alt+Tab）。
/// 2 度目の登録の失敗は無視する（同じ類）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の RegisterClassExW / LoadCursorW を呼ぶ（ADR 0021）"
)]
fn register_classes() -> Result<(), Win32Failure> {
    // SAFETY: 自分のモジュールのハンドルと、系統の共有の矢印を読むだけ。
    let (instance, arrow) = unsafe { (GetModuleHandleW(None)?, LoadCursorW(None, IDC_ARROW)?) };
    let large = icon(instance, (SM_CXICON, SM_CYICON))?;
    let small = icon(instance, (SM_CXSMICON, SM_CYSMICON))?;
    let size = u32::try_from(size_of::<WNDCLASSEXW>())
        .map_err(|_error| Win32Failure::GeometryOutOfRange)?;
    for (name, cursor) in [(FRAME_CLASS, None), (MODAL_CLASS, Some(arrow))] {
        let class = WNDCLASSEXW {
            cbSize: size,
            lpfnWndProc: Some(procedure),
            hInstance: instance.into(),
            lpszClassName: name,
            hCursor: cursor.unwrap_or_default(),
            hIcon: large,
            hIconSm: small,
            ..WNDCLASSEXW::default()
        };
        // SAFETY: class はこの関数の局所で、登録の間だけ読まれる。
        let _registered: u16 = unsafe { RegisterClassExW(&raw const class) };
    }
    Ok(())
}

/// exe に埋めたアイコンを、系統の大きさ（大か小）で読む。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の LoadImageW / GetSystemMetrics を呼ぶ（ADR 0021）"
)]
fn icon(
    instance: HMODULE,
    (width, height): (SYSTEM_METRICS_INDEX, SYSTEM_METRICS_INDEX),
) -> Result<HICON, Win32Failure> {
    // 資源番号を名前の位置に渡す（MAKEINTRESOURCEW）。指す先は読まれない。
    let name = PCWSTR(ptr::without_provenance(ICON_RESOURCE));
    // SAFETY: 自分のモジュールの資源を読むだけ。返る共有でない icon はプロセスの終わりまで使う。
    let handle = unsafe {
        LoadImageW(
            Some(instance.into()),
            name,
            IMAGE_ICON,
            GetSystemMetrics(width),
            GetSystemMetrics(height),
            LR_DEFAULTCOLOR,
        )
    }?;
    Ok(HICON(handle.0))
}

/// 窓を作る。作成の引数に共有の状態のポインタを渡し、`WM_NCCREATE` で窓に結ぶ。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の CreateWindowExW を呼ぶ（ADR 0021）"
)]
fn create(
    (class, title): (PCWSTR, PCWSTR),
    (extended, style): (WINDOW_EX_STYLE, WINDOW_STYLE),
    rect: RECT,
    (owner, cell): (Option<HWND>, *const AppCell<'_, '_>),
) -> Result<HWND, Win32Failure> {
    // SAFETY: 自分のモジュールのハンドルを読むだけ。
    let instance = unsafe { GetModuleHandleW(None) }?;
    // SAFETY: cell は run の局所の状態で、すべての窓より長く生きる。
    let created = unsafe {
        CreateWindowExW(
            extended,
            class,
            title,
            style,
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
            owner,
            None,
            Some(instance.into()),
            Some(cell.cast::<c_void>()),
        )
    }?;
    Ok(created)
}

/// 窓の手続き。状態を借りられれば処理し、借りられない（入れ子）か扱わない種類なら既定の手続きへ渡す。
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
        // SAFETY: WM_NCCREATE の lparam は CREATESTRUCTW を指し、lpCreateParams は run が渡した状態。
        unsafe {
            let create = ptr::with_exposed_provenance::<CREATESTRUCTW>(lparam.0.cast_unsigned());
            if let Some(create) = create.as_ref() {
                let address = create.lpCreateParams.expose_provenance().cast_signed();
                let _previous = SetWindowLongPtrW(window, GWLP_USERDATA, address);
            }
        }
    }
    // SAFETY: GWLP_USERDATA は WM_NCCREATE で結んだ状態か 0。状態は run の終わりまで生き、
    // 窓はそれより先に壊れる。共有の参照だけを作り、書き換えは RefCell の借用を通す。
    let cell = unsafe {
        ptr::with_exposed_provenance::<AppCell<'static, 'static>>(
            GetWindowLongPtrW(window, GWLP_USERDATA).cast_unsigned(),
        )
        .as_ref()
    };
    let handled = cell.and_then(|cell| {
        let mut app = cell.try_borrow_mut().ok()?;
        dispatch(&mut app, cell, window, (message, wparam, lparam))
    });
    match handled {
        Some(result) => result,
        // SAFETY: 既定の手続きにそのまま渡す。借用は上で手放してある。
        None => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

/// 宛先の窓で振り分ける。`None` は既定の手続きへ渡す印。
fn dispatch(
    app: &mut WindowsApp<'_, '_>,
    cell: &AppCell<'_, '_>,
    window: HWND,
    message: (u32, WPARAM, LPARAM),
) -> Option<LRESULT> {
    if window == app.frame() {
        frame_message(app, cell, window, message)
    } else if app.modal() == Some(window) {
        modal_message(app, window, message)
    } else {
        None
    }
}

/// 枠の窓宛のメッセージ。
fn frame_message(
    app: &mut WindowsApp<'_, '_>,
    cell: &AppCell<'_, '_>,
    window: HWND,
    (message, wparam, lparam): (u32, WPARAM, LPARAM),
) -> Option<LRESULT> {
    match message {
        // 窓全体を中身にする。作成時の最初の 1 回（wparam = 0）にも答える（ADR 0021）。
        WM_NCCALCSIZE => Some(LRESULT(0)),
        WM_NCHITTEST => Some(hit_code(app.frame_hit(client_point(window, lparam)?))),
        WM_SETCURSOR => set_cursor(lparam),
        WM_GETMINMAXINFO => {
            limit_size(app.smallest_window(), lparam);
            Some(LRESULT(0))
        }
        WM_SIZE => {
            let (width, height) = packed_words(lparam);
            report(app.resized(width, height));
            Some(LRESULT(0))
        }
        WM_LBUTTONDOWN => {
            if app.frame_pressed(signed_words(lparam)).unwrap_or(false) {
                report(open_modal(app, cell));
            }
            Some(LRESULT(0))
        }
        WM_MOUSEMOVE => {
            track_leave(window);
            report(app.pointer_moved(Some(signed_words(lparam))));
            Some(LRESULT(0))
        }
        WM_MOUSELEAVE => {
            report(app.pointer_moved(None));
            Some(LRESULT(0))
        }
        WM_KEYDOWN => frame_key(app, wparam),
        WM_DESTROY => {
            post_quit();
            Some(LRESULT(0))
        }
        _other => None,
    }
}

/// 設定モーダルの窓宛のメッセージ。
fn modal_message(
    app: &mut WindowsApp<'_, '_>,
    window: HWND,
    (message, wparam, lparam): (u32, WPARAM, LPARAM),
) -> Option<LRESULT> {
    match message {
        WM_NCHITTEST => Some(hit_code(app.modal_hit(client_point(window, lparam)?))),
        WM_LBUTTONDOWN => {
            capture(window);
            report(app.modal_pressed(signed_words(lparam)).map(|_reply| ()));
            Some(LRESULT(0))
        }
        WM_MOUSEMOVE => {
            report(app.modal_dragged(signed_words(lparam)).map(|_reply| ()));
            Some(LRESULT(0))
        }
        WM_LBUTTONUP => {
            let _was_picking = app.modal_released();
            release_capture();
            Some(LRESULT(0))
        }
        WM_CHAR => {
            let character = u32::try_from(wparam.0).ok().and_then(char::from_u32)?;
            report(app.modal_typed(character).map(|_reply| ()));
            Some(LRESULT(0))
        }
        WM_KEYDOWN => {
            let key = modal_key(wparam)?;
            report(app.modal_key(key).map(|_reply| ()));
            Some(LRESULT(0))
        }
        WM_SETFOCUS => {
            report(app.modal_refocused(ModalFocus::Focused));
            Some(LRESULT(0))
        }
        WM_KILLFOCUS => {
            report(app.modal_refocused(ModalFocus::Unfocused));
            Some(LRESULT(0))
        }
        WM_DESTROY => {
            app.modal_destroyed();
            report(app.present_frame());
            Some(LRESULT(0))
        }
        _other => None,
    }
}

/// 設定モーダルを開く。枠の右（入らなければ左）に、モニタの作業領域に収めて置く。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の窓の位置とモニタの作業領域を読み、窓を見せる（ADR 0021）"
)]
fn open_modal(app: &mut WindowsApp<'_, '_>, cell: &AppCell<'_, '_>) -> Result<(), Win32Failure> {
    app.open_settings();
    let Some(settings) = app.controller().settings_frame() else {
        return Ok(());
    };
    let bounds = settings.layout().window();
    let (width, height) = (i32::from(bounds.width()), i32::from(bounds.height()));
    let mut frame = RECT::default();
    let mut monitor = MONITORINFO {
        cbSize: u32::try_from(size_of::<MONITORINFO>())
            .map_err(|_error| Win32Failure::GeometryOutOfRange)?,
        ..MONITORINFO::default()
    };
    // SAFETY: 枠の窓はこのアプリのもの。frame と monitor はこの関数の局所。
    unsafe {
        GetWindowRect(app.frame(), &raw mut frame)?;
        let nearest = MonitorFromWindow(app.frame(), MONITOR_DEFAULTTONEAREST);
        let _found = GetMonitorInfoW(nearest, &raw mut monitor);
    }
    let work = monitor.rcWork;
    let right_side = frame.right + MODAL_GAP;
    let left = if right_side + width <= work.right {
        right_side
    } else {
        (frame.left - MODAL_GAP - width).max(work.left)
    };
    let top = frame
        .top
        .clamp(work.top, (work.bottom - height).max(work.top));
    let modal = create(
        (MODAL_CLASS, MODAL_TITLE),
        (WS_EX_LAYERED | WS_EX_TOOLWINDOW, WS_POPUP),
        RECT {
            left,
            top,
            right: left + width,
            bottom: top + height,
        },
        (Some(app.frame()), ptr::from_ref(cell)),
    )?;
    app.modal_opened(modal)?;
    // SAFETY: 作ったモーダルを見せる。入れ子のメッセージは借りられず既定へ流れる（焦点は modal_opened が持たせた）。
    let _was_visible = unsafe { ShowWindow(modal, SW_SHOW) };
    Ok(())
}

/// 失敗を握る。窓の手続きは失敗を返せないので、次の描き直しに任せる（ADR 0021 の 11）。
fn report(result: Result<(), Win32Failure>) {
    let _ignored: Result<(), Win32Failure> = result;
}

/// 枠の窓の鍵。Esc で閉じ、矢印で動かす（FR-015）。Alt+F4 は Windows の既定（`WM_CLOSE`）に任せる。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetKeyState を呼ぶ（ADR 0021）"
)]
fn frame_key(app: &WindowsApp<'_, '_>, wparam: WPARAM) -> Option<LRESULT> {
    let key = u16::try_from(wparam.0).ok()?;
    if key == VK_ESCAPE.0 {
        app.close_frame();
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
    report(app.nudge(direction, fast));
    Some(LRESULT(0))
}

/// モーダルが受ける鍵。
fn modal_key(wparam: WPARAM) -> Option<ModalKey> {
    let key = u16::try_from(wparam.0).ok()?;
    [
        (VK_ESCAPE.0, ModalKey::Escape),
        (VK_BACK.0, ModalKey::Backspace),
        (VK_RETURN.0, ModalKey::Return),
    ]
    .into_iter()
    .find_map(|(code, modal_key)| (code == key).then_some(modal_key))
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
fn limit_size(smallest: i32, lparam: LPARAM) {
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

/// ピッカーを押したまま窓の外へ引いても、ポインタの動きを受け取る。
#[expect(unsafe_code, reason = "RS-010: Win32 の SetCapture を呼ぶ（ADR 0021）")]
fn capture(window: HWND) {
    // SAFETY: 自分の窓にポインタを取り込む。
    let _previous = unsafe { SetCapture(window) };
}

/// 取り込みを外す。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の ReleaseCapture を呼ぶ（ADR 0021）"
)]
fn release_capture() {
    // SAFETY: このスレッドの取り込みを外すだけ。
    let _released: Result<(), _> = unsafe { ReleaseCapture() };
}

/// このスレッドのループに `WM_QUIT` を積む。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の PostQuitMessage を呼ぶ（ADR 0021）"
)]
fn post_quit() {
    // SAFETY: 値だけを渡す。
    unsafe {
        PostQuitMessage(0);
    }
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

/// 画面の中央に置いた窓の矩形。
const fn centred((screen_width, screen_height): (i32, i32), (width, height): (i32, i32)) -> RECT {
    let (left, top) = (
        (screen_width - width) / 2_i32,
        (screen_height - height) / 2_i32,
    );
    RECT {
        left,
        top,
        right: left + width,
        bottom: top + height,
    }
}
