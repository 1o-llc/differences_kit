// Copyright © 2026 1o. All rights reserved.

//! One call per kind of history, each answering a [`Difference`].

use crate::layout::Project;
use crate::refusal::{Refusal, Result};
use receipt_history::{Journal, Receipt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use task_history::{DiffDocument, DiffRecordKind};
use tree_history::count::{compare_counts, counts_in, diff_table, history_table, resolve, route_word, CountDiff, Counted};
use tree_history::tree::{self, TreeDiff, TreeSnapshot, TreeSpec};

/// A difference: the page that puts the two sides next to each other, what
/// was said about it in one line each, whether the sides differ, and the
/// change records (`nodeplan.1o.diff.v1`, unwritten) where it is one of files.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Difference {
    /// The kind's word.
    pub kind: String,
    /// The two sides, as named (`#73`, `before`, `base`), or one when the kind lists.
    pub sides: Vec<String>,
    pub page: String,
    pub said: Vec<String>,
    /// Whether the sides differ.
    pub differs: bool,
    /// What the difference is as change records, where it is one of files.
    #[serde(skip)]
    pub records: Vec<DiffDocument>,
}

// ---- tree ----

/// The tree of `spec` at `point`, kept in the project's `data/trees/`, with the tree line's receipt in the journal.
pub fn tree_take(p: &Project, spec: &TreeSpec, point: &str) -> Result<(TreeSnapshot, Receipt)> {
    let s = tree::take(spec, point)?;
    let dir = p.trees_dir()?;
    let (json, txt) = tree::keep(&dir, &s)?;
    let rel = |f: &std::path::Path| f.strip_prefix(&p.root).unwrap_or(f).to_string_lossy().into_owned();
    let mut r = Receipt {
        schema: receipt_history::RECEIPT_SCHEMA.into(),
        seq: 0,
        utc: s.utc.clone(),
        command: "tree".into(),
        name: point.into(),
        spec: Some(serde_json::to_string(&s.spec).unwrap_or_default()),
        spec_hash: Some(receipt_history::digest::of_value(&s.spec)),
        code_digest: Some(s.digest.clone()),
        index_key: None,
        plan_id: s.receipt.plan_id.clone(),
        plan_digest: s.receipt.plan_digest.clone(),
        operation_digest: s.receipt.operation_digest.clone(),
        receipt_digest: s.receipt.receipt_digest.clone(),
        line: s.line.clone(),
        exit_code: s.receipt.exit_code,
        accepted: s.receipt.accepted,
        disposition: s.receipt.disposition.clone(),
        reason: s.receipt.reason.clone(),
        outputs: vec![rel(&txt), rel(&json)],
        job: None,
        reused_seq: None,
        millis: s.receipt.millis,
        runner: receipt_history::RUNNER_LINE.into(),
    };
    let receipts = p.receipts_dir()?;
    p.journal()?.keep(&mut r, |n| Ok(receipts.join(format!("{n}.json"))))?;
    Ok((s, r))
}

/// The tree at two kept points, side by side.
pub fn tree(p: &Project, a: &str, b: &str) -> Result<Difference> {
    let dir = p.trees_dir()?;
    let (x, y) = (tree::load(&dir, a)?, tree::load(&dir, b)?);
    let d: TreeDiff = tree::compare(&x, &y);
    Ok(Difference {
        kind: "tree".into(),
        sides: vec![a.into(), b.into()],
        page: tree::render(&d, &x, &y),
        said: vec![format!("tree {a} → {b}: {} added, {} removed, {} changed in kind ({} → {} files)", d.added.len(), d.removed.len(), d.kind_changed.len(), d.files.0, d.files.1)],
        differs: !d.is_empty(),
        records: Vec::new(),
    })
}

/// Every point kept, oldest first.
pub fn trees(p: &Project) -> Result<Difference> {
    let all = tree::points(&p.trees_dir()?);
    let mut page = String::from("# tree points\n\n| point | utc | dir | line | directories | files | digest |\n|---|---|---|---|---|---|---|\n");
    for s in &all {
        page.push_str(&format!("| {} | {} | `{}` | `{}` | {} | {} | {} |\n", s.point, s.utc, s.dir, s.line, s.directories, s.files, receipt_history::digest::short(&s.digest, 12)));
    }
    Ok(Difference { kind: "tree".into(), sides: all.iter().map(|s| s.point.clone()).collect(), page, said: vec![format!("{} tree point(s)", all.len())], differs: false, records: Vec::new() })
}

