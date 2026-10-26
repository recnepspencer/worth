use crate::branch::{RelationalBranchReferenceObservation, RelationalBranchVersion};
#[cfg(test)]
use crate::history::data::CanonicalCommitEnvelope;
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::publication::patch::data::PatchStreamPosition;

use super::HistoryAccess;

impl<'runtime> HistoryAccess<'runtime> {
    /// Read the canonical head carried by one owner-admitted repeatable
    /// observation. No live branch cell or catalog-latest lookup participates.
    pub fn branch_head_for_observation(
        &self,
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> Result<Option<RelationalCommitReceipt>, crate::branch::RelationalBranchBasisDenial> {
        if observation.identity().runtime_instance_id() != self.runtime.runtime_instance_id() {
            return Err(crate::branch::RelationalBranchBasisDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime.runtime_instance_id(),
                actual_runtime_instance_id: observation.identity().runtime_instance_id(),
            });
        }
        Ok(observation.commit_receipt().cloned())
    }

    /// Diagnostic immutable-identity lookup. This returns commit identity only
    /// and cannot select a branch head or a visible read root.
    pub fn immutable_commit_identity(
        &self,
        commit_id: CommitId,
    ) -> Option<crate::history::RelationalCommitIdentity> {
        self.runtime
            .history
            .commit_artifact(commit_id)
            .map(|artifact| artifact.identity().clone())
            .or_else(|| {
                self.runtime
                    .history
                    .canonical_envelope(commit_id)
                    .map(|envelope| {
                        crate::history::RelationalCommitIdentity::new(
                            envelope.commit.commit_id,
                            envelope.commit.version_id,
                            envelope.branch_context.clone(),
                        )
                    })
            })
    }

    /// Return the immutable catalog receipt, including ordered parentage, for
    /// evidence consumers that need to compare the canonical artifact without
    /// receiving a mutable history authority.
    pub fn immutable_commit_receipt(&self, commit_id: CommitId) -> Option<RelationalCommitReceipt> {
        self.runtime
            .history
            .commit_artifact(commit_id)
            .map(|artifact| artifact.receipt())
            .or_else(|| {
                self.runtime
                    .history
                    .canonical_envelope(commit_id)
                    .map(|envelope| envelope.commit.clone())
            })
    }

    pub fn latest_catalog_commit_identity(
        &self,
    ) -> Option<crate::history::RelationalCommitIdentity> {
        self.runtime.history.latest_commit_identity()
    }

    pub fn immutable_commit_count(&self) -> usize {
        let mut commit_ids = self
            .runtime
            .history
            .commit_artifacts()
            .into_iter()
            .map(|artifact| artifact.commit_id())
            .collect::<std::collections::BTreeSet<_>>();
        commit_ids.extend(
            self.runtime
                .history
                .performed_canonical_envelopes()
                .into_iter()
                .map(|envelope| envelope.commit.commit_id),
        );
        commit_ids.len()
    }

    /// Latest canonical commit in performed stream order. The returned value
    /// is immutable evidence and cannot select a branch cell or mint
    /// currentness authority.
    pub fn historical_latest_commit(&self) -> Option<RelationalCommitReceipt> {
        self.latest_patch_stream_position()
            .and_then(|position| {
                self.runtime
                    .history
                    .canonical_envelope_at_patch(position)
                    .map(|envelope| envelope.commit.clone())
                    .or_else(|| {
                        self.runtime
                            .history
                            .recorded_commit_at_patch_position(position)
                            .and_then(|commit_id| self.runtime.history.commit_artifact(commit_id))
                            .map(|artifact| artifact.receipt())
                    })
            })
            .or_else(|| {
                self.runtime
                    .history
                    .latest_commit_artifact()
                    .map(|artifact| artifact.receipt())
            })
    }

    /// Resolve immutable version identity from the catalog index without loading a body.
    pub fn immutable_commit_receipt_for_version(
        &self,
        version_id: crate::identity::data::VersionId,
    ) -> Option<RelationalCommitReceipt> {
        self.runtime
            .history
            .commit_artifact_for_version(version_id)
            .map(|artifact| artifact.receipt())
            .or_else(|| {
                self.runtime
                    .history
                    .canonical_envelope_for_version(version_id)
                    .map(|envelope| envelope.commit.clone())
            })
    }

    /// The durable ledger's latest patch-stream position.
    ///
    /// Recorded commits and recovered canonical routes both count, so a
    /// runtime restored from a checkpoint or replayed from its log reports the
    /// same position it held before. Patch-stream reads resolve against this
    /// position; a reader that resumes the stream starts after it.
    pub fn latest_patch_stream_position(&self) -> Option<PatchStreamPosition> {
        let projected = self.runtime.history.latest_recorded_patch_position();
        projected.max(
            self.runtime
                .history
                .latest_canonical_patch_route()
                .map(|(position, _)| position),
        )
    }

