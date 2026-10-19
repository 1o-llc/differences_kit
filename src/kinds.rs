// Copyright © 2026 1o. All rights reserved.

//! The kinds of history, as one word.

use crate::refusal::Refusal;
use big_bad_bool::bitpack;

/// Which history a difference is of.
#[bitpack(storage = u8, version = 1)]
pub struct Kind {
    #[bits(3)]
    #[describe("which history a difference is of")]
    #[labels(tree = 0, count = 1, receipts = 2, revisions = 3, changes = 4, readings = 5, tasks = 6)]
    #[invalid(7)]
    kind: u8,
}

impl Kind {
    /// The kind a word names.
    pub fn of_word(word: &str) -> crate::Result<Kind> {
        Self::LABELS
            .iter()
            .find(|l| l.field == "kind" && l.label == word)
            .map(|l| Self::from_raw(l.value as u8))
            .ok_or_else(|| Refusal::NoKind.refuse(format!("`{word}`: one of {}", Self::words().join(", "))))
    }

    /// Every kind's word, in order.
    pub fn words() -> Vec<&'static str> {
        Self::LABELS.iter().filter(|l| l.field == "kind").map(|l| l.label).collect()
    }

    pub fn word(self) -> &'static str {
        Self::LABELS.iter().find(|l| l.field == "kind" && l.value == self.kind() as u128).map(|l| l.label).unwrap_or("tree")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_kinds_are_words() {
        assert_eq!(Kind::words(), ["tree", "count", "receipts", "revisions", "changes", "readings", "tasks"]);
        assert_eq!(Kind::of_word("revisions").unwrap().word(), "revisions");
        assert_eq!(Kind::of_word("moods").unwrap_err().refusal, Refusal::NoKind);
    }
}
