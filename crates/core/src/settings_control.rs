//! 設定モーダルの操作部品の意味。

use crate::modal_theme::ModalTheme;
use crate::unit::Unit;
use crate::window_layer::WindowLayer;

/// クリックした部品が何か。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsControl {
    /// プリセットの色見本。`Rgb::PRESETS` の添字。
    Preset(u8),
    /// `#RRGGBB` の入力欄。
    HexField,
    /// カラーピッカーの彩度（横）× 明度（縦）の四角。値は押した位置から決まる（ADR 0016）。
    Shade,
    /// カラーピッカーの色相の帯。値は押した位置から決まる。
    Hue,
    /// 長さの単位の選択（FR-008・ADR 0027）。
    Unit(Unit),
    /// モーダルの見えの選択。
    Theme(ModalTheme),
    /// 重なり順の切替。
    Layer(WindowLayer),
    /// 右上の閉じるボタン（WM の飾りの代わり・ADR 0014）。
    Close,
    /// アプリを終える（施主指示 2026-10-17・Issue #42）。
    Quit,
}
