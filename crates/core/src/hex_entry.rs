//! `#RRGGBB` の直接入力の編集状態（FR-010）。

use alloc::string::String;

use crate::rgb::Rgb;
use crate::rgb_failure::RgbFailure;

/// `#` ＋ 6 桁。入力は 16 進の文字だけを受け、長さは 7 を超えない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexEntry {
    text: String,
}

impl HexEntry {
    /// 受け入れる最大の長さ（`#RRGGBB`）。
    const MAX_CHARS: usize = 7;

    /// いまの色の文字列から始める。
    #[must_use]
    pub fn showing(rgb: Rgb) -> Self {
        Self { text: rgb.hex() }
    }

    /// 空から始める。
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            text: String::new(),
        }
    }

    /// 1 文字足した状態。16 進の文字と先頭の `#` 以外、7 文字を超える分は無視する。
    #[must_use]
    pub fn typed(&self, character: char) -> Self {
        let accepted = character.is_ascii_hexdigit() || (character == '#' && self.text.is_empty());
        if !accepted || self.text.chars().count() >= Self::MAX_CHARS {
            return self.clone();
        }
        let mut text = self.text.clone();
        text.push(character.to_ascii_uppercase());
        Self { text }
    }

    /// 末尾を 1 文字消した状態。
    #[must_use]
    pub fn backspaced(&self) -> Self {
        let mut text = self.text.clone();
        text.pop();
        Self { text }
    }

    /// いまの文字列を色として読む。`#` が無ければ補う。
    ///
    /// # Errors
    ///
    /// 桁数が違う・16 進でない。
    pub fn parse(&self) -> Result<Rgb, RgbFailure> {
        if self.text.starts_with('#') {
            Rgb::parse(&self.text)
        } else {
            let mut prefixed = String::from("#");
            prefixed.push_str(&self.text);
            Rgb::parse(&prefixed)
        }
    }

    /// 表示する文字列。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_hex_digits_up_to_seven_characters() {
        let entry = "#1a2B3c9"
            .chars()
            .fold(HexEntry::empty(), |entry, c| entry.typed(c));
        assert_eq!(entry.text(), "#1A2B3C");
        assert_eq!(entry.parse(), Ok(Rgb::new(0x1A, 0x2B, 0x3C)));
    }

    #[test]
    fn ignores_other_characters_and_a_late_hash() {
        let entry = HexEntry::empty()
            .typed('g')
            .typed('1')
            .typed('#')
            .typed('2');
        assert_eq!(entry.text(), "12");
    }

    #[test]
    fn supplies_the_hash_when_parsing() {
        let entry = "E53935"
            .chars()
            .fold(HexEntry::empty(), |entry, c| entry.typed(c));
        assert_eq!(entry.parse(), Ok(Rgb::INITIAL_INK));
    }

    #[test]
    fn rejects_short_input_and_can_backspace() {
        let entry = HexEntry::showing(Rgb::INITIAL_INK).backspaced();
        assert_eq!(entry.text(), "#E5393");
        assert_eq!(entry.parse(), Err(RgbFailure::WrongLength));
        assert_eq!(HexEntry::empty().backspaced().text(), "");
    }
}