// ---- count ----

fn all_counts(p: &Project, j: &Journal) -> Result<Vec<Counted>> {
    Ok(counts_in(&p.root, &j.runs()?))
}

/// The change records a count diff is: each file added, removed or changed,
/// with its digest at each side (`none` where it is not).
pub fn count_records(d: &CountDiff, a: &tree_history::CountReport, b: &tree_history::CountReport) -> Vec<DiffDocument> {
    let digests = |c: &tree_history::CountReport| -> BTreeMap<String, String> { c.crates.iter().flat_map(|k| k.file_counts.iter()).map(|f| (f.path.clone(), f.digest.clone())).collect() };
    let (da, db) = (digests(a), digests(b));
    let none = || "none".to_string();
    let mut out = Vec::new();
    for (files, op) in [(&d.files_added, "add"), (&d.files_removed, "remove"), (&d.files_changed, "modify")] {
        for f in files {
            out.push(DiffDocument::record(DiffRecordKind::Change).with("path", f.as_str()).with("operation", op).with("before_digest", da.get(f).cloned().unwrap_or_else(none)).with("after_digest", db.get(f).cloned().unwrap_or_else(none)));
        }
    }
    out
}

/// Two counts, each named by its receipt number, job (`<job>:open`, `<job>:close`), commit or date, side by side.
pub fn count(p: &Project, a: &str, b: &str) -> Result<Difference> {
    let j = p.journal()?;
    let all = all_counts(p, &j)?;
    let reused_of = |n: i64| j.receipt(n).ok().flatten().and_then(|r| r.reused_seq);
    let job_id = |name: &str| j.job(name).ok().flatten().map(|r| r.id);
    let (x, y) = (resolve(&all, a, reused_of, job_id)?, resolve(&all, b, reused_of, job_id)?);
    let d = compare_counts(&x.count, &y.count);
    Ok(Difference {
        kind: "count".into(),
        sides: vec![format!("#{}", x.seq), format!("#{}", y.seq)],
        page: diff_table(&d, x.seq, y.seq),
        said: vec![format!("compared count #{} ({}) with #{} ({}): {}", x.seq, x.utc, y.seq, y.utc, d.line("moved"))],
        differs: !d.is_empty(),
        records: count_records(&d, &x.count, &y.count),
    })
}

/// Every count of a route (all routes when none), newest first.
pub fn count_history(p: &Project, route: Option<&str>) -> Result<Difference> {
    let j = p.journal()?;
    let all = all_counts(p, &j)?;
    let want = route.map(route_word);
    let (page, n) = history_table(&all, want.as_deref());
    Ok(Difference { kind: "count".into(), sides: Vec::new(), page, said: vec![format!("{n} count(s) in the journal{}", want.map(|w| format!(" of {w}")).unwrap_or_default())], differs: false, records: Vec::new() })
}

// ---- receipts ----

fn outcome(r: &Receipt) -> String {
    if r.reused_seq.is_some() {
        format!("reused #{}", r.reused_seq.unwrap_or_default())
    } else if r.accepted {
        r.reason.clone()
    } else {
        format!("refused: {}", r.reason)
    }
}

