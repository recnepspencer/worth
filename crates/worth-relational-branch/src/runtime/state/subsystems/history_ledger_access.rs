use std::sync::Arc;

use crate::history::commit::RelationalCommitArtifact;
use crate::history::commit::RelationalCommitCatalog;
use crate::history::commit::RelationalCommitCatalogEnvelopeAppendDenial;
use crate::history::commit::RelationalCommitIdentity;
use crate::history::data::{CanonicalCommitEnvelope, CommitId, VersionNode};
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;

use super::{HistoryLedger, HistorySubsystem};

/// Ledger truth read through shared ownership.
///
/// Every accessor carries its answer out of the ledger lock: catalog artifacts
/// and canonical envelopes are already `Arc`, and the sidecar entries are small
/// enough to copy. No accessor lends a borrow into the lock, so no caller can
/// hold the ledger across storage, durability, or projection work.
impl HistorySubsystem {
    pub fn try_commit_artifact(
        &self,
        commit_id: CommitId,
    ) -> Result<
        Option<Arc<RelationalCommitArtifact>>,
        worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    >{
        self.ledger.read().try_catalog_artifact(commit_id)
    }

    pub fn commit_artifact(&self, commit_id: CommitId) -> Option<Arc<RelationalCommitArtifact>> {
        self.ledger.read().catalog_artifact(commit_id)
    }

    pub fn commit_receipt(
        &self,
        commit_id: CommitId,
    ) -> Option<crate::history::data::RelationalCommitReceipt> {
        self.ledger
            .read()
            .try_catalog_receipt(commit_id)
            .ok()
            .flatten()
    }

    pub fn commit_identity(&self, commit_id: CommitId) -> Option<RelationalCommitIdentity> {
        self.ledger
            .read()
            .try_catalog_identity(commit_id)
            .ok()
            .flatten()
    }

    pub fn commit_receipt_for_version(
        &self,
        version_id: VersionId,
    ) -> Option<crate::history::data::RelationalCommitReceipt> {
        self.ledger
            .read()
            .try_catalog_receipt_for_version(version_id)
            .ok()
            .flatten()
    }

    pub fn latest_commit_receipt(&self) -> Option<crate::history::data::RelationalCommitReceipt> {
        self.ledger
            .read()
            .try_latest_catalog_receipt()
            .ok()
            .flatten()
    }

    pub fn latest_commit_artifact(&self) -> Option<Arc<RelationalCommitArtifact>> {
        self.ledger.read().latest_catalog_artifact()
    }

    pub fn latest_commit_identity(&self) -> Option<RelationalCommitIdentity> {
        use crate::history::commit::identity::RelationalCommitIdentityOwnerPort as _;
        self.latest_commit_receipt().map(|receipt| {
            RelationalCommitIdentity::new(receipt.commit_id, receipt.version_id, receipt.branch_id)
        })
    }

    pub fn try_commit_artifact_for_version(
        &self,
        version_id: VersionId,
    ) -> Result<
        Option<Arc<RelationalCommitArtifact>>,
        worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    >{
        self.ledger
            .read()
            .try_catalog_artifact_for_version(version_id)
    }

    pub fn commit_artifact_for_version(
        &self,
        version_id: VersionId,
    ) -> Option<Arc<RelationalCommitArtifact>> {
        self.ledger.read().catalog_artifact_for_version(version_id)
    }

    pub fn try_commit_artifacts(
        &self,
    ) -> Result<
        Vec<Arc<RelationalCommitArtifact>>,
        worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    >{
        self.ledger.read().try_catalog_artifacts()
    }

