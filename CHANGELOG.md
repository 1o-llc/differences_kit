# Changelog

## 0.1.1 (2026-10-19)

The folder this crate reads is custodian_kit's, and a reading is the receipt
that made it, by custodian_kit's one rule. Before, this crate declared its
own copy of five of the folder's parts, and picked a spec's readings by a
rule of its own that custodian did not share. Nothing is removed from the
API. Every link is pinned to 269201c; the earlier lines are pinned to 54c8f4b
(0.1.0).

- `layout` declares no part of its own. `FOLDER` and the descriptors in
  `layout::named` re-export custodian_kit's `store::named`, with the same
  values ([src/layout.rs#L15](https://github.com/1o-llc/differences_kit/blob/269201c59e097d8ae6f015eca4ef0450a8a5869e/src/layout.rs#L15), [src/layout.rs#L18](https://github.com/1o-llc/differences_kit/blob/269201c59e097d8ae6f015eca4ef0450a8a5869e/src/layout.rs#L18)). The
  policy and every path go through custodian_kit's `named::policy` and
  `named::resolve` ([src/layout.rs#L29](https://github.com/1o-llc/differences_kit/blob/269201c59e097d8ae6f015eca4ef0450a8a5869e/src/layout.rs#L29), [src/layout.rs#L52](https://github.com/1o-llc/differences_kit/blob/269201c59e097d8ae6f015eca4ef0450a8a5869e/src/layout.rs#L52)).
  Before, they were declared here ([src/layout.rs#L32](https://github.com/1o-llc/differences_kit/blob/54c8f4b27a7b1127f84502d44b90fda887beff9f/src/layout.rs#L32)). New:
  `Project::open` for a caller that has found the folder already
  ([src/layout.rs#L47](https://github.com/1o-llc/differences_kit/blob/269201c59e097d8ae6f015eca4ef0450a8a5869e/src/layout.rs#L47)), `Project::jobs_root` and
  `Project::job_folders` ([src/layout.rs#L76](https://github.com/1o-llc/differences_kit/blob/269201c59e097d8ae6f015eca4ef0450a8a5869e/src/layout.rs#L76),
  [src/layout.rs#L82](https://github.com/1o-llc/differences_kit/blob/269201c59e097d8ae6f015eca4ef0450a8a5869e/src/layout.rs#L82)).
- A spec's readings are the receipts that made them, by custodian_kit's
  `receipts::reading_of` ([src/receipts.rs#L69](https://github.com/1o-llc/custodian_kit/blob/0d4ac01bb6274a7dbe669c0e05c96449ada1846d/src/receipts.rs#L69)). `reading_of`
  ([src/difference.rs#L297](https://github.com/1o-llc/differences_kit/blob/269201c59e097d8ae6f015eca4ef0450a8a5869e/src/difference.rs#L297)) and `readings_of`
  ([src/difference.rs#L303](https://github.com/1o-llc/differences_kit/blob/269201c59e097d8ae6f015eca4ef0450a8a5869e/src/difference.rs#L303)) read it, and `ReadingFiles` is its type.
  Two behaviours change. A receipt of the same name that read no spec, a
  preflight's for one, is no longer counted as a reading. A reading kept in a
  job's folder is found wherever that folder is now. Before, any accepted
  receipt of the name whose `.json` output was still in place counted
  ([src/difference.rs#L291](https://github.com/1o-llc/differences_kit/blob/54c8f4b27a7b1127f84502d44b90fda887beff9f/src/difference.rs#L291)).
- `reading_pair` ([src/difference.rs#L311](https://github.com/1o-llc/differences_kit/blob/269201c59e097d8ae6f015eca4ef0450a8a5869e/src/difference.rs#L311)) is the pair of readings
  `readings` compares ([src/difference.rs#L330](https://github.com/1o-llc/differences_kit/blob/269201c59e097d8ae6f015eca4ef0450a8a5869e/src/difference.rs#L330)), and custodian's
  `diff` reads the same pair.
- New dependency: custodian_kit 0.3.1.

Proof: the rustdoc JSON of 0.1.0 compared with this release: 254 public items
before, 268 after. Six paths are now re-exports of custodian_kit's items with
the same values (`layout::FOLDER` and five `layout::named` descriptors), and
none is removed. `cargo test` runs 4 tests, the 2 of 0.1.0 and two new:
`every_path_read_here_is_the_kits` resolves every one of custodian_kit's parts
here and in its store and finds them equal ([tests/layout.rs#L11](https://github.com/1o-llc/differences_kit/blob/269201c59e097d8ae6f015eca4ef0450a8a5869e/tests/layout.rs#L11)), and
`a_reading_is_a_reading_receipt_and_its_record_follows_its_job_folder`
([tests/kinds.rs#L137](https://github.com/1o-llc/differences_kit/blob/269201c59e097d8ae6f015eca4ef0450a8a5869e/tests/kinds.rs#L137)).

## 0.1.0 (2026-10-19)

The library that puts every kind of history side by side. Each kind of
history is a crate of its own: receipt_history, revision_history,
tree_history, change_history, diff_history and task_history. This crate
composes them into one call per kind, over a project's `.custodian/`
folder. It is the one API the `differences` command line and custodian
call. Every link is pinned to 3a67471.

- Every call answers a `Difference` ([src/difference.rs#L18](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/difference.rs#L18)). It holds:
  - the page that puts the two sides next to each other;
  - one line said about it;
  - whether the sides differ;
  - for a difference of files, the change records
    (`nodeplan.1o.diff.v1`) that say it.
- `Project::find` ([src/layout.rs#L55](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/layout.rs#L55)) walks up to a `.custodian/`
  folder. The files read are named through pathgen with custodian_kit's route
  ids and templates; the tree snapshots go in `data/trees/{point}.json`
  ([src/layout.rs#L39](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/layout.rs#L39)). `Project::journal` ([src/layout.rs#L96](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/layout.rs#L96)) opens
  receipt_history's journal. When the journal is missing rows, it is rebuilt
  from the receipt files the folder carries.
- `tree_take` ([src/difference.rs#L35](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/difference.rs#L35)) takes the tree at a point
  through tree_history's declared line. It keeps the snapshot, and keeps the
  tree line's receipt in the journal (command `tree`, its plan, line and
  decision digests). `tree` ([src/difference.rs#L71](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/difference.rs#L71)) compares two kept
  points; `trees` ([src/difference.rs#L86](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/difference.rs#L86)) lists them.
- `count` ([src/difference.rs#L117](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/difference.rs#L117)) compares two counts. Each is named by
  its receipt number, its job (`<job>:open` or `<job>:close`), its commit
  or its date. The change records come from `count_records`
  ([src/difference.rs#L103](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/difference.rs#L103)): one per file, with its digest on each side.
  `count_history` ([src/difference.rs#L135](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/difference.rs#L135)) lists a route's counts across
  commits.
- `receipts_since` ([src/difference.rs#L157](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/difference.rs#L157)) lists:
  - the receipts after a point;
  - their commands, then and now;
  - every spec read again since then over code it was not read over before.

  `receipts_pair` ([src/difference.rs#L194](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/difference.rs#L194)) puts two receipts side by
  side field by field, marking the fields that differ.
- `revisions` ([src/difference.rs#L225](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/difference.rs#L225)) shows a task's revision chain
  (revision_history). `tasks` ([src/difference.rs#L245](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/difference.rs#L245)) lists every task
  history, its revisions and its last result. Both read the project's history
  root, or another root when one is given.
- `changes` ([src/difference.rs#L264](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/difference.rs#L264)) puts the lines each job touched side
  by side, then together (change_history).
- `readings` ([src/difference.rs#L306](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/difference.rs#L306)) compares two readings of one spec.
  It finds them by their receipt numbers; by default it takes the last two,
  and a reuse names the reading it reused. Two counts are compared file by
  file (tree_history). Any other pair is compared row by row (diff_history).
- `Kind` ([src/kinds.rs#L10](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/kinds.rs#L10)) is the kinds as one big-bad-bool word:
  `tree`, `count`, `receipts`, `revisions`, `changes`, `readings`,
  `tasks`.
- Proven by `every_kind_answers_side_by_side_from_one_project`
  ([tests/kinds.rs#L64](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/tests/kinds.rs#L64)), which builds one project and calls every kind on
  it, and `the_kinds_are_words` ([src/kinds.rs#L43](https://github.com/1o-llc/differences_kit/blob/3a67471bdbe84b3df9383e4c75790ead9626fc8e/src/kinds.rs#L43)).

Refusals are liaise codes `DIFS0001` to `DIFS0007`. A history crate's
refusal arrives as `DIFS0002`, with its own code in the detail.

Proof: `cargo test`, 2 tests.
