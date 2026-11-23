//! Prepared canonical movement and its post-publication completion custody.
use super::*;
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::VersionId;

pub(in crate::authority::commit::pipeline::publication_execution) struct PreparedPublicationPhase {
    pub(in crate::authority::commit::pipeline::publication_execution) live_inputs:
        Option<crate::publication::live_publication_inputs::BoundLivePublicationInputs>,
    pub(in crate::authority::commit::pipeline::publication_execution) application_unchanged: bool,
    pub(in crate::authority::commit::pipeline::publication_execution) record_allocations:
        crate::runtime::state::subsystems::record_identity::PendingRecordAllocations,
    pub(in crate::authority::commit::pipeline::publication_execution) symbol_meanings:
        crate::runtime::state::subsystems::services::symbol_table::PreparedSymbolPublication,
    // Keep the same symbol-owner share until this prepared phase is consumed.
    pub(in crate::authority::commit::pipeline::publication_execution) _candidate_symbols:
        crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    pub(in crate::authority::commit::pipeline::publication_execution) mirror_publication:
        crate::storage::PreparedMirrorPublication,
    pub(in crate::authority::commit::pipeline::publication_execution) prepared_history:
        crate::mvcc::publication::authority::PreparedRelationalPublication,
    pub(in crate::authority::commit::pipeline::publication_execution) validated_lineage_events:
        crate::runtime::state::subsystems::lineage::ValidatedLineageEventBatch,
    pub(in crate::authority::commit::pipeline::publication_execution) lineage_nodes:
        Vec<crate::lineage::data::LineageNode>,
    pub(in crate::authority::commit::pipeline::publication_execution) relation_lineages: Vec<(
        crate::identity::data::RelationId,
        crate::identity::data::LineageId,
    )>,
    pub(in crate::authority::commit::pipeline::publication_execution) artifacts:
        crate::publication::authority::artifacts::PublicationArtifacts,
    pub(in crate::authority::commit::pipeline::publication_execution) changed_records:
        Vec<crate::transactions::data::primitives::RecordRef>,
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
        crate::runtime::state::subsystems::history::canonical_publication_routes::PreparedCanonicalPublicationRoute,
}

pub(in crate::authority::commit::pipeline::publication_execution) struct PreparedPublicationCompletion
{
    pub(in crate::authority::commit::pipeline::publication_execution) mirror_publication:
        crate::storage::PreparedMirrorPublication,
    pub(in crate::authority::commit::pipeline::publication_execution) prepared_history:
        crate::mvcc::publication::authority::PreparedRelationalPublicationAccelerators,
    pub(in crate::authority::commit::pipeline::publication_execution) validated_lineage_events:
        crate::runtime::state::subsystems::lineage::ValidatedLineageEventBatch,
    pub(in crate::authority::commit::pipeline::publication_execution) lineage_nodes:
        Vec<crate::lineage::data::LineageNode>,
    pub(in crate::authority::commit::pipeline::publication_execution) relation_lineages: Vec<(
        crate::identity::data::RelationId,
        crate::identity::data::LineageId,
    )>,
    pub(in crate::authority::commit::pipeline::publication_execution) artifacts:
        crate::publication::authority::artifacts::PublicationArtifacts,
    pub(in crate::authority::commit::pipeline::publication_execution) changed_records:
        Vec<crate::transactions::data::primitives::RecordRef>,
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
    ) -> Option<crate::history::data::RelationalCallerAssociation> {
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
    ) -> &Arc<crate::branch::root::RelationalBranchRoot> {
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
        crate::mvcc::publication::PreparedCanonicalBranchMovement,
        PreparedPublicationCompletion,
    ) {
        let (next_cell, root, _envelope, prepared_history) =
            self.prepared_history.into_canonical_parts();
        let movement = crate::mvcc::publication::PreparedCanonicalBranchMovement::new(
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
            relation_lineages: self.relation_lineages,
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

#[allow(clippy::too_many_arguments)]
pub(super) fn complete_deferred_lineage(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    lineage_authority: crate::authority::commit::lineage_preparation::LineagePreparationAuthority<
        '_,
    >,
    working_state: &mut crate::storage::overlay::WorkingState,
    envelope: &mut crate::history::data::CanonicalCommitEnvelopeDraft,
    changed_records: &[crate::transactions::data::primitives::RecordRef],
    created_entities: &crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    created_relations: &crate::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
    naming_plan: Option<&crate::lineage::data::CommitNamingPlan>,
) -> Result<
    (
        Vec<crate::lineage::data::LineageNode>,
        Vec<(
            crate::identity::data::RelationId,
            crate::identity::data::LineageId,
        )>,
    ),
    TransactionCommitError,
> {
    let graph_selection = match &envelope.authority_kind {
        crate::history::data::CanonicalCommitAuthorityKind::OwnedGraphSelection(selection) => {
            Some(std::sync::Arc::clone(selection))
        }
        _ => None,
    };
    let prepared = lineage_authority.ensure_lineage_for_commit(
        execution,
        working_state,
        &envelope.commit,
        &envelope.merged_plan.merged_intents,
        changed_records,
        created_entities,
        created_relations,
        naming_plan,
        graph_selection.as_deref(),
    )?;
    let (lineage, nodes, relations) = prepared.into_parts();
    envelope.complete_lineage(lineage)?;
    Ok((nodes, relations))
}