    pub(crate) fn canonical_stream_position(
        &self,
        commit_id: CommitId,
    ) -> Option<PatchStreamPosition> {
        self.runtime.history.canonical_stream_position(commit_id)
    }

    pub(crate) fn contains_patch_stream_position(&self, position: PatchStreamPosition) -> bool {
        self.runtime
            .history
            .recorded_commit_at_patch_position(position)
            .is_some()
            || self
                .runtime
                .history
                .canonical_envelope_at_patch(position)
                .is_some()
    }

    #[cfg(test)]
    pub(crate) fn commit_envelopes_snapshot(&self) -> Vec<CanonicalCommitEnvelope> {
        let mut envelopes = self
            .runtime
            .history
            .commit_artifacts()
            .into_iter()
            .map(|artifact| (artifact.commit_id(), artifact.envelope().as_ref().clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        for envelope in self.runtime.history.performed_canonical_envelopes() {
            envelopes.insert(envelope.commit.commit_id, envelope.as_ref().clone());
        }
        envelopes.into_values().collect()
    }

    pub(crate) fn positioned_commits_snapshot(
        &self,
    ) -> Vec<crate::history::data::PositionedCanonicalCommit> {
        self.runtime
            .history
            .positioned_canonical_commits_snapshot()
            .into_iter()
            .map(|commit| commit.as_ref().clone())
            .collect()
    }

    pub(crate) fn branch_cells_snapshot(
        &self,
    ) -> Vec<crate::branch::RelationalBranchCellCheckpoint> {
        self.runtime.history.branch_cells_snapshot()
    }

    pub(crate) fn next_commit_id(&self) -> CommitId {
        self.runtime.history.preview_next_commit_id()
    }

    #[cfg(test)]
    pub(crate) fn preview_next_version_id(&self) -> crate::identity::data::VersionId {
        self.runtime.history.preview_next_version_id()
    }

    pub(crate) fn commit_count(&self) -> usize {
        self.runtime.history.catalog_len()
    }

    pub(crate) fn branch_head(&self, branch_id: &BranchId) -> Option<RelationalCommitReceipt> {
        let cell = self.runtime.history.branch_cell(branch_id)?;
        if let Some(root) = cell.root() {
            return root
                .canonical_envelope()
                .map(|envelope| envelope.commit.clone());
        }
        // Rootless cells exist only in legacy recovery fixtures. Operational
        // cells always select their head through the immutable root above.
        let commit_id = match cell.observation().target() {
            worth_foundational::FoundationalBranchTarget::Empty => return None,
            worth_foundational::FoundationalBranchTarget::Basis(target) => {
                CommitId(target.selected_commit_id())
            }
        };
        self.runtime
            .history
            .commit_artifact(commit_id)
            .map(|artifact| artifact.receipt())
    }

    pub(crate) fn latest_commit(&self) -> Option<RelationalCommitReceipt> {
        self.historical_latest_commit()
    }

    pub(crate) fn latest_canonical_commit_id(&self) -> Option<CommitId> {
        self.latest_patch_stream_position()
            .and_then(|position| {
                self.runtime
                    .history
                    .canonical_envelope_at_patch(position)
                    .map(|envelope| envelope.commit.commit_id)
                    .or_else(|| {
                        self.runtime
                            .history
                            .recorded_commit_at_patch_position(position)
                    })
            })
            .or_else(|| self.latest_commit().map(|commit| commit.commit_id))
    }

    pub(crate) fn branch_reference_state(
        &self,
        branch_id: &BranchId,
    ) -> Option<(
        RelationalBranchReferenceObservation,
        RelationalBranchVersion,
    )> {
        self.runtime.history.branch_cell(branch_id).map(|cell| {
            let state = cell.evidence_state();
            (state.observation().clone(), state.truth_version())
        })
    }

    pub(crate) fn recent_commit_ids(
        &self,
        branch_id: Option<&BranchId>,
        limit: usize,
    ) -> Vec<CommitId> {
        match branch_id {
            Some(branch_id) => {
                let Some(head) = self.branch_head(branch_id) else {
                    return Vec::new();
                };
                let mut receipts = self
                    .ancestor_set(head.commit_id)
                    .into_iter()
                    .filter_map(|id| self.immutable_commit_receipt(id))
                    .filter(|receipt| receipt.branch_id == *branch_id)
                    .collect::<Vec<_>>();
                receipts.sort_by_key(|receipt| (receipt.version_id.0, receipt.commit_id.0));
                receipts
                    .into_iter()
                    .rev()
                    .take(limit)
                    .map(|receipt| receipt.commit_id)
                    .collect()
            }
            None => {
                let mut references = self
                    .runtime
                    .history
                    .latest_recorded_patch_references(limit)
                    .into_iter()
                    .collect::<std::collections::BTreeMap<_, _>>();
                references.extend(
                    self.runtime
                        .history
                        .latest_canonical_patch_references(limit),
                );
                references.into_values().rev().take(limit).collect()
            }
        }
    }
}
