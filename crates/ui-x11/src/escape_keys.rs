//! Esc に割り当てられた keycode（FR-015）。

use x11rb::protocol::xproto::{GetKeyboardMappingReply, Keycode};

/// X の keysym `XK_Escape`。
const ESCAPE_KEYSYM: u32 = 0xff1b;

/// キーボード配列から拾った、Esc を出す keycode の集合。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscapeKeys(Vec<Keycode>);

impl EscapeKeys {
    /// `GetKeyboardMapping` の応答から拾う。`first` はその応答の最初の keycode。
    #[must_use]
    pub fn from_mapping(first: Keycode, mapping: &GetKeyboardMappingReply) -> Self {
        let per_keycode = usize::from(mapping.keysyms_per_keycode.max(1));
        let keycodes = mapping
            .keysyms
            .chunks(per_keycode)
            .enumerate()
            .filter(|(_, keysyms)| keysyms.contains(&ESCAPE_KEYSYM))
            .filter_map(|(index, _)| u8::try_from(index).ok())
            .map(|index| first.saturating_add(index))
            .collect();
        Self(keycodes)
    }

    /// この keycode は Esc か。
    #[must_use]
    pub fn contains(&self, keycode: Keycode) -> bool {
        self.0.contains(&keycode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_escape_among_the_keysyms() {
        let mapping = GetKeyboardMappingReply {
            keysyms_per_keycode: 2,
            sequence: 0,
            keysyms: vec![0, 0, ESCAPE_KEYSYM, 0, 0x61, 0x41],
        };
        let keys = EscapeKeys::from_mapping(8, &mapping);
        assert!(keys.contains(9));
        assert!(!keys.contains(8));
        assert!(!keys.contains(10));
    }
}
