// Copyright © 2026 1o. All rights reserved.

//! `differences_kit`: every kind of history a project keeps, side by side.
//!
//! A project is a folder holding `.custodian/` ([`Project`]). Each kind of
//! history is its own crate; this one composes them into one call per kind,
//! each answering a [`Difference`]: the page that puts the two sides next to
//! each other, the facts it was made from, and, where the difference is one
//! of files, the change records (`nodeplan.1o.diff.v1`) that say it.
//!
//! | kind | what is compared | from |
//! |---|---|---|
//! | `tree` | the tree at two points | `tree_history::tree` |
//! | `count` | two counts (receipt, job, commit or date) | `tree_history::count` |
//! | `receipts` | the receipts since a point, or two receipts | `receipt_history` |
//! | `revisions` | a task's revisions, one to the next | `revision_history` |
//! | `changes` | the lines jobs touched | `change_history` |
//! | `readings` | two readings of one spec | `diff_history` (two counts: `tree_history`) |
//! | `tasks` | every task history and its revisions | `task_history` |

pub mod difference;
pub mod kinds;
pub mod layout;
pub mod refusal;

pub use difference::*;
pub use kinds::Kind;
pub use layout::Project;
pub use refusal::{Error, Refusal, Result};