    pub fn commit_artifacts_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_execution::ExecutionArray<Arc<RelationalCommitArtifact>>,
        worth_execution::ExecutionRequestArrayDenial,
    > {
        let artifacts = self.commit_artifacts();
        let mut rows =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*request, artifacts.len())?;
        for artifact in artifacts {
            rows.push(artifact)?;
        }
        rows.seal()
    }

    /// Every catalog artifact. After a cold reopen this faults every commit
    /// page; if one is unreadable it answers with the resident catalog only.
    pub fn commit_artifacts(&self) -> Vec<Arc<RelationalCommitArtifact>> {
        let ledger = self.ledger.read();
        ledger
            .try_catalog_artifacts()
            .unwrap_or_else(|_| ledger.resident_catalog_artifacts())
    }

    /// The resident catalog and faulted cold artifacts still held: what a
    /// checkpoint capture must rebind, without reading any commit page.
    pub fn try_resident_commit_artifacts(
        &self,
    ) -> Result<
        Vec<Arc<RelationalCommitArtifact>>,
        worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    >{
        let mut artifacts = self.ledger.read().resident_catalog_artifacts();
        // Every branch's selected commit stays capturable even when no resident
        // artifact holds it (a fork of a cold commit faults it here).
        let mut held = artifacts
            .iter()
            .map(|artifact| artifact.commit_id())
            .collect::<std::collections::BTreeSet<_>>();
        for cell in self.branch_cells.values() {
            if let worth_foundational::FoundationalBranchTarget::Basis(target) =
                cell.observation().target()
            {
                let id = CommitId(target.selected_commit_id());
                if held.insert(id)
                    && let Some(artifact) = self.try_commit_artifact(id)?
                {
                    artifacts.push(artifact);
                }
            }
        }
        artifacts.sort_by_key(|artifact| artifact.commit_id());
        Ok(artifacts)
    }

    pub fn resident_commit_artifacts(&self) -> Vec<Arc<RelationalCommitArtifact>> {
        self.try_resident_commit_artifacts()
            .unwrap_or_else(|_| self.ledger.read().resident_catalog_artifacts())
    }

    /// Largest commit and version ids, answered from page descriptors.
    pub fn try_max_commit_and_version(
        &self,
    ) -> Result<
        (Option<u64>, Option<u64>),
        worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    >{
        self.ledger.read().try_max_commit_and_version()
    }

    pub fn max_commit_and_version(&self) -> (Option<u64>, Option<u64>) {
        self.ledger.read().max_commit_and_version()
    }

    pub fn try_has_commit(
        &self,
        commit_id: CommitId,
    ) -> Result<
        bool,
        worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    >{
        self.ledger.read().try_has_commit(commit_id)
    }

    pub fn has_commit(&self, commit_id: CommitId) -> bool {
        self.ledger.read().has_commit(commit_id)
    }

    /// What a checkpoint catalog installs: resident artifacts over the cold
    /// base, without enumerating cold commits.
    pub fn checkpoint_catalog_parts(
        &self,
    ) -> (
        Vec<Arc<RelationalCommitArtifact>>,
        Option<Arc<crate::history::commit::cold_catalog::ColdCommitCatalog>>,
    ) {
        let ledger = self.ledger.read();
        (
            ledger.resident_catalog_artifacts(),
            ledger.cold_catalog.clone(),
        )
    }

    pub fn cold_commit_catalog(
        &self,
    ) -> Option<Arc<crate::history::commit::cold_catalog::ColdCommitCatalog>> {
        self.ledger.read().cold_catalog.clone()
    }

    pub fn catalog_len(&self) -> usize {
        self.ledger.read().catalog_len()
    }

    pub fn validate_catalog_envelope(
        &self,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), RelationalCommitCatalogEnvelopeAppendDenial> {
        self.ledger.read().validate_envelope(envelope)
    }

    pub fn validate_new_catalog_envelope(
        &self,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), RelationalCommitCatalogEnvelopeAppendDenial> {
        self.ledger.read().validate_new_envelope(envelope)
    }

    pub fn install_commit_catalog(&self, catalog: RelationalCommitCatalog) {
        self.ledger.write().commit_catalog = catalog;
    }

    pub fn recorded_commit_envelope(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.ledger
            .read()
            .commit_envelopes
            .get(&commit_id)
            .and_then(|cell| cell.get())
    }

    pub fn recorded_commit_envelopes(&self) -> Vec<Arc<CanonicalCommitEnvelope>> {
        self.ledger
            .read()
            .commit_envelopes
            .values()
            .filter_map(|cell| cell.get())
            .collect()
    }

    pub fn recorded_commit_envelope_count(&self) -> usize {
        self.ledger.read().commit_envelopes.len()
    }

    pub fn has_recorded_commit_envelope(&self, commit_id: CommitId) -> bool {
        self.ledger.read().commit_envelopes.contains_key(&commit_id)
    }

    pub fn latest_recorded_patch_position(&self) -> Option<PatchStreamPosition> {
        self.ledger.read().latest_patch_position()
    }

    pub fn try_recorded_commit_at_patch_position(
        &self,
        position: PatchStreamPosition,
    ) -> Result<
        Option<CommitId>,
        worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    >{
        self.ledger.read().try_commit_at_patch_position(position)
    }

    pub fn recorded_commit_at_patch_position(
        &self,
        position: PatchStreamPosition,
    ) -> Option<CommitId> {
        self.ledger.read().commit_at_patch_position(position)
    }

    pub fn try_recorded_patch_positions_after(
        &self,
        after_position: Option<PatchStreamPosition>,
        max_commits: usize,
    ) -> Result<
        Vec<(PatchStreamPosition, CommitId)>,
        worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    >{
        self.ledger
            .read()
            .try_patch_positions_after(after_position, max_commits)
    }

    pub fn recorded_patch_positions_after(
        &self,
        after_position: Option<PatchStreamPosition>,
        max_commits: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        self.ledger
            .read()
            .patch_positions_after(after_position, max_commits)
    }

    pub fn latest_recorded_patch_references(
        &self,
        limit: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        self.ledger.read().latest_patch_references(limit)
    }

    pub fn recorded_patch_position_count_for_test(&self) -> usize {
        self.ledger.read().patch_position_count() as usize
    }

    pub fn commit_graph_len_for_test(&self) -> usize {
        let ledger = self.ledger.read();
        match &ledger.cold_catalog {
            None => ledger.commit_graph.len(),
            Some(_) => ledger.patch_position_count() as usize,
        }
    }

    pub fn insert_commit_graph_node(&self, commit_id: CommitId, node: VersionNode) {
        self.ledger.write().commit_graph.insert(commit_id, node);
    }

    /// The reserved commit/version identity floors, read together so a court
    /// cannot observe half of an advanced sequence.
    pub fn reserved_sequence_floors_for_test(&self) -> (u64, u64) {
        let ledger = self.ledger.read();
        (ledger.next_commit_id, ledger.next_version_id)
    }

    /// Run one bounded, self-contained edit against the whole ledger.
    ///
    /// Callers must not re-enter the history subsystem from inside the closure;
    /// recovery paths read what they need first, build the replacement, and
    /// install it here.
    pub fn with_ledger_mut<T>(&self, edit: impl FnOnce(&mut HistoryLedger) -> T) -> T {
        edit(&mut self.ledger.write())
    }

    pub fn ledger_snapshot(&self) -> HistoryLedger {
        self.ledger.read().clone()
    }
}
