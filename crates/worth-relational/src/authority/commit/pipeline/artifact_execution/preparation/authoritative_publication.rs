//! Assemble the canonical authority and lineage for the already checked state.
use super::{
    CanonicalAuthoritativePatch, MergedCommitPlan, PublishedMergeExecutionAuthority, RecordRef,
    RelationalCommitReceipt, SchemaContinuityPlan, TransactionCommitError,
};
use crate::authority::commit::phases::publication::canonical_commit_envelope;
use crate::publication::{bundle::PublicationStage, data::PublicationError};

pub(super) struct PublicationAuthorityInput<'a> {
    pub(super) working_state: &'a mut crate::runtime::WorkingState,
    pub(super) patch: &'a CanonicalAuthoritativePatch,
    pub(super) commit_reference: &'a RelationalCommitReceipt,
    pub(super) branch_id: &'a crate::history::data::BranchId,
    pub(super) version_id: crate::identity::data::VersionId,
    pub(super) merge_parent_branches: &'a [crate::history::data::BranchId],
    pub(super) merge_base_commits: &'a [crate::history::data::CommitId],
    pub(super) merged_plan: &'a MergedCommitPlan,
    pub(super) record_allocations: &'a [crate::history::data::CanonicalRecordAllocation],
    pub(super) strategy_artifacts:
        Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
    pub(super) merge_execution_authority: Option<PublishedMergeExecutionAuthority>,
    pub(super) schema_continuity: &'a SchemaContinuityPlan,
    pub(super) changed_records: &'a [RecordRef],
    pub(super) diagnostics_summary: &'a crate::diagnostics::data::RelationalDiagnosticArtifact,
    pub(super) descriptive_touches: crate::history::data::RelationalDescriptiveTouchGraph,
    pub(super) authored_index_writes: crate::history::data::CanonicalAuthoredIndexWrites,
}

pub(super) struct PreparedPublicationAuthority {
    pub(super) artifacts: crate::storage::overlay::PublicationArtifacts,
    pub(super) canonical_commit_envelope: crate::history::data::CanonicalCommitEnvelope,
    pub(super) lineage_event_count: usize,
    pub(super) lineage_nodes: Vec<crate::lineage::data::LineageNode>,
}

pub(super) fn prepare_authoritative_publication(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    input: PublicationAuthorityInput<'_>,
) -> Result<PreparedPublicationAuthority, TransactionCommitError> {
    let artifacts = runtime
        .publication_preparation_authority()
        .assemble_publication_bundle(
            input.commit_reference.clone(),
            input.version_id,
            input.patch.clone(),
            input.diagnostics_summary.clone(),
            input.schema_continuity.target_schema_authority().clone(),
        )
        .map_err(TransactionCommitError::publication_failed)?;
    let prepared_lineage = runtime
        .lineage_preparation_authority()
        .ensure_lineage_for_commit(
            input.working_state,
            input.commit_reference,
            &input.merged_plan.merged_intents,
            input.changed_records,
        )
        .map_err(|detail| {
            TransactionCommitError::publication(PublicationError::new(
                PublicationStage::BundleAssembly,
                detail,
            ))
        })?;
    let lineage_event_count = prepared_lineage
        .artifact()
        .event_batch()
        .counters()
        .event_batch_width;
    let (lineage_artifact, lineage_nodes) = prepared_lineage.into_parts();
    let mut canonical_commit_envelope = canonical_commit_envelope(
        runtime,
        input.working_state.content.clone(),
        input.commit_reference,
        input.branch_id,
        crate::history::data::CanonicalCommitAuthorityKind::VersionedTransaction,
        input.strategy_artifacts,
        input.merge_execution_authority,
        input.merge_parent_branches,
        input.merge_base_commits,
        input.merged_plan,
        input.patch.clone(),
        input.descriptive_touches,
        input.diagnostics_summary.clone(),
        lineage_artifact,
        crate::indexes::data::DerivedIndexArtifacts::default(),
        input.schema_continuity,
    )?;
    canonical_commit_envelope.authored_index_writes = input.authored_index_writes;
    canonical_commit_envelope.install_record_allocations(input.record_allocations.to_vec());
    Ok(PreparedPublicationAuthority {
        artifacts,
        canonical_commit_envelope,
        lineage_event_count,
        lineage_nodes,
    })
}
