// Copyright © 2026 1o. All rights reserved.

//! Every refusal `differences_kit` gives, declared once as liaise codes; a
//! history crate's refusal arrives as [`Refusal::History`] with its own code
//! in the detail.

use liaise::{Liaise, LiaiseCodes};

#[derive(Debug, Clone, Copy, PartialEq, Eq, LiaiseCodes)]
#[liaise(prefix = "DIFS", domain = "differences_kit")]
pub enum Refusal {
    #[liaise(code = 1, msg = "no initialized folder (.custodian) here or above")]
    NoProject,
    #[liaise(code = 2, msg = "a history refused")]
    History,
    #[liaise(code = 3, msg = "no such receipt in the journal")]
    NoReceipt,
    #[liaise(code = 4, msg = "the spec has no reading by that receipt number")]
    NoReading,
    #[liaise(code = 5, msg = "nothing to compare: this needs two")]
    NothingToCompare,
    #[liaise(code = 6, msg = "no such job in the journal")]
    NoJob,
    #[liaise(code = 7, msg = "not a kind of history")]
    NoKind,
}

impl Refusal {
    /// The error this refusal raises, with what was refused.
    pub fn refuse(self, detail: impl std::fmt::Display) -> Error {
        Error { refusal: self, detail: detail.to_string() }
    }
}

/// A refusal and what was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub refusal: Refusal,
    pub detail: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.refusal.code(), self.refusal.message(), self.detail)
    }
}

impl std::error::Error for Error {}

macro_rules! from_history {
    ($($e:ty),*) => {$(
        impl From<$e> for Error {
            fn from(e: $e) -> Self {
                Refusal::History.refuse(e)
            }
        }
    )*};
}

from_history!(receipt_history::Error, revision_history::Error, tree_history::Error, change_history::Error, diff_history::Error);

pub type Result<T> = std::result::Result<T, Error>;
