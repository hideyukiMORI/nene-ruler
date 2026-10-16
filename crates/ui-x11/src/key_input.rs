//! keysym を、この製品が反応する鍵に読み替える。

/// `XK_Escape`。
const ESCAPE: u32 = 0xff1b;
/// `XK_Return`。
const RETURN: u32 = 0xff0d;
/// `XK_KP_Enter`。
const KP_ENTER: u32 = 0xff8d;
/// `XK_BackSpace`。
const BACKSPACE: u32 = 0xff08;

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
}
