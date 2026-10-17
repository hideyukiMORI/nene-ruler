//! keysym を、この製品が反応する鍵に読み替える。

use nene_ruler_core::nudge_direction::NudgeDirection;

/// `XK_Escape`。
const ESCAPE: u32 = 0xff1b;
/// `XK_Return`。
const RETURN: u32 = 0xff0d;
/// `XK_KP_Enter`。
const KP_ENTER: u32 = 0xff8d;
/// `XK_BackSpace`。
const BACKSPACE: u32 = 0xff08;
/// `XK_F4`。Alt と一緒なら閉じる（ADR 0009）。
const F4: u32 = 0xffc1;
/// `XK_Alt_L` / `XK_Alt_R`。Alt+F4 の押下を見たかを覚える（ADR 0009）。
const ALT_L: u32 = 0xffe9;
const ALT_R: u32 = 0xffea;
/// `XK_Left` / `XK_Up` / `XK_Right` / `XK_Down`。
const LEFT: u32 = 0xff51;
const UP: u32 = 0xff52;
const RIGHT: u32 = 0xff53;
const DOWN: u32 = 0xff54;

/// 反応する鍵の閉じた集合。それ以外は `Other`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyInput {
    /// 閉じる。
    Escape,
    /// 入力欄を送る。
    Return,
    /// 入力欄の末尾を消す。
    Backspace,
    /// 印字できる ASCII の 1 文字。
    Character(char),
    /// 矢印。枠を動かす（FR-015）。
    Arrow(NudgeDirection),
    /// F4。Alt と一緒なら閉じる（FR-015）。
    F4,
    /// Alt。WSLg では F4 の押下が届かないので、Alt の押下を覚える（ADR 0009）。
    Alt,
    /// 反応しない鍵。
    Other,
}

impl KeyInput {
    /// keysym から。印字できる ASCII は keysym の値がそのまま文字になる。
    #[must_use]
    pub fn from_keysym(keysym: u32) -> Self {
        match keysym {
            ESCAPE => Self::Escape,
            RETURN | KP_ENTER => Self::Return,
            BACKSPACE => Self::Backspace,
            F4 => Self::F4,
            ALT_L | ALT_R => Self::Alt,
            LEFT => Self::Arrow(NudgeDirection::Left),
            UP => Self::Arrow(NudgeDirection::Up),
            RIGHT => Self::Arrow(NudgeDirection::Right),
            DOWN => Self::Arrow(NudgeDirection::Down),
            0x20..=0x7e => char::from_u32(keysym).map_or(Self::Other, Self::Character),
            _ => Self::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_the_keys_the_product_uses() {
        assert_eq!(KeyInput::from_keysym(0xff1b), KeyInput::Escape);
        assert_eq!(KeyInput::from_keysym(0xff0d), KeyInput::Return);
        assert_eq!(KeyInput::from_keysym(0xff8d), KeyInput::Return);
        assert_eq!(KeyInput::from_keysym(0xff08), KeyInput::Backspace);
        assert_eq!(KeyInput::from_keysym(0x23), KeyInput::Character('#'));
        assert_eq!(KeyInput::from_keysym(0x46), KeyInput::Character('F'));
        assert_eq!(KeyInput::from_keysym(0xffe1), KeyInput::Other);
    }

    #[test]
    fn maps_the_arrows_and_f4() {
        assert_eq!(
            KeyInput::from_keysym(0xff51),
            KeyInput::Arrow(NudgeDirection::Left)
        );
        assert_eq!(
            KeyInput::from_keysym(0xff54),
            KeyInput::Arrow(NudgeDirection::Down)
        );
        assert_eq!(KeyInput::from_keysym(0xffc1), KeyInput::F4);
        assert_eq!(KeyInput::from_keysym(0xffe9), KeyInput::Alt);
        assert_eq!(KeyInput::from_keysym(0xffea), KeyInput::Alt);
    }
}
