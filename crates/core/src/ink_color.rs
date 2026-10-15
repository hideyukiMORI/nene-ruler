//! 枠の窓に描くものすべてに使う 1 色（FR-018）。

use alloc::format;
use alloc::string::String;

use crate::ink_color_failure::InkColorFailure;

/// 不透明な RGB。`#RRGGBB` で読み書きする（FR-010 の直接入力と設定の保存）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InkColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl InkColor {
    /// 既定のインク色（SPECIFICATION 第 2 節。決め打ちしない）。
    pub const INITIAL: Self = Self::new(0xE5, 0x39, 0x35);

    /// 成分から作る。どの値も有効なので失敗しない。
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// `#RRGGBB`（大文字小文字は問わない）を読む。
    ///
    /// # Errors
    ///
    /// `#` が無い・桁数が違う・16 進でない、のいずれか。
    pub fn parse(text: &str) -> Result<Self, InkColorFailure> {
        let digits = text.strip_prefix('#').ok_or(InkColorFailure::MissingHash)?;
        if digits.chars().count() != 6 {
            return Err(InkColorFailure::WrongLength);
        }
        let value =
            u32::from_str_radix(digits, 16).map_err(|_error| InkColorFailure::NotHexadecimal)?;
        let [_, red, green, blue] = value.to_be_bytes();
        Ok(Self::new(red, green, blue))
    }

    /// `#RRGGBB`（大文字）。設定の保存とモーダルの表示に使う。
    #[must_use]
    pub fn hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }

    /// 赤。
    #[must_use]
    pub const fn red(self) -> u8 {
        self.red
    }

    /// 緑。
    #[must_use]
    pub const fn green(self) -> u8 {
        self.green
    }

    /// 青。
    #[must_use]
    pub const fn blue(self) -> u8 {
        self.blue
    }

    /// 合成器が無く不透明に落ちるときの背景（ADR 0005 の 5）。明るいインクには黒、暗いインクには白。
    #[must_use]
    pub fn contrasting_background(self) -> Self {
        let luma = 299_u32 * u32::from(self.red)
            + 587_u32 * u32::from(self.green)
            + 114_u32 * u32::from(self.blue);
        if luma > 128_000 {
            Self::new(0, 0, 0)
        } else {
            Self::new(0xFF, 0xFF, 0xFF)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_hex() {
        let ink = InkColor::parse("#e53935").unwrap();
        assert_eq!(ink, InkColor::INITIAL);
        assert_eq!(ink.hex(), "#E53935");
        assert_eq!((ink.red(), ink.green(), ink.blue()), (0xE5, 0x39, 0x35));
    }

    #[test]
    fn rejects_malformed_text() {
        assert_eq!(InkColor::parse("E53935"), Err(InkColorFailure::MissingHash));
        assert_eq!(InkColor::parse("#E539"), Err(InkColorFailure::WrongLength));
        assert_eq!(
            InkColor::parse("#E5393G"),
            Err(InkColorFailure::NotHexadecimal)
        );
    }

    #[test]
    fn picks_a_contrasting_background() {
        assert_eq!(
            InkColor::new(0xFF, 0xFF, 0xFF).contrasting_background(),
            InkColor::new(0, 0, 0)
        );
        assert_eq!(
            InkColor::new(0, 0, 0x40).contrasting_background(),
            InkColor::new(0xFF, 0xFF, 0xFF)
        );
    }
}
