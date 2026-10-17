//! Alt+F4 を、F4 の押下でも離しでも読む（ADR 0009）。

use crate::key_input::KeyInput;

/// この窓が Alt の押下を受けたかを覚え、Alt+F4 で閉じるかを決める。
///
/// `WSLg` では Windows が Alt+F4 の押下を握り、X の窓には F4 の離しだけが届く（ADR 0009）。
/// 離しだけで閉じると、別の窓を Alt+F4 で閉じた直後に焦点が移ってきた窓まで閉じるので、
/// この窓が Alt の押下を見ていることを条件にする。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AltF4Gesture {
    alt_pressed_here: bool,
}

impl AltF4Gesture {
    /// まだ Alt の押下を見ていない。
    pub const IDLE: Self = Self {
        alt_pressed_here: false,
    };

    /// 鍵の押下。閉じるなら `true`。`alt_held` は `state` の `Mod1`。
    pub const fn pressed(&mut self, input: KeyInput, alt_held: bool) -> bool {
        match input {
            KeyInput::Alt => {
                self.alt_pressed_here = true;
                false
            }
            KeyInput::F4 => alt_held,
            KeyInput::Escape
            | KeyInput::Return
            | KeyInput::Backspace
            | KeyInput::Character(_)
            | KeyInput::Arrow(_)
            | KeyInput::Other => false,
        }
    }

    /// 鍵の離し。閉じるなら `true`。`alt_held` は `state` の `Mod1`（離す直前の状態）。
    pub const fn released(&mut self, input: KeyInput, alt_held: bool) -> bool {
        match input {
            KeyInput::Alt => {
                self.alt_pressed_here = false;
                false
            }
            KeyInput::F4 => self.alt_pressed_here && alt_held,
            KeyInput::Escape
            | KeyInput::Return
            | KeyInput::Backspace
            | KeyInput::Character(_)
            | KeyInput::Arrow(_)
            | KeyInput::Other => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closes_on_the_f4_press_with_alt() {
        let mut gesture = AltF4Gesture::IDLE;
        assert!(gesture.pressed(KeyInput::F4, true));
        assert!(!gesture.pressed(KeyInput::F4, false));
    }

    #[test]
    fn closes_on_the_f4_release_after_this_window_saw_alt() {
        // WSLg の実測の並び: Alt 押下 → F4 離し（Mod1）→ Alt 離し。
        let mut gesture = AltF4Gesture::IDLE;
        assert!(!gesture.pressed(KeyInput::Alt, false));
        assert!(gesture.released(KeyInput::F4, true));
    }

    #[test]
    fn ignores_an_f4_release_without_the_alt_press_here() {
        // 別の窓で Alt+F4 を押し、焦点が移ってから F4 の離しだけが届いた。
        let mut gesture = AltF4Gesture::IDLE;
        assert!(!gesture.released(KeyInput::F4, true));
    }

    #[test]
    fn forgets_alt_once_it_is_released() {
        let mut gesture = AltF4Gesture::IDLE;
        assert!(!gesture.pressed(KeyInput::Alt, false));
        assert!(!gesture.released(KeyInput::Alt, true));
        assert!(!gesture.released(KeyInput::F4, true));
    }

    #[test]
    fn needs_mod1_on_the_f4_release() {
        let mut gesture = AltF4Gesture::IDLE;
        assert!(!gesture.pressed(KeyInput::Alt, false));
        assert!(!gesture.released(KeyInput::F4, false));
    }

    #[test]
    fn other_keys_never_close() {
        let mut gesture = AltF4Gesture::IDLE;
        assert!(!gesture.pressed(KeyInput::Alt, false));
        assert!(!gesture.pressed(KeyInput::Escape, true));
        assert!(!gesture.released(KeyInput::Other, true));
        assert!(!gesture.released(KeyInput::Character('a'), true));
    }
}
