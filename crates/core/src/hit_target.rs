//! 当たり判定の意味。

use crate::anchor::Anchor;
use crate::axis::Axis;

/// クリックした場所が何を意味するか。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTarget {
    /// 線の帯。枠全体の移動（FR-002）。
    Move,
    /// 鉤括弧。伸縮（FR-005）。
    Resize(Anchor),
    /// ギア。設定モーダルを開く（FR-009）。
    Gear,
    /// 幅または高さの表示。その文字列をコピーする（FR-007）。
    Copy(Axis),
}
