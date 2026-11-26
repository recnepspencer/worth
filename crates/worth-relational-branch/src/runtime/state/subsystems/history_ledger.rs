use std::collections::BTreeMap;
use std::sync::Arc;

use crate::history::commit::RelationalCommitCatalog;
use crate::history::data::{CommitId, VersionNode};
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;

/// The immutable commit catalog and its durable recovery sidecars, held
/// together because publication installs all of them under one decision.
///
/// Keeping them in one cell is what makes a published commit atomic to any
/// concurrent reader: nobody can observe a catalog artifact whose envelope,
/// graph node, or patch-stream position has not landed yet.
#[derive(Debug, Clone)]
pub struct HistoryLedger {
    pub commit_catalog: RelationalCommitCatalog,
    /// Durable recovery/diagnostic sidecar. Currentness and fork identity
    /// read the catalog, not this map.
    pub commit_graph: BTreeMap<CommitId, VersionNode>,
    /// Durable recovery sidecar holding the same sealed envelope the catalog
    /// already admitted. It cannot mint a branch cell or a fork basis.
    pub commit_envelopes:
        BTreeMap<CommitId, Arc<crate::history::commit::RelationalCommitEnvelopeResidency>>,
    pub patch_stream_index: BTreeMap<PatchStreamPosition, CommitId>,
    /// `G7`: the selected checkpoint's commits, faulted on demand. The
    /// resident maps above hold only commits published or recovered since
    /// reopen and recovery overrides; they win wherever both answer.
    pub cold_catalog: Option<Arc<crate::history::commit::cold_catalog::ColdCommitCatalog>>,
    pub next_commit_id: u64,
    pub next_version_id: u64,
}

impl Default for HistoryLedger {
    fn default() -> Self {
        Self {
            commit_catalog: RelationalCommitCatalog::default(),
            commit_graph: BTreeMap::new(),
            commit_envelopes: BTreeMap::new(),
            patch_stream_index: BTreeMap::new(),
            cold_catalog: None,
            next_commit_id: 1,
            next_version_id: 1,
        }
    }
}

impl HistoryLedger {
    pub fn install_published_commit(
        &mut self,
        commit_id: CommitId,
        commit_reference: crate::history::data::RelationalCommitReceipt,
        artifact: Arc<crate::history::commit::RelationalCommitArtifact>,
        patch_position: PatchStreamPosition,
        recovery_readmission: bool,
        sequence: super::PreparedCommitSequence,
    ) {
        let envelope = artifact.envelope_residency();
        if recovery_readmission {
            self.commit_catalog.install_prepared_recovery(artifact);
        } else {
            self.commit_catalog.install_prepared(artifact);
        }
        let (next_commit, next_version) = sequence.into_successors();
        self.next_commit_id = self.next_commit_id.max(next_commit);
        self.next_version_id = self.next_version_id.max(next_version);
        self.commit_graph.insert(
            commit_id,
            VersionNode {
                commit: commit_reference,
            },
        );
        self.commit_envelopes.insert(commit_id, envelope);
        self.patch_stream_index.insert(patch_position, commit_id);
    }

    pub fn next_retained_envelope(
        &self,
        after: Option<CommitId>,
    ) -> Option<(
        CommitId,
        Arc<crate::history::commit::RelationalCommitEnvelopeResidency>,
    )> {
        use std::ops::Bound::{Excluded, Unbounded};
        let next = match after {
            None => self.commit_envelopes.first_key_value(),
            Some(key) => self
                .commit_envelopes
                .range((Excluded(key), Unbounded))
                .next(),
        };
        next.map(|(&id, cell)| (id, cell.clone()))
    }

    pub fn set_sequence(&mut self, next_commit_id: u64, next_version_id: u64) {
        self.next_commit_id = next_commit_id;
        self.next_version_id = next_version_id;
    }

    pub const fn preview_next_commit_id(&self) -> CommitId {
        CommitId(self.next_commit_id)
    }

    pub const fn current_version_id(&self) -> VersionId {
        VersionId(self.next_version_id.saturating_sub(1))
    }
}
