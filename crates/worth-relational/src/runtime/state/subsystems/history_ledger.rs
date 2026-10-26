use std::collections::BTreeMap;
use std::sync::Arc;

use crate::history::data::{CommitId, VersionNode};
use crate::history::RelationalCommitCatalog;
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;

/// The immutable commit catalog and its durable recovery sidecars, held
/// together because publication installs all of them under one decision.
///
/// Keeping them in one cell is what makes a published commit atomic to any
/// concurrent reader: nobody can observe a catalog artifact whose envelope,
/// graph node, or patch-stream position has not landed yet.
#[derive(Debug, Clone)]
pub(crate) struct HistoryLedger {
    pub(crate) commit_catalog: RelationalCommitCatalog,
    /// Durable recovery/diagnostic sidecar. Currentness and fork identity
    /// read the catalog, not this map.
    pub(crate) commit_graph: BTreeMap<CommitId, VersionNode>,
    /// Durable recovery sidecar holding the same sealed envelope the catalog
    /// already admitted. It cannot mint a branch cell or a fork basis.
    pub(crate) commit_envelopes:
        BTreeMap<CommitId, Arc<crate::history::RelationalCommitEnvelopeResidency>>,
    pub(crate) patch_stream_index: BTreeMap<PatchStreamPosition, CommitId>,
    pub(crate) next_commit_id: u64,
    pub(crate) next_version_id: u64,
}

impl Default for HistoryLedger {
    fn default() -> Self {
        Self {
            commit_catalog: RelationalCommitCatalog::default(),
            commit_graph: BTreeMap::new(),
            commit_envelopes: BTreeMap::new(),
            patch_stream_index: BTreeMap::new(),
            next_commit_id: 1,
            next_version_id: 1,
        }
    }
}

impl HistoryLedger {
    pub(crate) fn install_published_commit(
        &mut self,
        commit_id: CommitId,
        commit_reference: crate::history::data::RelationalCommitReceipt,
        artifact: crate::history::RelationalCommitArtifact,
        patch_position: PatchStreamPosition,
        recovery_readmission: bool,
    ) {
        let envelope = artifact.envelope_residency();
        if recovery_readmission {
            self.commit_catalog.install_prepared_recovery(artifact);
        } else {
            self.commit_catalog.install_prepared(artifact);
        }
        self.next_commit_id = self.next_commit_id.max(
            commit_id
                .0
                .checked_add(1)
                .expect("reserved commit id has successor"),
        );
        self.next_version_id = self.next_version_id.max(
            commit_reference
                .version_id
                .0
                .checked_add(1)
                .expect("reserved version id has successor"),
        );
        self.commit_graph.insert(
            commit_id,
            VersionNode {
                commit: commit_reference,
            },
        );
        self.commit_envelopes.insert(commit_id, envelope);
        self.patch_stream_index.insert(patch_position, commit_id);
    }

    pub(crate) fn set_sequence(&mut self, next_commit_id: u64, next_version_id: u64) {
        self.next_commit_id = next_commit_id;
        self.next_version_id = next_version_id;
    }

    pub(crate) const fn preview_next_commit_id(&self) -> CommitId {
        CommitId(self.next_commit_id)
    }

    pub(crate) const fn current_version_id(&self) -> VersionId {
        VersionId(self.next_version_id.saturating_sub(1))
    }
}
