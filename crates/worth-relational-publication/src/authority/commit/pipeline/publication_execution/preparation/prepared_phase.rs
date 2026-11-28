//! Prepared canonical movement and its post-publication completion custody.
use super::*;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::history::data::coordinates::BranchId;
use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::primitives::VersionId;
use worth_relational_contracts::internal_port as contracts;

pub(in crate::internal_port::authority::commit::pipeline::publication_execution) struct PreparedPublicationPhase {
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) live_inputs:
        Option<crate::publication::live_publication_inputs::BoundLivePublicationInputs>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) application_unchanged: bool,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) record_allocations:
        worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::pending_allocations::PendingRecordAllocations,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) symbol_meanings:
        worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::reservations::PreparedSymbolPublication,
    // Keep the same symbol-owner share until this prepared phase is consumed.
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) _candidate_symbols:
        worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) mirror_publication:
        worth_relational_storage::internal_port::storage::mirror_plan::PreparedMirrorPublication,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) prepared_history:
        crate::mvcc::publication::authority::PreparedRelationalPublication,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) validated_lineage_events:
        worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::state::ValidatedLineageEventBatch,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) lineage_nodes:
        Vec<contracts::lineage::data::graph::LineageNode>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) relation_lineages: Vec<(
        contracts::identity::data::record_ids::RelationId,
        contracts::identity::data::primitives::LineageId,
    )>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) artifacts:
        crate::publication::authority::artifacts::PublicationArtifacts,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) changed_records:
        Vec<contracts::transactions::data::primitives::RecordRef>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) version_id: VersionId,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) previous_branch_head_version:
        Option<VersionId>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) commit_id: CommitId,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) commit_reference:
        RelationalCommitReceipt,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) branch_id: BranchId,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) deferred_diagnostic_artifacts:
        Vec<contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) canonical_publication_route:
        crate::runtime::state::subsystems::history::canonical_publication_routes::PreparedCanonicalPublicationRoute,
}

pub(in crate::internal_port::authority::commit::pipeline::publication_execution) struct PreparedPublicationCompletion
{
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) mirror_publication:
        worth_relational_storage::internal_port::storage::mirror_plan::PreparedMirrorPublication,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) prepared_history:
        crate::mvcc::publication::authority::PreparedRelationalPublicationAccelerators,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) validated_lineage_events:
        worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::state::ValidatedLineageEventBatch,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) lineage_nodes:
        Vec<contracts::lineage::data::graph::LineageNode>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) relation_lineages: Vec<(
        contracts::identity::data::record_ids::RelationId,
        contracts::identity::data::primitives::LineageId,
    )>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) artifacts:
        crate::publication::authority::artifacts::PublicationArtifacts,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) changed_records:
        Vec<contracts::transactions::data::primitives::RecordRef>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) version_id: VersionId,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) previous_branch_head_version:
        Option<VersionId>,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) commit_id: CommitId,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) commit_reference:
        RelationalCommitReceipt,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) branch_id: BranchId,
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) deferred_diagnostic_artifacts:
        Vec<contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact>,
}

impl PreparedPublicationPhase {
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) fn caller_association(
        &self,
    ) -> Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>{
        self.prepared_history.caller_association().cloned()
    }
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) fn canonical_envelope(
        &self,
    ) -> &Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope>{
        self.prepared_history.canonical_envelope()
    }
    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) fn descriptive_touches(
        &self,
    ) -> &worth_relational_branch::internal_port::history::data::canonical_commit_envelope::descriptive_touches::RelationalDescriptiveTouchGraph{
        self.prepared_history.descriptive_touches()
    }

    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) fn changed_record_count(
        &self,
    ) -> usize {
        self.changed_records.len()
    }

    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) fn reservation_count(
        &self,
    ) -> usize {
        self.record_allocations.reservation_count()
    }

    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) fn prepared_root(
        &self,
    ) -> &Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot> {
        self.prepared_history.root()
    }

    pub(in crate::internal_port::authority::commit::pipeline::publication_execution) fn materialization_counts(
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
    working_state: &mut worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    envelope: &mut worth_relational_branch::internal_port::history::data::canonical_commit_envelope::draft::CanonicalCommitEnvelopeDraft,
    changed_records: &[contracts::transactions::data::primitives::RecordRef],
    created_entities: &contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    created_relations: &contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
    naming_plan: Option<&contracts::lineage::data::postures::CommitNamingPlan>,
) -> Result<
    (
        Vec<contracts::lineage::data::graph::LineageNode>,
        Vec<(
            contracts::identity::data::record_ids::RelationId,
            contracts::identity::data::primitives::LineageId,
        )>,
    ),
    TransactionCommitError,
> {
    let graph_selection = match &envelope.authority_kind {
        worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitAuthorityKind::OwnedGraphSelection(selection) => {
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
