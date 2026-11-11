//! One exhaustive classification of complete envelope fields for each placement.
use super::{CanonicalCommitEnvelope, CanonicalCommitEnvelopeBody};
use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::indexes::data::DerivedIndexArtifacts;
use serde::ser::SerializeStruct;

#[derive(Clone, Copy)]
pub(crate) enum EnvelopeProjection {
    Full,
    Authoritative,
    #[cfg(test)]
    Checkpoint,
}
pub(super) fn serialize<S: serde::Serializer>(
    envelope: &CanonicalCommitEnvelope,
    serializer: S,
    projection: EnvelopeProjection,
) -> Result<S::Ok, S::Error> {
    let CanonicalCommitEnvelopeBody {
        content,
        producer_cascade_retirement,
        authored_cascade_retirement,
        authored_cascade_restoration,
        index_definition_delta,
        caller_association,
        authored_index_writes,
        required_output_codec_retirements,
        required_output_origin,
        coupled_authored_publication,
        historical_output_selection,
        required_output_materialization,
        commit,
        branch_context,
        branch_cell_checkpoint,
        authority_kind,
        strategy_artifacts,
        merge_execution_authority,
        merge_parent_branches,
        merge_base_commits,
        schema_version,
        schema_authority,
        merged_plan,
        symbol_texts,
        revision_symbol_references,
        record_allocations,
        patch,
        descriptive_touches,
        diagnostics_summary,
        lineage,
        derived_index_artifacts,
        schema_transition,
        schema_continuation_descriptor,
        schema_reconciliation_descriptor,
        descriptor_semantics_version,
    } = &envelope.body;
    let omitted_diagnostics = RelationalDiagnosticArtifact {
        scope: diagnostics_summary.scope,
        kind: diagnostics_summary.kind,
        determinism: diagnostics_summary.determinism,
        entries: Vec::new(),
    };
    let omitted_indexes = DerivedIndexArtifacts::default();
    let field_count = 32
        + usize::from(!authored_index_writes.is_unavailable())
        + usize::from(producer_cascade_retirement.is_some())
        + usize::from(coupled_authored_publication.is_some())
        + usize::from(historical_output_selection.is_some());
    let mut fields = serializer.serialize_struct("CanonicalCommitEnvelope", field_count)?;
    fields.serialize_field("caller_association", caller_association)?;
    fields.serialize_field("index_definition_delta", index_definition_delta)?;
    fields.serialize_field("semantic_result", envelope.semantic_result())?;
    if !authored_index_writes.is_unavailable() {
        fields.serialize_field("authored_index_writes", authored_index_writes)?;
    }
    fields.serialize_field(
        "required_output_codec_retirements",
        required_output_codec_retirements,
    )?;
    fields.serialize_field("required_output_origin", required_output_origin)?;
    if let Some(group) = coupled_authored_publication {
        fields.serialize_field("coupled_authored_publication", group)?;
    }
    if let Some(selection) = historical_output_selection {
        fields.serialize_field("historical_output_selection", selection)?;
    }
    if producer_cascade_retirement.is_some() {
        fields.serialize_field("producer_cascade_retirement", producer_cascade_retirement)?;
    }
    fields.serialize_field("authored_cascade_retirement", authored_cascade_retirement)?;
    fields.serialize_field("authored_cascade_restoration", authored_cascade_restoration)?;
    fields.serialize_field("content", content)?;
    fields.serialize_field(
        "required_output_materialization",
        required_output_materialization,
    )?;
    fields.serialize_field("commit", commit)?;
    fields.serialize_field("branch_context", branch_context)?;
    fields.serialize_field("branch_cell_checkpoint", branch_cell_checkpoint)?;
    fields.serialize_field("authority_kind", authority_kind)?;
    fields.serialize_field("strategy_artifacts", strategy_artifacts)?;
    fields.serialize_field("merge_execution_authority", merge_execution_authority)?;
    fields.serialize_field("merge_parent_branches", merge_parent_branches)?;
    fields.serialize_field("merge_base_commits", merge_base_commits)?;
    fields.serialize_field("schema_version", schema_version)?;
    fields.serialize_field("schema_authority", schema_authority)?;
    fields.serialize_field("merged_plan", merged_plan)?;
    fields.serialize_field("symbol_texts", symbol_texts)?;
    fields.serialize_field("revision_symbol_references", revision_symbol_references)?;
    fields.serialize_field("record_allocations", record_allocations)?;
    fields.serialize_field("patch", patch)?;
    fields.serialize_field("descriptive_touches", descriptive_touches)?;
    fields.serialize_field(
        "diagnostics_summary",
        if matches!(projection, EnvelopeProjection::Authoritative) {
            &omitted_diagnostics
        } else {
            diagnostics_summary
        },
    )?;
    fields.serialize_field("lineage", lineage)?;
    fields.serialize_field(
        "derived_index_artifacts",
        if matches!(projection, EnvelopeProjection::Full) {
            derived_index_artifacts
        } else {
            &omitted_indexes
        },
    )?;
    fields.serialize_field("schema_transition", schema_transition)?;
    fields.serialize_field(
        "schema_continuation_descriptor",
        schema_continuation_descriptor,
    )?;
    fields.serialize_field(
        "schema_reconciliation_descriptor",
        schema_reconciliation_descriptor,
    )?;
    fields.serialize_field("descriptor_semantics_version", descriptor_semantics_version)?;
    fields.end()
}
