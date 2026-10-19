# differences_kit

Reserved; the library lands here. The library for building a kind of history (records, digests, revisions, verification) that the differences CLI composes — the toolkit under task_history, diff_history, change_history, revision_history, tree_history and receipt_history.

This repository holds the 0.0.1 placeholder source as published on crates.io
([`differences_kit`](https://crates.io/crates/differences_kit)): it holds the name, and nothing in it is usable yet.

## Ownership

Developed and published by James Hunt. The repository is held by the 1o organization
(https://github.com/1o-llc). Assignment of the copyright to 1o is pending documentation. The license texts
(LICENSE-MIT, LICENSE-APACHE) are the ones the other 1o crates carry.

## Releasing

Bump `version` in Cargo.toml, add a dated `## <version> (YYYY-MM-DD)` entry to CHANGELOG.md, and push to
`main`. CI (.github/workflows/ci.yml) runs the tests and clippy, then publishes that version to crates.io.

## License

MIT OR Apache-2.0.
