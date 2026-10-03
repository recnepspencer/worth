use crate::authority::commit::phases::schema_continuity::validate_schema_continuity_publication;
use crate::capabilities::{DiagnosticArtifactSink, DurabilityWrite, PublicationPolicySource};
use crate::diagnostics::data::{
    DiagnosticCode, DiagnosticsScope, RelationalDiagnosticFields, RelationalDiagnosticValue,
};
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::history::data::{CanonicalCommitAuthorityKind, CanonicalCommitEnvelope};
use crate::indexes::data::DerivedIndexArtifacts;
use crate::lineage::data::LineageFinalizationArtifact;
use crate::publication::bundle::PublicationStage;
use crate::publication::data::PublicationError;
use crate::publication::patch::data::CanonicalAuthoritativePatch;
use crate::transactions::data::{
    MergedCommitPlan, PublishedMergeExecutionAuthority, RecordRef, TransactionCommitError,
};

pub(crate) fn enforce_patch_budget(
    runtime: &(impl DiagnosticArtifactSink + PublicationPolicySource),
    patch: &CanonicalAuthoritativePatch,
) -> Result<(), TransactionCommitError> {
    let max_patch_records_per_commit = runtime.max_patch_records_per_commit();
    if patch.authoritative_record_patches.len() > max_patch_records_per_commit {
        runtime.emit_failure_diagnostic(
            DiagnosticsScope::PatchPublication,
            DiagnosticCode::DiagnosticsPublicationFailure,
            "patch record budget exceeded",
            patch_record_budget_exceeded_fields(
                patch.authoritative_record_patches.len(),
                max_patch_records_per_commit,
            ),
        );
        return Err(TransactionCommitError::publication(PublicationError::new(
            PublicationStage::BundleAssembly,
            "patch record budget exceeded",
        )));
    }
    Ok(())
}

pub(crate) fn canonical_commit_envelope(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    commit_reference: &RelationalCommitReceipt,
    branch_id: &BranchId,
    authority_kind: CanonicalCommitAuthorityKind,
    strategy_artifacts: Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
    merge_execution_authority: Option<PublishedMergeExecutionAuthority>,
    merge_parent_branches: &[BranchId],
    merge_base_commits: &[CommitId],
    merged_plan: &MergedCommitPlan,
    patch: CanonicalAuthoritativePatch,
    descriptive_touches: crate::history::data::RelationalDescriptiveTouchGraph,
    diagnostics_summary: crate::diagnostics::data::RelationalDiagnosticArtifact,
    lineage_artifact: LineageFinalizationArtifact,
    derived_index_artifacts: DerivedIndexArtifacts,
    schema_continuity: &crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan,
) -> Result<CanonicalCommitEnvelope, TransactionCommitError> {
    let published_lineage = lineage_artifact.publish();
    runtime
        .performance_access()
        .count_lineage_publication_artifact(
            published_lineage.lineage_events().len(),
            published_lineage.lineage_decision_log().len(),
        );
    if published_lineage.branch_id() != branch_id || lineage_artifact.branch_id() != branch_id {
        runtime.emit_failure_diagnostic(
            DiagnosticsScope::Lineage,
            DiagnosticCode::DiagnosticsPublicationFailure,
            "lineage artifact branch scope did not match publication branch",
            lineage_branch_scope_mismatch_fields(branch_id, lineage_artifact.branch_id()),
        );
        return Err(TransactionCommitError::publication(PublicationError::new(
            PublicationStage::BundleAssembly,
            "lineage artifact branch scope mismatch",
        )));
    }
    let mut envelope = CanonicalCommitEnvelope::new(
        commit_reference.clone(),
        published_lineage.branch_id().clone(),
        authority_kind,
        strategy_artifacts,
        merge_execution_authority,
        merge_parent_branches.to_vec(),
        merge_base_commits.to_vec(),
        schema_continuity.target_schema_version(),
        schema_continuity.target_schema_authority().clone(),
        merged_plan.clone(),
        patch,
        descriptive_touches,
        diagnostics_summary,
        published_lineage,
        derived_index_artifacts,
        schema_continuity.schema_transition.clone(),
        schema_continuity.schema_continuation_descriptor.clone(),
        schema_continuity.schema_reconciliation_descriptor.clone(),
        schema_continuity.descriptor_semantics_version,
    );
    envelope.branch_cell_checkpoint = runtime
        .history
        .branch_cell(branch_id)
        .map(|cell| cell.checkpoint());
    validate_schema_continuity_publication(runtime, branch_id, schema_continuity, &envelope)?;
    Ok(envelope)
}

pub(crate) fn append_durable_commit(
    runtime: &(impl DiagnosticArtifactSink + DurabilityWrite),
    append_authority: crate::durability::authority::DurableAppendAuthority,
    positioned_commit: &crate::history::data::PositionedCanonicalCommit,
) -> Result<(), TransactionCommitError> {
    let commit_id = append_authority.commit_id();
    let branch_id = append_authority.branch_id().clone();
    if let Err(error) = runtime.append_durable_envelope(append_authority, positioned_commit) {
        runtime.emit_failure_diagnostic(
            DiagnosticsScope::History,
            DiagnosticCode::DurableAppendFailed,
            error.detail.clone(),
            durable_append_failure_fields(commit_id, &branch_id),
        );
        return Err(TransactionCommitError::publication(PublicationError::new(
            PublicationStage::DurableAppend,
            error.detail,
        )));
    }
    Ok(())
}

pub(crate) fn canonicalize_changed_records(records: &mut Vec<RecordRef>) {
    records.sort_unstable();
    records.dedup();
}

fn patch_record_budget_exceeded_fields(
    patch_records: usize,
    max_patch_records_per_commit: usize,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "patch_records",
            RelationalDiagnosticValue::unsigned(patch_records),
        ),
        (
            "max_patch_records_per_commit",
            RelationalDiagnosticValue::unsigned(max_patch_records_per_commit),
        ),
    ])
    .into()
}

fn lineage_branch_scope_mismatch_fields(
    publication_branch: &BranchId,
    artifact_branch: &BranchId,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "publication_branch_id",
            RelationalDiagnosticValue::string(publication_branch.0.clone()),
        ),
        (
            "artifact_branch_id",
            RelationalDiagnosticValue::string(artifact_branch.0.clone()),
        ),
    ])
    .into()
}

fn durable_append_failure_fields(
    commit_id: CommitId,
    branch_id: &BranchId,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "commit_id",
            RelationalDiagnosticValue::Unsigned(commit_id.0),
        ),
        (
            "branch_id",
            RelationalDiagnosticValue::string(branch_id.0.clone()),
        ),
    ])
    .into()
}
