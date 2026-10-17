//! 設定モーダルの操作部品の意味。

use crate::modal_theme::ModalTheme;
use crate::window_layer::WindowLayer;

/// クリックした部品が何か。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsControl {
    /// プリセットの色見本。`Rgb::PRESETS` の添字。
    Preset(u8),
    /// `#RRGGBB` の入力欄。
    HexField,
    /// モーダルの見えの選択。
    Theme(ModalTheme),
    /// 重なり順の切替。
    Layer(WindowLayer),
    /// 右上の閉じるボタン（WM の飾りの代わり・ADR 0014）。
    Close,
}
