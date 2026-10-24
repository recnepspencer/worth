use serde::ser::SerializeStruct;
use serde::Serialize;

use super::CanonicalCommitEnvelope;
use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::indexes::data::DerivedIndexArtifacts;

/// Canonical-commit-artifact projection of one envelope: every authoritative
/// field plus the symbol dictionary and codec-retirement headers readmission
/// requires. The diagnostics and derived-index lanes have their own lifecycle,
/// so they are emitted as explicit empty values: diagnostics keep their
/// scope, kind and determinism with no entries, and derived indexes are the
/// empty rebuildable cache. The destructure is exhaustive so a new envelope
/// field must be classified here before it compiles.
///
/// The emitted map decodes through `CanonicalEnvelopeWire`, the same codec
/// every other current envelope input uses.
pub(crate) struct AuthoritativeArtifactEnvelopeRef<'a> {
    envelope: &'a CanonicalCommitEnvelope,
}

impl<'a> AuthoritativeArtifactEnvelopeRef<'a> {
    pub(crate) fn new(envelope: &'a CanonicalCommitEnvelope) -> Self {
        Self { envelope }
    }
}

impl Serialize for AuthoritativeArtifactEnvelopeRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let CanonicalCommitEnvelope {
            required_output_codec_retirements,
            required_output_origin,
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
            record_allocations,
            patch,
            descriptive_touches,
            diagnostics_summary,
            lineage,
            derived_index_artifacts: _,
            schema_transition,
            schema_continuation_descriptor,
            schema_reconciliation_descriptor,
            descriptor_semantics_version,
        } = self.envelope;
        let omitted_diagnostics = RelationalDiagnosticArtifact {
            scope: diagnostics_summary.scope,
            kind: diagnostics_summary.kind,
            determinism: diagnostics_summary.determinism,
            entries: Vec::new(),
        };
        let mut fields = serializer.serialize_struct("CanonicalCommitEnvelope", 25)?;
        fields.serialize_field(
            "required_output_codec_retirements",
            required_output_codec_retirements,
        )?;
        fields.serialize_field("required_output_origin", required_output_origin)?;
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
        fields.serialize_field("record_allocations", record_allocations)?;
        fields.serialize_field("patch", patch)?;
        fields.serialize_field("descriptive_touches", descriptive_touches)?;
        fields.serialize_field("diagnostics_summary", &omitted_diagnostics)?;
        fields.serialize_field("lineage", lineage)?;
        fields.serialize_field("derived_index_artifacts", &DerivedIndexArtifacts::default())?;
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
}
