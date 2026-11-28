//! Assemble canonical authority for the checked complete semantic plan.
use super::CommitPhaseBacking;
use super::SchemaContinuityPlan;
use crate::authority::commit::phases::publication::{
    canonical_commit_envelope, CanonicalEnvelopeInputs,
};
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::publication::patch::data::authoritative_patch_envelope::CanonicalAuthoritativePatch;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use contracts::transactions::data::primitives::RecordRef;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::authority::commit::semantic_plan::meaning::RelationalSemanticPlan;

pub struct ArtifactAuthority<'a> {
    pub publication: crate::publication::authority::PublicationPreparationAuthority<'a>,
    pub lineage: crate::authority::commit::lineage_preparation::LineagePreparationAuthority<'a>,
    pub envelope: CanonicalEnvelopeInputs<'a>,
}

pub(super) struct PublicationAuthorityInput<'a> {
    pub(super) index_definition_delta: branch::history::data::canonical_commit_envelope::index_catalog::CanonicalIndexDefinitionDelta,
    pub(super) defer_lineage: bool,
    pub(super) aggregate_partitions: &'a worth_relational_mutation::internal_port::aggregate_identity::preparation::TouchedAggregatePartitions,
    pub(super) working_state: &'a mut worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    pub(super) content: branch::content::publication::ContentPublication,
    pub(super) patch: &'a CanonicalAuthoritativePatch,
    pub(super) commit_reference: &'a RelationalCommitReceipt,
    pub(super) branch_id: &'a contracts::history::data::coordinates::BranchId,
    pub(super) version_id: contracts::identity::data::primitives::VersionId,
    pub(super) merged_plan: RelationalSemanticPlan<'a>,
    pub(super) created_entities: &'a contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    pub(super) created_relations: &'a contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
    pub(super) naming_plan: Option<&'a contracts::lineage::data::postures::CommitNamingPlan>,
    pub(super) record_allocations: &'a worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::pending_allocations::PendingRecordAllocations,
    pub(super) schema_continuity: &'a SchemaContinuityPlan,
    pub(super) changed_records: &'a [RecordRef],
    pub(super) diagnostics_summary: &'a contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact,
    pub(super) descriptive_touches: branch::history::data::canonical_commit_envelope::descriptive_touches::RelationalDescriptiveTouchGraph,
    pub(super) authored_index_writes: branch::history::data::canonical_commit_envelope::authored_index_writes::CanonicalAuthoredIndexWrites,
}

pub(super) struct PreparedPublicationAuthority {
    pub(super) artifacts: crate::publication::authority::artifacts::PublicationArtifacts,
    pub(super) canonical_commit_envelope:
        branch::history::data::canonical_commit_envelope::draft::CanonicalCommitEnvelopeDraft,
    pub(super) lineage_event_count: usize,
    pub(super) lineage_nodes: Vec<contracts::lineage::data::graph::LineageNode>,
    pub(super) relation_lineages: Vec<(
        contracts::identity::data::record_ids::RelationId,
        contracts::identity::data::primitives::LineageId,
    )>,
}

pub(super) fn prepare_authoritative_publication(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    authority: ArtifactAuthority<'_>,
    input: PublicationAuthorityInput<'_>,
) -> Result<CommitPhaseBacking<PreparedPublicationAuthority>, TransactionCommitError> {
    let result = {
        let artifacts = authority
            .publication
            .assemble_publication_bundle(
                input.commit_reference.clone(),
                input.version_id,
                input.patch.clone(),
                input.diagnostics_summary.clone(),
                input.schema_continuity.target_schema_authority().clone(),
            )
            .map_err(TransactionCommitError::publication_failed)?;
        let prepared_lineage = if input.defer_lineage {
            use contracts::lineage::data::{
                FinalizedLineageEventBatch, LineageDecisionLog, LineageFinalizationArtifact,
                PreparedLineageFinalization,
            };
            PreparedLineageFinalization::new(
                LineageFinalizationArtifact::new(
                    input.branch_id.clone(),
                    FinalizedLineageEventBatch::new(Vec::new()),
                    LineageDecisionLog::new(Vec::new()),
                ),
                Vec::new(),
            )
        } else {
            authority.lineage.ensure_lineage_for_commit(
                execution,
                input.working_state,
                input.commit_reference,
                &input.merged_plan.commands().merged_intents,
                input.changed_records,
                input.created_entities,
                input.created_relations,
                input.naming_plan,
                input.aggregate_partitions,
                None,
            )?
        };
        let lineage_event_count = prepared_lineage
            .artifact()
            .event_batch()
            .counters()
            .event_batch_width;
        let (lineage_artifact, lineage_nodes, relation_lineages) = prepared_lineage.into_parts();
        let mut canonical_commit_envelope = canonical_commit_envelope(
            authority.envelope,
            input.content,
            input.index_definition_delta,
            input.commit_reference,
            input.branch_id,
            input.merged_plan.canonical_authority_kind(),
            input.merged_plan.commands(),
            input.patch.clone(),
            input.descriptive_touches,
            input.diagnostics_summary.clone(),
            lineage_artifact,
            branch::indexes::data::DerivedIndexArtifacts::default(),
            input.schema_continuity,
        )?;
        (*canonical_commit_envelope.owner_authored_index_writes_mut()) =
            input.authored_index_writes;
        canonical_commit_envelope
            .install_record_allocations(input.record_allocations.canonical().to_vec());
        canonical_commit_envelope
            .install_cancelled_record_allocations(input.record_allocations.cancelled().to_vec());
        PreparedPublicationAuthority {
            artifacts,
            canonical_commit_envelope,
            lineage_event_count,
            lineage_nodes,
            relation_lineages,
        }
    };
    CommitPhaseBacking::new(execution, result)
}
