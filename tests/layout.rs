// Copyright © 2026 1o. All rights reserved.

//! The folder's paths are custodian_kit's: the descriptors this crate names are the kit's, and a project resolves
//! each part where the kit's store does.

use custodian_kit::store::named as kit;
use differences_kit::layout::{named, FOLDER};
use differences_kit::Project;

#[test]
fn every_path_read_here_is_the_kits() {
    assert_eq!(FOLDER, custodian_kit::store::FOLDER);
    for (mine, theirs) in [(named::SETTINGS, kit::SETTINGS), (named::RECEIPT, kit::RECEIPT), (named::JOURNAL, kit::JOURNAL), (named::HISTORY_INDEX, kit::HISTORY_INDEX), (named::TREE, kit::TREE), (named::JOB_FILE, kit::JOB_FILE), (named::READING_RECORD, kit::READING_RECORD)] {
        assert_eq!(mine, theirs, "{}", theirs.route_id);
    }
    let d = std::env::temp_dir().join(format!("differences-kit-layout-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    let (store, _) = custodian_kit::store::Store::init(&d, custodian_kit::store::Mode::Agent).unwrap();
    let p = Project::find(&d).unwrap();
    assert_eq!(p.root, store.root);
    let vars: Vec<(&str, &str)> = ["name", "stamp", "seq", "state", "job", "kind", "modifier", "point"].iter().map(|k| (*k, "x")).collect();
    for part in kit::ALL {
        assert_eq!(p.path(part, &vars).unwrap(), store.path(part, &vars).unwrap(), "{}", part.route_id);
    }
    assert_eq!(p.receipts_dir().unwrap(), store.dir_of(&kit::RECEIPT, &[("seq", "1")]).unwrap());
    assert_eq!(p.jobs_root().unwrap(), store.jobs_dir(custodian_kit::store::JobState::Done).unwrap().parent().unwrap());
    assert_eq!(p.job_folders().unwrap(), Vec::<(String, std::path::PathBuf)>::new());
}
