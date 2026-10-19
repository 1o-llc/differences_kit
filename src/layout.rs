// Copyright © 2026 1o. All rights reserved.

//! The project: the folder holding `.custodian/`, and the files of it this
//! crate reads. The folder is custodian_kit's, named once in its
//! `store::named` (a dev policy whose base is the project root and whose app
//! is `.custodian`): this crate takes those descriptors and that policy, and
//! declares none of its own.

use crate::refusal::{Refusal, Result};
use pathgen::{ArtifactOutputDescriptor, PathPolicy};
use receipt_history::Journal;
use std::path::{Path, PathBuf};

/// The initialized folder's name (custodian_kit's).
pub use custodian_kit::store::FOLDER;

/// The files read here, custodian_kit's `store::named` descriptors.
pub mod named {
    pub use custodian_kit::store::named::{HISTORY_INDEX, JOB_FILE, JOURNAL, READING_RECORD, RECEIPT, SETTINGS, TREE};
}

/// A project: its root and the folder's paths.
#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    policy: PathPolicy,
}

fn policy(root: &Path) -> Result<PathPolicy> {
    custodian_kit::store::named::policy(root).map_err(|e| Refusal::NoProject.refuse(format!("{}: {e}", root.display())))
}

impl Project {
    /// The initialized folder at `start` or above.
    pub fn find(start: &Path) -> Result<Project> {
        let start = start.canonicalize().map_err(|e| Refusal::NoProject.refuse(format!("{}: {e}", start.display())))?;
        for dir in start.ancestors() {
            let p = Project { root: dir.to_path_buf(), policy: policy(dir)? };
            if p.path(&named::SETTINGS, &[])?.is_file() {
                return Ok(p);
            }
        }
        Err(Refusal::NoProject.refuse(start.display()))
    }

    /// The project at `root`, the folder there or not (custodian, which has found the folder already, opens it so).
    pub fn open(root: &Path) -> Result<Project> {
        Ok(Project { root: root.to_path_buf(), policy: policy(root)? })
    }

    /// A named file, its template filled.
    pub fn path(&self, d: &ArtifactOutputDescriptor, vars: &[(&str, &str)]) -> Result<PathBuf> {
        custodian_kit::store::named::resolve(&self.policy, d, vars).map_err(|e| Refusal::NoProject.refuse(format!("{}: {e}", self.root.display())))
    }

    fn dir_of(&self, d: &ArtifactOutputDescriptor, vars: &[(&str, &str)]) -> Result<PathBuf> {
        Ok(self.path(d, vars)?.parent().map(Path::to_path_buf).unwrap_or_default())
    }

    /// `data/receipts/`.
    pub fn receipts_dir(&self) -> Result<PathBuf> {
        self.dir_of(&named::RECEIPT, &[("seq", "1")])
    }

    /// `data/history/`, the task histories' root.
    pub fn history_root(&self) -> Result<PathBuf> {
        self.dir_of(&named::HISTORY_INDEX, &[])
    }

    /// `data/trees/`, the tree snapshots.
    pub fn trees_dir(&self) -> Result<PathBuf> {
        self.dir_of(&named::TREE, &[("point", "x")])
    }

    /// `state/jobs/`, the job folders' root (a folder per state, a job's folder in it).
    pub fn jobs_root(&self) -> Result<PathBuf> {
        Ok(self.dir_of(&named::JOB_FILE, &[("state", "x"), ("job", "x")])?.parent().and_then(Path::parent).map(Path::to_path_buf).unwrap_or_default())
    }

    /// Every job folder, with its state, in custodian_kit's state order (`queued`, `running`, `done`), each state's
    /// folders in name order.
    pub fn job_folders(&self) -> Result<Vec<(String, PathBuf)>> {
        Ok(custodian_kit::store::job_folders(&self.jobs_root()?))
    }

    /// The journal, brought up to the receipt files the folder carries (a
    /// fresh checkout's journal is rebuilt from them).
    pub fn journal(&self) -> Result<Journal> {
        let j = Journal::open(&self.path(&named::JOURNAL, &[])?)?;
        let dir = self.receipts_dir()?;
        let on_disk = receipt_history::journal::receipt_files(&dir).len() as i64;
        if j.count()? < on_disk {
            j.rebuild_from(&dir, |_, _| Ok(()))?;
        }
        Ok(j)
    }

    /// A path a receipt names (relative to the root), made absolute.
    pub fn at(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }
}
