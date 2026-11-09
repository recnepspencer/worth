//! Prepared canonical movement and its post-publication completion custody.
use super::*;

pub(in crate::authority::commit::pipeline::publication_execution) struct PreparedPublicationPhase {
    pub(in crate::authority::commit::pipeline::publication_execution) record_allocations:
        crate::runtime::PendingRecordAllocations,
    pub(in crate::authority::commit::pipeline::publication_execution) symbol_meanings:
        crate::runtime::PreparedSymbolPublication,
    // Keep the same symbol-owner share until this prepared phase is consumed.
    pub(in crate::authority::commit::pipeline::publication_execution) _candidate_symbols:
        crate::runtime::RuntimeSymbolTable,
    pub(in crate::authority::commit::pipeline::publication_execution) mirror_publication:
        crate::storage::PreparedMirrorPublication,
    pub(in crate::authority::commit::pipeline::publication_execution) prepared_history:
        crate::mvcc::PreparedRelationalPublication,
    pub(in crate::authority::commit::pipeline::publication_execution) validated_lineage_events:
        crate::runtime::ValidatedLineageEventBatch,
    pub(in crate::authority::commit::pipeline::publication_execution) lineage_nodes:
        Vec<crate::lineage::data::LineageNode>,
    pub(in crate::authority::commit::pipeline::publication_execution) artifacts:
        crate::storage::overlay::PublicationArtifacts,
    pub(in crate::authority::commit::pipeline::publication_execution) changed_records:
        Vec<crate::transactions::data::RecordRef>,
    pub(in crate::authority::commit::pipeline::publication_execution) version_id: VersionId,
    pub(in crate::authority::commit::pipeline::publication_execution) previous_branch_head_version:
        Option<VersionId>,
    pub(in crate::authority::commit::pipeline::publication_execution) commit_id: CommitId,
    pub(in crate::authority::commit::pipeline::publication_execution) commit_reference:
        RelationalCommitReceipt,
    pub(in crate::authority::commit::pipeline::publication_execution) branch_id: BranchId,
    pub(in crate::authority::commit::pipeline::publication_execution) merge_base_commits:
        Vec<CommitId>,
    pub(in crate::authority::commit::pipeline::publication_execution) merge_parent_branches:
        Vec<BranchId>,
    pub(in crate::authority::commit::pipeline::publication_execution) deferred_diagnostic_artifacts:
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
    pub(in crate::authority::commit::pipeline::publication_execution) canonical_publication_route:
        crate::runtime::PreparedCanonicalPublicationRoute,
}

pub(in crate::authority::commit::pipeline::publication_execution) struct PreparedPublicationCompletion
{
    pub(in crate::authority::commit::pipeline::publication_execution) mirror_publication:
        crate::storage::PreparedMirrorPublication,
    pub(in crate::authority::commit::pipeline::publication_execution) prepared_history:
        crate::mvcc::PreparedRelationalPublicationAccelerators,
    pub(in crate::authority::commit::pipeline::publication_execution) validated_lineage_events:
        crate::runtime::ValidatedLineageEventBatch,
    pub(in crate::authority::commit::pipeline::publication_execution) lineage_nodes:
        Vec<crate::lineage::data::LineageNode>,
    pub(in crate::authority::commit::pipeline::publication_execution) artifacts:
        crate::storage::overlay::PublicationArtifacts,
    pub(in crate::authority::commit::pipeline::publication_execution) changed_records:
        Vec<crate::transactions::data::RecordRef>,
    pub(in crate::authority::commit::pipeline::publication_execution) version_id: VersionId,
    pub(in crate::authority::commit::pipeline::publication_execution) previous_branch_head_version:
        Option<VersionId>,
    pub(in crate::authority::commit::pipeline::publication_execution) commit_id: CommitId,
    pub(in crate::authority::commit::pipeline::publication_execution) commit_reference:
        RelationalCommitReceipt,
    pub(in crate::authority::commit::pipeline::publication_execution) branch_id: BranchId,
    pub(in crate::authority::commit::pipeline::publication_execution) merge_base_commits:
        Vec<CommitId>,
    pub(in crate::authority::commit::pipeline::publication_execution) merge_parent_branches:
        Vec<BranchId>,
    pub(in crate::authority::commit::pipeline::publication_execution) deferred_diagnostic_artifacts:
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
}

impl PreparedPublicationPhase {
    pub(in crate::authority::commit::pipeline::publication_execution) fn caller_association(
        &self,
    ) -> Option<crate::mvcc::RelationalCallerAssociation> {
        self.prepared_history.caller_association().cloned()
    }
    pub(in crate::authority::commit::pipeline::publication_execution) fn canonical_envelope(
        &self,
    ) -> &Arc<crate::history::data::CanonicalCommitEnvelope> {
        self.prepared_history.canonical_envelope()
    }
    pub(in crate::authority::commit::pipeline::publication_execution) fn descriptive_touches(
        &self,
    ) -> &crate::history::data::RelationalDescriptiveTouchGraph {
        self.prepared_history.descriptive_touches()
    }

    pub(in crate::authority::commit::pipeline::publication_execution) fn changed_record_count(
        &self,
    ) -> usize {
        self.changed_records.len()
    }

    pub(in crate::authority::commit::pipeline::publication_execution) fn reservation_count(
        &self,
    ) -> usize {
        self.record_allocations.reservation_count()
    }

    pub(in crate::authority::commit::pipeline::publication_execution) fn prepared_root(
        &self,
    ) -> &Arc<crate::branch::RelationalBranchRoot> {
        self.prepared_history.root()
    }

    #[cfg(test)]
    pub(in crate::authority::commit::pipeline::publication_execution) fn materialization_counts(
        &self,
    ) -> (u64, u64) {
        self.prepared_history.materialization_counts()
    }

    pub(crate) fn split(
        self,
    ) -> (
        crate::mvcc::PreparedCanonicalBranchMovement,
        PreparedPublicationCompletion,
    ) {
        let (next_cell, root, _envelope, prepared_history) =
            self.prepared_history.into_canonical_parts();
        let movement = crate::mvcc::PreparedCanonicalBranchMovement::new(
            self.record_allocations,
            self.symbol_meanings,
            next_cell,
            root,
            self.canonical_publication_route,
        );
        let completion = PreparedPublicationCompletion {
            mirror_publication: self.mirror_publication,
            prepared_history,
            validated_lineage_events: self.validated_lineage_events,
            lineage_nodes: self.lineage_nodes,
            artifacts: self.artifacts,
            changed_records: self.changed_records,
            version_id: self.version_id,
            previous_branch_head_version: self.previous_branch_head_version,
            commit_id: self.commit_id,
            commit_reference: self.commit_reference,
            branch_id: self.branch_id,
            merge_base_commits: self.merge_base_commits,
            merge_parent_branches: self.merge_parent_branches,
            deferred_diagnostic_artifacts: self.deferred_diagnostic_artifacts,
        };
        (movement, completion)
    }
}
