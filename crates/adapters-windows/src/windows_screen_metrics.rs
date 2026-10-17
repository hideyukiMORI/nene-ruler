//! 主モニタの画素数と dpi（FR-005 / FR-016・ADR 0021）。

use nene_ruler_application::screen_metrics_port::ScreenMetricsPort;
use nene_ruler_core::screen_metrics::ScreenMetrics;
use nene_ruler_core::screen_metrics_failure::ScreenMetricsFailure;
use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, HMONITOR, MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MonitorFromPoint,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

/// 主モニタ（原点を含むモニタ）。プロセスは per-monitor v2 の dpi 対応なので、値は物理画素。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsScreenMetrics;

impl WindowsScreenMetrics {
    /// 主モニタの dpi。UI の倍率の元（`Environment::with_desktop_dpi`）。取れなければ `None`。
    #[must_use]
    pub fn desktop_dpi() -> Option<u32> {
        let (horizontal, _vertical) = monitor_dpi(primary_monitor())?;
        Some(horizontal)
    }
}

impl ScreenMetricsPort for WindowsScreenMetrics {
    fn screen_metrics(&self) -> Result<ScreenMetrics, ScreenMetricsFailure> {
        let rect = monitor_rect(primary_monitor()).ok_or(ScreenMetricsFailure::Unavailable)?;
        let width = u32::try_from(rect.right - rect.left)
            .map_err(|_error| ScreenMetricsFailure::ZeroPixels)?;
        let height = u32::try_from(rect.bottom - rect.top)
            .map_err(|_error| ScreenMetricsFailure::ZeroPixels)?;
        // ミリ寸法は使わない（ADR 0012）。dpi は GetDpiForMonitor から別に渡す。
        ScreenMetrics::new(width, height, 0)
    }
}

/// 原点を含むモニタ。無ければ主モニタ。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の MonitorFromPoint を呼ぶ（ADR 0021）"
)]
fn primary_monitor() -> HMONITOR {
    // SAFETY: 引数は値だけで、ポインタを渡さない。MONITOR_DEFAULTTOPRIMARY なので必ずモニタが返る。
    unsafe { MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY) }
}

/// モニタの矩形（仮想画面の物理画素）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetMonitorInfoW を呼ぶ（ADR 0021）"
)]
fn monitor_rect(monitor: HMONITOR) -> Option<RECT> {
    let mut info = MONITORINFO {
        cbSize: u32::try_from(size_of::<MONITORINFO>()).ok()?,
        rcMonitor: RECT::default(),
        rcWork: RECT::default(),
        dwFlags: 0,
    };
    // SAFETY: info はこの関数の局所で、cbSize を正しく入れてある。呼び出しの間だけ書かれる。
    let found = unsafe { GetMonitorInfoW(monitor, &raw mut info) };
    found.as_bool().then_some(info.rcMonitor)
}

/// モニタの実効 dpi（横・縦）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetDpiForMonitor を呼ぶ（ADR 0021）"
)]
fn monitor_dpi(monitor: HMONITOR) -> Option<(u32, u32)> {
    let (mut horizontal, mut vertical) = (0_u32, 0_u32);
    // SAFETY: 出力の 2 つはこの関数の局所の u32 で、呼び出しの間だけ書かれる。
    unsafe {
        GetDpiForMonitor(
            monitor,
            MDT_EFFECTIVE_DPI,
            &raw mut horizontal,
            &raw mut vertical,
        )
    }
    .ok()?;
    Some((horizontal, vertical))
}
