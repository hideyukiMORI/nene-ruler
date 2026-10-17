//! `SelectionRequest` への答えを決める純関数（ADR 0007 の 3）。X 無しでテストする。

use x11rb::protocol::xproto::{Atom, AtomEnum};

use crate::selection_atoms::SelectionAtoms;

/// 要求された target に対して何を書くか。閉じた集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionAnswer {
    /// `TARGETS`: 答えられる target の一覧（`ATOM` の列）。
    Targets,
    /// 文字列を、要求された型で。
    Text(Atom),
    /// 答えられない。`property = NONE` で知らせる。
    Refuse,
}

impl SelectionAnswer {
    /// target から決める。`STRING` は Latin-1 だが、置く文字列は ASCII なのでそのまま答える。
    #[must_use]
    pub fn decide(target: Atom, atoms: SelectionAtoms) -> Self {
        let string: Atom = AtomEnum::STRING.into();
        if target == atoms.targets {
            Self::Targets
        } else if target == atoms.utf8_string || target == string {
            Self::Text(target)
        } else if target == atoms.text {
            Self::Text(atoms.utf8_string)
        } else {
            Self::Refuse
        }
    }

    /// `TARGETS` に答える一覧。順は固定（RS-016）。
    #[must_use]
    pub fn targets(atoms: SelectionAtoms) -> [Atom; 3] {
        [atoms.targets, atoms.utf8_string, AtomEnum::STRING.into()]
    }
}

/// 要求の `property` が `NONE` なら（古い要求者）、`target` をそのまま property に使う（ICCCM）。
#[must_use]
pub fn reply_property(requested: Atom, target: Atom) -> Atom {
    if requested == Atom::from(AtomEnum::NONE) {
        target
    } else {
        requested
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atoms() -> SelectionAtoms {
        SelectionAtoms {
            clipboard: 100,
            targets: 101,
            utf8_string: 102,
            text: 103,
        }
    }

    #[test]
    fn answers_each_known_target() {
        assert_eq!(
            SelectionAnswer::decide(101, atoms()),
            SelectionAnswer::Targets
        );
        assert_eq!(
            SelectionAnswer::decide(102, atoms()),
            SelectionAnswer::Text(102)
        );
        assert_eq!(
            SelectionAnswer::decide(AtomEnum::STRING.into(), atoms()),
            SelectionAnswer::Text(AtomEnum::STRING.into())
        );
        assert_eq!(
            SelectionAnswer::decide(103, atoms()),
            SelectionAnswer::Text(102)
        );
        assert_eq!(
            SelectionAnswer::decide(999, atoms()),
            SelectionAnswer::Refuse
        );
    }

    #[test]
    fn lists_the_targets_it_can_answer() {
        assert_eq!(
            SelectionAnswer::targets(atoms()),
            [101, 102, AtomEnum::STRING.into()]
        );
    }

    #[test]
    fn falls_back_to_the_target_as_property() {
        assert_eq!(reply_property(AtomEnum::NONE.into(), 102), 102);
        assert_eq!(reply_property(555, 102), 555);
    }
}
