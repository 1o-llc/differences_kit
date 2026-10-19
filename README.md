# differences_kit

Every kind of history a project keeps, side by side. A project is a folder holding `.custodian/` (`Project::find`
walks up to it; the files it reads are named through pathgen as custodian_kit names them). Each kind of history is
its own crate; this one composes them into one call per kind, each answering a `Difference`: the page that puts
the two sides next to each other, a line said about it, whether the sides differ, and, where the difference is one
of files, the change records (`nodeplan.1o.diff.v1`) that say it.

| kind | call | what is compared | from |
|---|---|---|---|
| `tree` | `tree_take`, `tree`, `trees` | the tree at two points (its line's receipt kept in the journal) | tree_history |
| `count` | `count`, `count_history` | two counts by receipt, job, commit or date | tree_history |
| `receipts` | `receipts_since`, `receipts_pair` | the receipts since a point, or two receipts field by field | receipt_history |
| `revisions` | `revisions` | a task's revisions, one to the next | revision_history |
| `changes` | `changes` | the lines jobs touched, side by side and together | change_history |
| `readings` | `readings` | two readings of one spec (two counts: by file) | diff_history, tree_history |
| `tasks` | `tasks` | every task history and its revisions | task_history |

```rust,no_run
# fn demo() -> differences_kit::Result<()> {
let p = differences_kit::Project::find(".".as_ref())?;
let d = differences_kit::count(&p, "32", "52")?;
print!("{}", d.page);
# Ok(()) }
```

custodian answers `count history` through it, and the `differences` command line is it on the line.

## Refusals

`DIFS0001`…`DIFS0007` (`Refusal`); a history crate's refusal arrives as `DIFS0002` with its own code in the detail.

## Releasing

Bump `version` in Cargo.toml, add a dated `## <version> (YYYY-MM-DD)` entry to CHANGELOG.md, and push to
`main`. CI (.github/workflows/ci.yml) runs the tests on stable and Rust 1.89 and clippy, then publishes that
version to crates.io through trusted publishing; a version already on crates.io publishes nothing.

## Ownership

Developed and published by James Hunt. The repository is held by the 1o organization
(https://github.com/1o-llc). Assignment of the copyright to 1o is pending documentation. The license texts
(LICENSE-MIT, LICENSE-APACHE) are left as they were found. Every file new in 0.1.0 carries the notice
Copyright © 2026 1o. All rights reserved.

## License

MIT OR Apache-2.0
