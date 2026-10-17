//! WM が最前面（`_NET_WM_STATE_ABOVE`）に対応しているか（FR-013 / ADR 0008）。

/// 起動時に `_NET_SUPPORTED` から一度取り込む。application が所有する（ARC-004）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layering {
    /// WM が `_NET_WM_STATE_ABOVE` を挙げている。
    Supported,
    /// 挙げていない（WSLg の XWM など）。最前面は効かず、そのことを示す（FR-014）。
    Unsupported,
}
