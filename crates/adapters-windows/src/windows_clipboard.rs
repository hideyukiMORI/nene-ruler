//! Windows のクリップボードへ文字列を置く（FR-007・ADR 0021）。

use core::iter;
use core::ptr;

use nene_ruler_application::clipboard_failure::ClipboardFailure;
use nene_ruler_application::clipboard_port::ClipboardPort;
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};
use windows::Win32::System::Ole::CF_UNICODETEXT;

/// `CF_UNICODETEXT` で置く。X11 と違い、置いた後の所有を保つスレッドは要らない（OS が預かる）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsClipboard;

impl ClipboardPort for WindowsClipboard {
    fn copy(&self, text: &str) -> Result<(), ClipboardFailure> {
        let wide: Vec<u16> = text.encode_utf16().chain(iter::once(0)).collect();
        let memory = global_text(&wide).ok_or(ClipboardFailure::Unavailable)?;
        hand_over(memory).ok_or(ClipboardFailure::Rejected)
    }
}

/// 文字列（NUL 終端の UTF-16）を、クリップボードへ渡せる移動可能なメモリに写す。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GlobalAlloc / GlobalLock と生ポインタの写しを使う（ADR 0021）"
)]
fn global_text(wide: &[u16]) -> Option<HGLOBAL> {
    let bytes = wide.len().checked_mul(size_of::<u16>())?;
    // SAFETY: 大きさは wide の全体。返った HGLOBAL はこの関数か hand_over が必ず手放す。
    let memory = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes) }.ok()?;
    // SAFETY: memory は直前に確保した移動可能なメモリで、ロックの間だけ固定のアドレスを持つ。
    let target = unsafe { GlobalLock(memory) }.cast::<u16>();
    if target.is_null() {
        // SAFETY: memory はまだ誰にも渡していない。失敗しても手元のメモリが残るだけ。
        let _freed: Result<HGLOBAL, _> = unsafe { GlobalFree(Some(memory)) };
        return None;
    }
    // SAFETY: target は bytes 分の書き込み可能な領域で、wide とは重ならない（別に確保した）。
    unsafe {
        ptr::copy_nonoverlapping(wide.as_ptr(), target, wide.len());
    }
    // SAFETY: ロックを 1 回取ったので 1 回外す。0 になったときの「失敗」は正常（ロックが外れた印）。
    let _unlocked: Result<(), _> = unsafe { GlobalUnlock(memory) };
    Some(memory)
}

/// クリップボードを開けて中身を置き換える。成功したらメモリは OS の持ち物になる。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の OpenClipboard / SetClipboardData を呼ぶ（ADR 0021）"
)]
fn hand_over(memory: HGLOBAL) -> Option<()> {
    // SAFETY: 持ち主の窓なし（None）で開く。開けたら必ず CloseClipboard で閉じる。
    if unsafe { OpenClipboard(None) }.is_err() {
        // SAFETY: memory はまだ OS に渡していない。
        let _freed: Result<HGLOBAL, _> = unsafe { GlobalFree(Some(memory)) };
        return None;
    }
    // SAFETY: クリップボードを開けている間に空にして置く。SetClipboardData が成功したら memory を手放さない。
    let placed = unsafe {
        EmptyClipboard()
            .and_then(|()| SetClipboardData(u32::from(CF_UNICODETEXT.0), Some(HANDLE(memory.0))))
    };
    if placed.is_err() {
        // SAFETY: 置けなかったので memory はまだ手元にある。
        let _freed: Result<HGLOBAL, _> = unsafe { GlobalFree(Some(memory)) };
    }
    // SAFETY: 上で開けたクリップボードを閉じる。
    let _closed: Result<(), _> = unsafe { CloseClipboard() };
    placed.ok().map(|_handle| ())
}
