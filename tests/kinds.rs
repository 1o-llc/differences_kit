// Copyright © 2026 1o. All rights reserved.

//! Every kind over one small project: its journal, receipts, readings, task history and tree points.

use differences_kit::{changes, count, count_history, readings, receipts_pair, receipts_since, revisions, tasks, tree, tree_take, trees, Project};
use receipt_history::{JobRow, Journal, Receipt, RECEIPT_SCHEMA};
use std::path::{Path, PathBuf};
use tree_history::count::{CountReport, FileCount, Lines};
use tree_history::tree::TreeSpec;

fn project() -> PathBuf {
    let d = std::env::temp_dir().join(format!("differences-kit-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(d.join(".custodian/config")).unwrap();
    std::fs::write(d.join(".custodian/config/custodian.toml"), "mode = \"agent\"\n").unwrap();
    d
}

fn receipt(name: &str, command: &str, outputs: Vec<String>, code: &str, job: Option<&str>) -> Receipt {
    Receipt {
        schema: RECEIPT_SCHEMA.into(),
        seq: 0,
        utc: "2026-10-19T12:00:00Z".into(),
        command: command.into(),
        name: name.into(),
        spec: None,
        spec_hash: Some(format!("spec-{name}")),
        code_digest: Some(code.into()),
        index_key: None,
        plan_id: "custodian".into(),
        plan_digest: "p".into(),
        operation_digest: "o".into(),
        receipt_digest: "r".into(),
        line: format!("custodian run {name}"),
        exit_code: Some(0),
        accepted: true,
        disposition: "Continue".into(),
        reason: "read".into(),
        outputs,
        job: job.map(str::to_string),
        reused_seq: None,
        millis: 1,
        runner: "line".into(),
    }
}

fn count_of(files: &[(&str, usize, &str)], commit: &str) -> CountReport {
    CountReport::from_files("*".into(), None, Some(commit.into()), files.iter().map(|(p, n, d)| ("a".to_string(), FileCount { path: p.to_string(), language: "Rust".into(), lines: Lines { lines: *n, code: *n, comments: 0, blanks: 0 }, tests: Lines::default(), digest: d.to_string() })).collect())
}

fn write(root: &Path, rel: &str, v: serde_json::Value) -> String {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, serde_json::to_string_pretty(&v).unwrap()).unwrap();
    rel.to_string()
}

fn keep(root: &Path, j: &Journal, r: Receipt) -> i64 {
    let dir = root.join(".custodian/data/receipts");
    let mut r = r;
    j.keep(&mut r, |n| Ok(dir.join(format!("{n}.json")))).unwrap().0
}

#[test]
fn every_kind_answers_side_by_side_from_one_project() {
    let root = project();
    let j = Journal::open(&root.join(".custodian/state/journal.sqlite")).unwrap();
    let d = |c: char| -> String { std::iter::repeat_n(c, 64).collect() };
    // two counts, at two commits
    let c1 = write(&root, ".custodian/data/readings/count-workspace/1.json", serde_json::json!({"name": "count-workspace", "stamp": "20261019T110000Z", "target": "*", "count": count_of(&[("a/src/lib.rs", 10, &d('1'))], "aaaaaaa1")}));
    let c2 = write(&root, ".custodian/data/readings/count-workspace/2.json", serde_json::json!({"name": "count-workspace", "stamp": "20261019T120000Z", "target": "*", "count": count_of(&[("a/src/lib.rs", 14, &d('2')), ("a/src/new.rs", 3, &d('3'))], "bbbbbbb2")}));
    let n1 = keep(&root, &j, receipt("count-workspace", "reading", vec![c1], "c1", Some("j-1")));
    let n2 = keep(&root, &j, receipt("count-workspace", "reading", vec![c2], "c2", Some("j-1")));
    // two readings of one spec over moved code
    let row = |label: &str, path: &str| serde_json::json!({"id": label, "kind": "use-site", "label": label, "path": path, "line": 3, "attributes": {"context": label}});
    let r1 = write(&root, ".custodian/data/readings/env/1.json", serde_json::json!({"name": "env", "stamp": "20261019T110000Z", "counts": [{"what": "use sites", "n": 1}], "query": {"results": [row("a", "a.rs")]}, "unresolved": []}));
    let r2 = write(&root, ".custodian/data/readings/env/2.json", serde_json::json!({"name": "env", "stamp": "20261019T120000Z", "counts": [{"what": "use sites", "n": 2}], "query": {"results": [row("a", "a.rs"), row("b", "b.rs")]}, "unresolved": []}));
    let e1 = keep(&root, &j, receipt("env", "reading", vec![r1], "c1", None));
    let e2 = keep(&root, &j, receipt("env", "reading", vec![r2], "c2", None));
    j.put_job(&JobRow { id: "j-1".into(), name: "batch".into(), kind: "specs".into(), state: "done".into(), folder: ".custodian/state/jobs/done/batch".into(), opened_utc: None, closed_utc: None }).unwrap();
    let p = Project::find(&root.join(".custodian/config")).unwrap();

    let c = count(&p, &n1.to_string(), "bbbbbbb").unwrap();
    assert!(c.page.contains(&format!("# count report: #{n1} → #{n2} (*)")), "{}", c.page);
    assert_eq!(c.records.iter().map(|r| format!("{} {}", r.one("operation").unwrap(), r.one("path").unwrap())).collect::<Vec<_>>(), ["add a/src/new.rs", "modify a/src/lib.rs"]);
    assert!(c.differs);
    assert_eq!(count(&p, "batch:open", "batch:close").unwrap().sides, [format!("#{n1}"), format!("#{n2}")]);
    assert!(count_history(&p, None).unwrap().page.contains("| aaaaaaa1 |"));

    let r = readings(&p, "env", None, None).unwrap();
    assert_eq!(r.sides, [format!("#{e1}"), format!("#{e2}")]);
    assert!(r.page.contains("- use sites: 1 → 2 (changed)") && r.page.contains("## added (1)"), "{}", r.page);
    let rc = readings(&p, "count-workspace", None, None).unwrap();
    assert!(rc.page.contains("# count report:"), "two counts read as counts: {}", rc.page);

    let s = receipts_since(&p, n2).unwrap();
    assert!(s.page.contains(&format!("| #{e1} |")) && !s.page.contains(&format!("| #{n2} | 2026")), "{}", s.page);
    let pair = receipts_pair(&p, e1, e2).unwrap();
    assert!(pair.page.contains("| code_digest | c1 | c2 | ≠ |"), "{}", pair.page);

    // a task with two revisions
    let h = p.history_root().unwrap();
    let base = revision_history::open(&h, "batch", "specs", "j-1", "read").unwrap();
    revision_history::close(&base, "pending", None).unwrap();
    let fix1 = revision_history::open(&h, "batch", "specs", "j-2", "again").unwrap();
    revision_history::verification(&fix1, "cargo test -p a", Some(true), None).unwrap();
    revision_history::close(&fix1, "accepted", None).unwrap();
    let rv = revisions(Some(&p), None, "batch").unwrap();
    assert_eq!(rv.sides, ["base", "fix1"]);
    assert!(rv.page.contains("| result | pending | accepted |"), "{}", rv.page);
    assert!(tasks(Some(&p), None).unwrap().page.contains("| batch | base, fix1 | accepted |"));

    // a job's lines touched
    let folder = root.join(".custodian/state/jobs/done/batch");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("closing.json"), serde_json::to_string(&serde_json::json!({"id": "j-1", "name": "batch", "closed_utc": "2026-10-19T12:00:00Z", "result": "accepted", "lines_touched": []})).unwrap()).unwrap();
    let ch = changes(&p, &["batch"]).unwrap();
    assert!(ch.page.contains("| result | accepted |"), "{}", ch.page);

    // two tree points
    if std::process::Command::new("tree").arg("--version").output().is_ok_and(|o| o.status.success()) {
        let work = root.join("work");
        std::fs::create_dir_all(work.join("src")).unwrap();
        std::fs::write(work.join("src/lib.rs"), "").unwrap();
        let (_, receipt) = tree_take(&p, &TreeSpec::new(&work), "one").unwrap();
        assert_eq!((receipt.command.as_str(), receipt.line.as_str()), ("tree", "tree -n --charset ascii ."));
        std::fs::write(work.join("src/two.rs"), "").unwrap();
        tree_take(&p, &TreeSpec::new(&work), "two").unwrap();
        let t = tree(&p, "one", "two").unwrap();
        assert!(t.page.contains("+ src/two.rs"), "{}", t.page);
        assert_eq!(trees(&p).unwrap().sides, ["one", "two"]);
        // the tree lines left receipts in the journal
        assert_eq!(receipts_since(&p, e2).unwrap().said[0], "2 receipt(s) after #4, 0 of them readings");
    }
}

