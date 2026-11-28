use crate::authority::commit::phases::schema_continuity::validate_schema_continuity_publication;
use crate::publication::authority::diagnostic_emission::PreparationDiagnostics;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::history::data::coordinates::BranchId;
use contracts::lineage::data::artifacts::finalization_artifact::LineageFinalizationArtifact;
use contracts::publication::bundle::publication_status::PublicationStage;
use contracts::publication::data::publication_error::PublicationError;
use contracts::publication::patch::data::authoritative_patch_envelope::CanonicalAuthoritativePatch;
use contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{
    CanonicalCommitAuthorityKind, CanonicalCommitEnvelopeBodyFieldsPort as _,
};
use worth_relational_branch::internal_port::indexes::data::DerivedIndexArtifacts;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

pub(crate) fn enforce_patch_budget(
    max_patch_records_per_commit: Option<usize>,
    patch: &CanonicalAuthoritativePatch,
) -> Result<(), TransactionCommitError> {
    let Some(max_patch_records_per_commit) = max_patch_records_per_commit else {
        return Ok(());
    };
    if patch.authoritative_record_patches.len() > max_patch_records_per_commit {
        return Err(TransactionCommitError::publication(PublicationError::new(
            PublicationStage::BundleAssembly,
            "patch record budget exceeded",
        )));
    }
    Ok(())
}

pub struct CanonicalEnvelopeInputs<'a> {
    pub history: &'a RelationalPreparationHistory,
    pub diagnostics: PreparationDiagnostics<'a>,
    pub symbols: &'a RuntimeSymbolTable,
    pub symbol_references: &'a std::collections::BTreeSet<contracts::symbols::data::Symbol>,
}

pub(crate) fn canonical_commit_envelope<'a>(
    inputs: impl Into<CanonicalEnvelopeInputs<'a>>,
    content: worth_relational_branch::internal_port::content::publication::ContentPublication,
    index_definition_delta: worth_relational_branch::internal_port::history::data::canonical_commit_envelope::index_catalog::CanonicalIndexDefinitionDelta,
    commit_reference: &RelationalCommitReceipt,
    branch_id: &BranchId,
    authority_kind: CanonicalCommitAuthorityKind,
    merged_plan: &MergedCommitPlan,
    patch: CanonicalAuthoritativePatch,
    descriptive_touches: worth_relational_branch::internal_port::history::data::canonical_commit_envelope::descriptive_touches::RelationalDescriptiveTouchGraph,
    diagnostics_summary: contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact,
    lineage_artifact: LineageFinalizationArtifact,
    derived_index_artifacts: DerivedIndexArtifacts,
    schema_continuity: &crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan,
) -> Result<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::draft::CanonicalCommitEnvelopeDraft, TransactionCommitError>{
    let inputs = inputs.into();
    let published_lineage = lineage_artifact.publish();
    inputs.diagnostics.instrumentation.count(|counters| {
        counters.lineage_publication_event_width += published_lineage.lineage_events().len();
        counters.lineage_publication_decision_width +=
            published_lineage.lineage_decision_log().len();
    });
    if published_lineage.branch_id() != branch_id || lineage_artifact.branch_id() != branch_id {
        return Err(TransactionCommitError::publication(PublicationError::new(
            PublicationStage::BundleAssembly,
            "lineage artifact branch scope mismatch",
        )));
    }
    let mut envelope = worth_relational_branch::internal_port::history::data::canonical_commit_envelope::draft::CanonicalCommitEnvelopeDraft::new(
        content,
        index_definition_delta,
        commit_reference.clone(),
        published_lineage.branch_id().clone(),
        authority_kind,
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
    (*envelope.owner_revision_symbol_references_mut()) =
        inputs.symbol_references.iter().copied().collect();
    (*envelope.owner_symbol_texts_mut()) = inputs
        .symbols
        .with_read(|symbols| {
            symbols.commit_texts(
                merged_plan,
                &envelope.patch,
                inputs.symbol_references.iter().copied(),
            )
        })
        .map_err(|detail| {
            TransactionCommitError::publication(PublicationError::new(
                PublicationStage::BundleAssembly,
                detail,
            ))
        })?;
    // The envelope records process-local identities in the canonical runtime
    // frame, so its bytes do not depend on which process produced it (D11).
    envelope.merged_plan.transaction_id =
        contracts::branch::reference_checkpoint_data::CANONICAL_TRANSACTION_ID;
    (*envelope.owner_branch_cell_checkpoint_mut()) = inputs
        .history
        .branch_cell(branch_id)
        .map(|cell| cell.checkpoint().in_canonical_frame())
        .transpose()
        .map_err(|denial| {
            TransactionCommitError::publication(PublicationError::new(
                PublicationStage::BundleAssembly,
                format!("branch cell cannot be recorded in the canonical frame: {denial:?}"),
            ))
        })?;
    validate_schema_continuity_publication(
        inputs.diagnostics,
        branch_id,
        schema_continuity,
        &envelope,
    )?;
    Ok(envelope)
}

pub(crate) fn canonicalize_changed_records(records: &mut Vec<RecordRef>) {
    records.sort_unstable();
    records.dedup();
}
