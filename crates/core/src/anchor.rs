//! 伸縮の取っ手（鉤括弧）。

/// 掴んで伸縮する角。左上は原点が動き、右下は幅と高さが動く（FR-005）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// 左上の鉤括弧。
    TopLeft,
    /// 右下の鉤括弧。すぐ外側にギアがある（FR-009）。
    BottomRight,
}