#[test]
fn a_reading_is_a_reading_receipt_and_its_record_follows_its_job_folder() {
    let root = project();
    let j = Journal::open(&root.join(".custodian/state/journal.sqlite")).unwrap();
    let rec = |stamp: &str, n: usize| serde_json::json!({"name": "env", "stamp": stamp, "counts": [{"what": "use sites", "n": n}], "query": {"results": []}, "unresolved": []});
    // a reading kept in its job's folder while the job ran; the folder has since moved to done/
    let done = root.join(".custodian/state/jobs/done/batch");
    std::fs::create_dir_all(&done).unwrap();
    std::fs::write(done.join(custodian_kit::jobs::job_file_name()), "1o.job {\n  id \"j-9\"\n  name \"batch\"\n}\n").unwrap();
    write(&root, ".custodian/state/jobs/done/batch/readings/env/1.json", rec("20261019T110000Z", 1));
    let ran = keep(&root, &j, receipt("env", "reading", vec![".custodian/state/jobs/running/batch/readings/env/1.json".into()], "c1", Some("j-9")));
    // a preflight of the same name is not a reading
    let pre = write(&root, ".custodian/data/preflight/env/x.json", serde_json::json!({}));
    keep(&root, &j, receipt("env", "preflight", vec![pre], "c1", None));
    let r2 = write(&root, ".custodian/data/readings/env/2.json", rec("20261019T120000Z", 2));
    let second = keep(&root, &j, receipt("env", "reading", vec![r2.clone()], "c2", None));
    let p = Project::find(&root).unwrap();
    let all = differences_kit::readings_of(&p, "env").unwrap();
    assert_eq!(all.iter().map(|(r, f)| (r.seq, f.as_str())).collect::<Vec<_>>(), [(ran, ".custodian/state/jobs/done/batch/readings/env/1.json"), (second, r2.as_str())]);
    let [a, b] = differences_kit::reading_pair(&p, "env", None, None).unwrap();
    assert_eq!((a.0.seq, b.0.seq), (ran, second));
    assert!(readings(&p, "env", None, None).unwrap().page.contains("- use sites: 1 → 2 (changed)"));
}
