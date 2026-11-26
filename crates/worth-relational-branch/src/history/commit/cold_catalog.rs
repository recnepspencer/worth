//! The cold base of a reopened commit catalog (`G7`).
//!
//! Reopen installs no artifact per commit. The ledger keeps a resident
//! catalog for commits published or recovered since reopen and consults this
//! cold base for every other commit: lookups fault only the commit-index
//! pages they touch (pages evict under the working set). A faulted artifact
//! is retained, so a touched commit keeps one identity and one envelope cell
//! (sticky damage, source replacement, close retirement): residency grows
//! with touched commits only, never with history length.
//! The source resolves the currently selected checkpoint, so a later
//! checkpoint that covers the same prefix serves later faults and no cold
//! base ever holds a checkpoint directory itself.
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;

use super::index_entry::RelationalCommitIndexEntry as Entry;
use super::paged_index::reader::ColdCommitIndex;
use super::RelationalCommitArtifact;
use crate::history::data::CommitId;
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;

/// The selected checkpoint's commit index and the artifact it places.
pub trait ColdCommitSource: Send + Sync + std::fmt::Debug {
    /// The currently selected index, or `None` once its directory retired.
    fn index(&self) -> Option<Arc<ColdCommitIndex>>;
    /// A catalog artifact for `entry`, placed in the selected directory.
    fn artifact(&self, entry: &Entry) -> Result<Arc<RelationalCommitArtifact>, Cause>;
}

#[derive(Debug)]
pub struct ColdCommitCatalog {
    source: Arc<dyn ColdCommitSource>,
    live: Mutex<BTreeMap<CommitId, Arc<RelationalCommitArtifact>>>,
}

impl ColdCommitCatalog {
    pub fn new(source: Arc<dyn ColdCommitSource>) -> Arc<Self> {
        Arc::new(Self {
            source,
            live: Mutex::new(BTreeMap::new()),
        })
    }

    pub fn index(&self) -> Result<Arc<ColdCommitIndex>, Cause> {
        self.source.index().ok_or(Cause::SourceUnavailable)
    }

    /// Commits the cold base covers; zero once its directory retired.
    pub fn count(&self) -> u64 {
        self.source.index().map_or(0, |index| index.count())
    }

    pub fn entry(&self, commit: CommitId) -> Result<Option<Entry>, Cause> {
        self.index()?.by_commit(commit)
    }

    pub fn artifact(
        &self,
        commit: CommitId,
    ) -> Result<Option<Arc<RelationalCommitArtifact>>, Cause> {
        if let Some(live) = self.live_artifact(commit) {
            return Ok(Some(live));
        }
        self.entry(commit)?
            .map(|entry| self.materialize(&entry))
            .transpose()
    }

    pub fn artifact_at(
        &self,
        position: PatchStreamPosition,
    ) -> Result<Option<Arc<RelationalCommitArtifact>>, Cause> {
        self.index()?
            .at_position(position)?
            .map(|entry| self.materialize(&entry))
            .transpose()
    }

    pub fn artifact_for_version(
        &self,
        version: VersionId,
    ) -> Result<Option<Arc<RelationalCommitArtifact>>, Cause> {
        self.index()?
            .by_version(version)?
            .map(|entry| self.materialize(&entry))
            .transpose()
    }

    /// The artifact with the largest commit id, found from descriptors.
    pub fn latest(&self) -> Result<Option<Arc<RelationalCommitArtifact>>, Cause> {
        match self.index()?.max_commit() {
            Some(commit) => self.artifact(commit),
            None => Ok(None),
        }
    }

    fn live_artifact(&self, commit: CommitId) -> Option<Arc<RelationalCommitArtifact>> {
        self.live
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(&commit)
            .cloned()
    }

    fn materialize(&self, entry: &Entry) -> Result<Arc<RelationalCommitArtifact>, Cause> {
        let commit = entry.commit.commit_id;
        let mut live = self.live.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(artifact) = live.get(&commit) {
            return Ok(artifact.clone());
        }
        let artifact = self.source.artifact(entry)?;
        live.insert(commit, artifact.clone());
        Ok(artifact)
    }

    /// Every faulted artifact; capture rebinds their cells, close retires them.
    pub fn live_artifacts(&self) -> Vec<Arc<RelationalCommitArtifact>> {
        self.live
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .values()
            .cloned()
            .collect()
    }

    /// How many cold commits this runtime has materialized.
    pub fn materialized(&self) -> usize {
        self.live
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .len()
    }

    /// Every cold artifact in position order. Whole-history consumers only.
    pub fn for_each_artifact(
        &self,
        visit: impl FnMut(Arc<RelationalCommitArtifact>),
    ) -> Result<(), Cause> {
        self.for_each_artifact_except(|_| false, visit)
    }

    /// Every cold artifact not skipped by `skip`, in position order.
    pub fn for_each_artifact_except(
        &self,
        mut skip: impl FnMut(CommitId) -> bool,
        mut visit: impl FnMut(Arc<RelationalCommitArtifact>),
    ) -> Result<(), Cause> {
        let index = self.index()?;
        for ordinal in 0..index.descriptor().pages.len() {
            let page = index.page(ordinal)?;
            for row in &page.page.rows {
                if !skip(row.commit.commit_id) {
                    visit(self.materialize(row)?);
                }
            }
        }
        Ok(())
    }
}
