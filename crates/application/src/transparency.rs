//! 透明に描けるかどうか（FR-017 / ADR 0005）。

/// 合成器の有無で決まる。起動時に一度取り込み、application が所有する（ARC-004）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transparency {
    /// 合成器がある。描いたもの以外はアルファ 0。
    Composited,
    /// 合成器が無い。背景をインク色の補色で塗る。利用者に示す（FR-014）。
    Opaque,
}