/// The receipts after `since`, with what moved since that point: each spec
/// read again whose code digest is not the one it was read over at or before `since`.
pub fn receipts_since(p: &Project, since: i64) -> Result<Difference> {
    let j = p.journal()?;
    let all = j.runs()?;
    let after: Vec<&Receipt> = all.iter().filter(|r| r.seq > since).collect();
    let mut page = format!("# receipts since #{since}\n\n| | at #{since} | now |\n|---|---|---|\n| receipts | {} | {} |\n", all.iter().filter(|r| r.seq <= since).count(), all.len());
    let mut by: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for r in &all {
        let e = by.entry(r.command.as_str()).or_default();
        if r.seq <= since {
            e.0 += 1;
        }
        e.1 += 1;
    }
    for (c, (b, a)) in &by {
        page.push_str(&format!("| {c} | {b} | {a} |\n"));
    }
    page.push_str(&format!("\n## the {} receipt(s) after #{since}\n\n| seq | utc | command | name | runner | outcome | job | outputs |\n|---|---|---|---|---|---|---|---|\n", after.len()));
    for r in &after {
        page.push_str(&format!("| #{} | {} | {} | {} | {} | {} | {} | {} |\n", r.seq, r.utc, r.command, r.name.replace('|', "\\|"), r.runner, outcome(r).replace('|', "\\|"), r.job.as_deref().unwrap_or("-"), r.outputs.len()));
    }
    // what moved: a spec read since the point over code it was not read over at the point
    let mut moved = Vec::new();
    for r in after.iter().filter(|r| r.command == "reading" && r.reused_seq.is_none() && r.accepted) {
        let before = all.iter().rfind(|x| x.seq <= since && x.command == "reading" && x.name == r.name && x.reused_seq.is_none());
        if let Some(b) = before {
            if b.code_digest != r.code_digest {
                moved.push(format!("- `{}`: #{} → #{} (the code moved: {} → {})", r.name, b.seq, r.seq, receipt_history::digest::short(b.code_digest.as_deref().unwrap_or("-"), 12), receipt_history::digest::short(r.code_digest.as_deref().unwrap_or("-"), 12)));
            }
        }
    }
    if !moved.is_empty() {
        page.push_str(&format!("\n## read again over moved code ({})\n\n{}\n", moved.len(), moved.join("\n")));
    }
    Ok(Difference { kind: "receipts".into(), sides: vec![format!("#{since}"), "now".into()], page, said: vec![format!("{} receipt(s) after #{since}, {} of them readings", after.len(), after.iter().filter(|r| r.command == "reading").count())], differs: !after.is_empty(), records: Vec::new() })
}

/// Two receipts side by side, field by field; a field that differs is marked.
pub fn receipts_pair(p: &Project, a: i64, b: i64) -> Result<Difference> {
    let j = p.journal()?;
    let get = |n: i64| j.receipt(n).ok().flatten().ok_or_else(|| Refusal::NoReceipt.refuse(format!("#{n}")));
    let (x, y) = (get(a)?, get(b)?);
    let (vx, vy) = (serde_json::to_value(&x).unwrap_or_default(), serde_json::to_value(&y).unwrap_or_default());
    let mut page = format!("# receipts #{a} and #{b}\n\n| field | #{a} | #{b} | |\n|---|---|---|---|\n");
    let mut differ = 0;
    if let (Some(ox), Some(oy)) = (vx.as_object(), vy.as_object()) {
        for (k, v) in ox {
            if k == "schema" {
                continue;
            }
            let w = oy.get(k).cloned().unwrap_or_default();
            let mark = if *v != w && k != "seq" { "≠" } else { "" };
            if !mark.is_empty() {
                differ += 1;
            }
            let show = |v: &serde_json::Value| match v {
                serde_json::Value::String(s) => s.replace('|', "\\|"),
                serde_json::Value::Null => "-".into(),
                other => other.to_string().replace('|', "\\|"),
            };
            page.push_str(&format!("| {k} | {} | {} | {mark} |\n", show(v), show(&w)));
        }
    }
    Ok(Difference { kind: "receipts".into(), sides: vec![format!("#{a}"), format!("#{b}")], page, said: vec![format!("receipts #{a} and #{b}: {differ} field(s) differ")], differs: differ > 0, records: Vec::new() })
}

// ---- revisions and tasks ----

