//! 枠をどこまで描くか（ADR 0017）。

/// 目盛と数字を全部描くか、詰めて線と `W × H px` だけにするか。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameDetail {
    /// 目盛・目盛の数字・幅と高さの数字・通知を描く。
    Full,
    /// 幅と高さの数字が目盛を避けた内側に収まらない。目盛と通知を描かず、中央に `W × H px`。
    Compact,
}
