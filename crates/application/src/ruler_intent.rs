//! 利用者と WM から届く意図（ARC-011）。

use nene_ruler_core::length_label::LengthLabel;
use nene_ruler_core::modal_theme::ModalTheme;
use nene_ruler_core::rgb::Rgb;
use nene_ruler_core::unit::Unit;
use nene_ruler_core::window_layer::WindowLayer;

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
    /// ギアを押した。設定モーダルを開く（FR-009）。
    OpenSettings,
    /// 設定モーダルを閉じた。
    CloseSettings,
    /// インク色を選んだ（プリセット）。
    ChooseInk(Rgb),
    /// 長さの単位を選んだ（FR-008）。
    ChooseUnit(Unit),
    /// モーダルの見えを選んだ。
    ChooseModalTheme(ModalTheme),
    /// 重なり順を選んだ。
    ChooseLayer(WindowLayer),
    /// ピッカーの四角を押した・引いた。彩度と明度（千分率）。色相はいまのピッカーの色のまま。
    PickShade {
        /// 彩度。
        saturation: u16,
        /// 明度。
        value: u16,
    },
    /// ピッカーの帯を押した・引いた。色相（0.1 度）。
    PickHue(u16),
    /// 入力欄に 1 文字打った。
    HexTyped(char),
    /// 入力欄の末尾を消した。
    HexBackspace,
    /// 入力欄の文字列を色として送った。
    HexSubmit,
    /// 幅・高さ・両方の数字を押した。表示しているその文字列をコピーする（FR-007）。
    CopyText(LengthLabel),
    /// ポインタが数字から離れた。コピーの印を消す。
    ClearCopyMark,
}
