//! 利用者と WM から届く意図（ARC-011）。

use nene_ruler_core::axis::Axis;
use nene_ruler_core::modal_theme::ModalTheme;
use nene_ruler_core::rgb::Rgb;
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
    /// モーダルの見えを選んだ。
    ChooseModalTheme(ModalTheme),
    /// 重なり順を選んだ。
    ChooseLayer(WindowLayer),
    /// 入力欄に 1 文字打った。
    HexTyped(char),
    /// 入力欄の末尾を消した。
    HexBackspace,
    /// 入力欄の文字列を色として送った。
    HexSubmit,
    /// 幅か高さの数字を押した。表示しているその文字列をコピーする（FR-007）。
    CopyText(Axis),
    /// ポインタが数字から離れた。コピーの印を消す。
    ClearCopyMark,
}