/// A task's revisions, one to the next (`history` names another history root than the project's).
pub fn revisions(p: Option<&Project>, history: Option<&std::path::Path>, task: &str) -> Result<Difference> {
    let root = match (history, p) {
        (Some(h), _) => h.to_path_buf(),
        (None, Some(p)) => p.history_root()?,
        (None, None) => return Err(Refusal::NoProject.refuse("revisions need a project or a history root")),
    };
    let t = revision_history::read(&root, task)?.ok_or_else(|| revision_history::Refusal::NoTask.refuse(format!("{task} in {}", root.display())))?;
    let c = revision_history::chain(&t);
    let moved = c.steps.iter().any(|s| !s.changes_new.is_empty() || !s.changes_again.is_empty() || !s.changes_dropped.is_empty() || !s.verifications_moved.is_empty() || s.status_before != s.status_after);
    Ok(Difference {
        kind: "revisions".into(),
        sides: c.revisions.iter().map(|r| r.name.clone()).collect(),
        page: revision_history::render(&c),
        said: vec![format!("{}: {} revision(s), {} step(s) compared", c.task, c.revisions.len(), c.steps.len())],
        differs: moved,
        records: t.revisions.iter().flat_map(|r| r.documents.iter().filter(|d| d.kind().ok() == Some(DiffRecordKind::Change)).cloned()).collect(),
    })
}

/// Every task history: its revisions, its last result.
pub fn tasks(p: Option<&Project>, history: Option<&std::path::Path>) -> Result<Difference> {
    let root = match (history, p) {
        (Some(h), _) => h.to_path_buf(),
        (None, Some(p)) => p.history_root()?,
        (None, None) => return Err(Refusal::NoProject.refuse("tasks need a project or a history root")),
    };
    let index = task_history::load_task_history(&root).map_err(|e| revision_history::Refusal::Unreadable.refuse(format!("{}: {e}", root.display())))?;
    let mut page = format!("# task histories\n\n- root: `{}`\n- tasks: {}, revisions: {}, records: {}, changes: {}, accepted revisions: {}\n- index digest: {}\n\n| task | revisions | last result | records |\n|---|---|---|---|\n", root.display(), index.task_count, index.revision_count, index.document_count, index.change_count, index.accepted_revision_count, receipt_history::digest::short(&index.index_digest, 12));
    for t in &index.tasks {
        let c = revision_history::chain(t);
        let last = c.revisions.last().map(|r| r.result.as_ref().map(|x| x.0.clone()).unwrap_or_else(|| r.task_status.clone())).unwrap_or_default();
        page.push_str(&format!("| {} | {} | {} | {} |\n", t.task_directory, c.revisions.iter().map(|r| r.name.as_str()).collect::<Vec<_>>().join(", "), last, c.revisions.iter().map(|r| r.records).sum::<usize>()));
    }
    Ok(Difference { kind: "tasks".into(), sides: Vec::new(), page, said: vec![format!("{} task(s), {} revision(s)", index.task_count, index.revision_count)], differs: false, records: Vec::new() })
}

// ---- changes ----

/// The lines each job touched, side by side (ids or names, as the journal knows them).
pub fn changes(p: &Project, jobs: &[&str]) -> Result<Difference> {
    if jobs.is_empty() {
        return Err(Refusal::NothingToCompare.refuse("changes names a job or more"));
    }
    let j = p.journal()?;
    let mut lines = Vec::new();
    let mut folders = Vec::new();
    for name in jobs {
        let row = j.job(name)?.ok_or_else(|| Refusal::NoJob.refuse(*name))?;
        let folder = p.at(&row.folder);
        lines.push(change_history::job_lines(&folder)?);
        folders.push(folder);
    }
    let refs: Vec<&std::path::Path> = folders.iter().map(|f| f.as_path()).collect();
    let mut page = change_history::render_jobs(&lines);
    let mut said = vec![format!("{} job(s)", lines.len())];
    if let Ok(w) = change_history::lines_within(&refs) {
        page.push_str(&format!("\n## together\n\n- +{} −{} across {} files in {} crates ({} count diff(s))\n", w.moved.added.lines, w.moved.removed.lines, w.files.len(), w.crates.len(), w.diffs));
        said.push(format!("together: +{} −{} across {} files", w.moved.added.lines, w.moved.removed.lines, w.files.len()));
    }
    Ok(Difference { kind: "changes".into(), sides: lines.iter().map(|l| l.id.clone()).collect(), page, said, differs: lines.iter().any(|l| !l.lines_touched.is_empty()), records: Vec::new() })
}

// ---- readings ----

/// Where a reading's files are, relative to the root: its record (the facts)
/// and its page (custodian_kit's).
pub use custodian_kit::receipts::ReadingFiles;

