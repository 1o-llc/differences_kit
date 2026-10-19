// Copyright © 2026 1o. All rights reserved.

//! The project: the folder holding `.custodian/`, and the files of it this
//! crate reads, named through pathgen as custodian_kit names them (a dev
//! policy whose base is the project root and whose app is `.custodian`, the
//! same route ids and templates), and the tree snapshots it keeps.

use crate::refusal::{Refusal, Result};
use pathgen::{AppPathRole, ArtifactEndpoint, ArtifactOutputDescriptor, PathMutability, PathOwner, PathPolicy, PathSensitivity, PathTemplateVars};
use receipt_history::Journal;
use std::path::{Path, PathBuf};

/// The initialized folder's name.
pub const FOLDER: &str = ".custodian";

const fn file(route_id: &'static str, kind: &'static str, endpoint: ArtifactEndpoint, base_role: AppPathRole, relative_template: &'static str, mutability: PathMutability) -> ArtifactOutputDescriptor {
    ArtifactOutputDescriptor {
        route_id,
        schema: "custodian.layout.v1",
        kind,
        version: 1,
        endpoint,
        base_role,
        relative_template,
        sensitivity: PathSensitivity::Internal,
        mutability,
        owner: PathOwner::CurrentUser,
    }
}

/// The files read here, each as custodian_kit's `store::named` declares it, and the one kept here.
pub mod named {
    use super::*;
    pub const SETTINGS: ArtifactOutputDescriptor = file("custodian.settings", "settings", ArtifactEndpoint::FileToml, AppPathRole::Config, "custodian.toml", PathMutability::Writable);
    pub const RECEIPT: ArtifactOutputDescriptor = file("custodian.receipt", "receipt", ArtifactEndpoint::FileJson, AppPathRole::Data, "receipts/{seq}.json", PathMutability::AppendOnly);
    pub const JOURNAL: ArtifactOutputDescriptor = file("custodian.journal", "journal", ArtifactEndpoint::FileTxt, AppPathRole::State, "journal.sqlite", PathMutability::Writable);
    pub const HISTORY_INDEX: ArtifactOutputDescriptor = file("custodian.history", "history", ArtifactEndpoint::FileTxt, AppPathRole::Data, "history/v1.history_index.1o.diff", PathMutability::Writable);
    /// A tree snapshot (`tree_history::tree`), one per point.
    pub const TREE: ArtifactOutputDescriptor = file("custodian.tree", "tree", ArtifactEndpoint::FileJson, AppPathRole::Data, "trees/{point}.json", PathMutability::AppendOnly);
}

/// A project: its root and the folder's paths.
#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    policy: PathPolicy,
}

fn policy(root: &Path) -> Result<PathPolicy> {
    PathPolicy::dev(FOLDER).map(|p| p.with_dev_base(root)).map_err(|e| Refusal::NoProject.refuse(format!("{}: {e}", root.display())))
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

    /// A named file, its template filled.
    pub fn path(&self, d: &ArtifactOutputDescriptor, vars: &[(&str, &str)]) -> Result<PathBuf> {
        let mut v = PathTemplateVars::new();
        for (k, val) in vars {
            v = v.with(k, val);
        }
        self.policy.artifact_output(d, &v).map(|r| r.path).map_err(|e| Refusal::NoProject.refuse(format!("{}: {e}", self.root.display())))
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
