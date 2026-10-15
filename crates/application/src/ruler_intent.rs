//! 利用者と WM から届く意図（ARC-011）。

/// 状態を次へ進める入力。UI は値を渡すだけで、解釈はしない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulerIntent {
    /// 窓の大きさが変わった（WM の `ConfigureNotify`）。窓全体の大きさであり、枠の大きさではない。
    WindowResized {
        /// 窓の幅。
        width_px: u16,
        /// 窓の高さ。
        height_px: u16,
    },
}