/// The reading a receipt made: custodian_kit's one rule
/// (`receipts::reading_of`), the rule its readings index and a job's
/// `reading:` verification read too. `folders` is [`Project::job_folders`],
/// read once by a caller that asks for many.
pub fn reading_of(p: &Project, r: &Receipt, folders: &[(String, std::path::PathBuf)]) -> Option<ReadingFiles> {
    custodian_kit::receipts::reading_of(&p.root, r, folders)
}

/// A spec's readings as the receipts hold them, oldest first: each receipt
/// of that name that made a reading ([`reading_of`]), with its record.
pub fn readings_of(p: &Project, spec: &str) -> Result<Vec<(Receipt, String)>> {
    let runs = p.journal()?.runs()?;
    Ok(custodian_kit::receipts::readings_of(&p.root, &runs, spec, &p.job_folders()?).into_iter().map(|(r, f)| (r, f.record)).collect())
}

/// The two readings of one spec a comparison reads (receipt numbers; the
/// last two by default; a reuse's number names the reading it reused), each
/// with its record: what `readings` compares and what custodian's `diff` reads.
pub fn reading_pair(p: &Project, spec: &str, a: Option<i64>, b: Option<i64>) -> Result<[(Receipt, String); 2]> {
    let j = p.journal()?;
    let all = readings_of(p, spec)?;
    // a reuse names the reading it reused
    let own = |n: i64| j.receipt(n).ok().flatten().and_then(|r| r.reused_seq).unwrap_or(n);
    let pick = |n: Option<i64>, default: Option<&(Receipt, String)>| -> Result<(Receipt, String)> {
        match n {
            Some(n) => all.iter().find(|(r, _)| r.seq == own(n)).cloned().ok_or_else(|| Refusal::NoReading.refuse(format!("{spec} #{n} ({} reading(s): {})", all.len(), all.iter().map(|(r, _)| format!("#{}", r.seq)).collect::<Vec<_>>().join(", ")))),
            None => default.cloned().ok_or_else(|| Refusal::NothingToCompare.refuse(format!("{spec}: {} reading(s)", all.len()))),
        }
    };
    let after = pick(b, all.last())?;
    let before = pick(a, all.iter().rev().find(|(r, _)| r.seq < after.0.seq))?;
    Ok([before, after])
}

/// Two readings of one spec (receipt numbers; the last two by default), side
/// by side: two counts by what moved file by file, any other reading by its
/// rows and counts.
pub fn readings(p: &Project, spec: &str, a: Option<i64>, b: Option<i64>) -> Result<Difference> {
    let [before, after] = reading_pair(p, spec, a, b)?;
    let (pa, pb) = (p.at(&before.1), p.at(&after.1));
    if let (Some((x, _, _)), Some((y, _, _))) = (tree_history::count::load_count(&pa), tree_history::count::load_count(&pb)) {
        let d = compare_counts(&x, &y);
        return Ok(Difference {
            kind: "readings".into(),
            sides: vec![format!("#{}", before.0.seq), format!("#{}", after.0.seq)],
            page: diff_table(&d, before.0.seq, after.0.seq),
            said: vec![format!("compared {spec}: #{} → #{}: {}", before.0.seq, after.0.seq, d.line("grew"))],
            differs: !d.is_empty(),
            records: count_records(&d, &x, &y),
        });
    }
    let (x, y) = (diff_history::load(&pa)?, diff_history::load(&pb)?);
    let d = diff_history::compare(&x, &y);
    let mut page = format!("| | #{} | #{} |\n|---|---|---|\n| stamp | {} | {} |\n| record | `{}` | `{}` |\n| rows | {} | {} |\n\n", before.0.seq, after.0.seq, x.stamp, y.stamp, before.1, after.1, x.rows.len(), y.rows.len());
    page.push_str(&diff_history::render(&d));
    Ok(Difference {
        kind: "readings".into(),
        sides: vec![format!("#{}", before.0.seq), format!("#{}", after.0.seq)],
        page,
        said: vec![format!("compared {spec}: #{} ({}) → #{} ({}): {} added, {} removed, {} count(s) moved", before.0.seq, x.stamp, after.0.seq, y.stamp, d.added.len(), d.removed.len(), d.counts.iter().filter(|c| c.before != c.after).count())],
        differs: !d.is_empty(),
        records: Vec::new(),
    })
}
