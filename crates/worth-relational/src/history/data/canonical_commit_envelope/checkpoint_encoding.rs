use serde::ser::SerializeStruct;
use serde::Serialize;

use super::CanonicalCommitEnvelope;

/// Checkpoint wire projection of canonical authority. New envelope fields must
/// be classified here because the destructure is exhaustive; only rebuildable
/// index caches are replaced with an empty payload.
pub(crate) struct CheckpointCanonicalEnvelopeRef<'a>(&'a CanonicalCommitEnvelope);

impl<'a> CheckpointCanonicalEnvelopeRef<'a> {
    pub(crate) fn new(envelope: &'a CanonicalCommitEnvelope) -> Self {
        Self(envelope)
    }
}

impl Serialize for CheckpointCanonicalEnvelopeRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let CanonicalCommitEnvelope {
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
            record_allocations,
            patch,
            diagnostics_summary,
            lineage,
            derived_index_artifacts: _,
            schema_transition,
            schema_continuation_descriptor,
            schema_reconciliation_descriptor,
            descriptor_semantics_version,
        } = self.0;
        let mut fields = serializer.serialize_struct("CanonicalCommitEnvelope", 20)?;
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
        fields.serialize_field("record_allocations", record_allocations)?;
        fields.serialize_field("patch", patch)?;
        fields.serialize_field("diagnostics_summary", diagnostics_summary)?;
        fields.serialize_field("lineage", lineage)?;
        fields.serialize_field(
            "derived_index_artifacts",
            &crate::indexes::data::DerivedIndexArtifacts::default(),
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
}
