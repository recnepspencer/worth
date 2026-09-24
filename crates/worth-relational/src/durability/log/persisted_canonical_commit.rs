use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;

/// Raw native-file vocabulary. Decoding this type never grants current
/// canonical authority; callers must pass it through owner readmission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PersistedCanonicalCommit {
    position: PatchStreamPosition,
    canonical: CanonicalCommitEnvelope,
}

/// Borrowed checkpoint encoding of a canonical envelope. The derived index
/// cache is omitted from the wire without copying its authoritative body.
#[derive(Serialize)]
pub(super) struct PersistedCheckpointCommitRef<'a> {
    position: PatchStreamPosition,
    canonical: CheckpointCanonicalEnvelopeRef<'a>,
}

struct CheckpointCanonicalEnvelopeRef<'a>(&'a CanonicalCommitEnvelope);

impl<'a> PersistedCheckpointCommitRef<'a> {
    pub(super) fn from_positioned(commit: &'a PositionedCanonicalCommit) -> Self {
        Self {
            position: commit.position(),
            canonical: CheckpointCanonicalEnvelopeRef(commit.envelope()),
        }
    }
}

impl Serialize for CheckpointCanonicalEnvelopeRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let envelope = self.0;
        let mut fields = serializer.serialize_struct("CanonicalCommitEnvelope", 20)?;
        fields.serialize_field("commit", &envelope.commit)?;
        fields.serialize_field("branch_context", &envelope.branch_context)?;
        fields.serialize_field("branch_cell_checkpoint", &envelope.branch_cell_checkpoint)?;
        fields.serialize_field("authority_kind", &envelope.authority_kind)?;
        fields.serialize_field("strategy_artifacts", &envelope.strategy_artifacts)?;
        fields.serialize_field(
            "merge_execution_authority",
            &envelope.merge_execution_authority,
        )?;
        fields.serialize_field("merge_parent_branches", &envelope.merge_parent_branches)?;
        fields.serialize_field("merge_base_commits", &envelope.merge_base_commits)?;
        fields.serialize_field("schema_version", &envelope.schema_version)?;
        fields.serialize_field("schema_authority", &envelope.schema_authority)?;
        fields.serialize_field("merged_plan", &envelope.merged_plan)?;
        fields.serialize_field("record_allocations", envelope.record_allocations())?;
        fields.serialize_field("patch", &envelope.patch)?;
        fields.serialize_field("diagnostics_summary", &envelope.diagnostics_summary)?;
        fields.serialize_field("lineage", envelope.published_lineage())?;
        fields.serialize_field(
            "derived_index_artifacts",
            &crate::indexes::data::DerivedIndexArtifacts::default(),
        )?;
        fields.serialize_field("schema_transition", &envelope.schema_transition)?;
        fields.serialize_field(
            "schema_continuation_descriptor",
            &envelope.schema_continuation_descriptor,
        )?;
        fields.serialize_field(
            "schema_reconciliation_descriptor",
            &envelope.schema_reconciliation_descriptor,
        )?;
        fields.serialize_field(
            "descriptor_semantics_version",
            &envelope.descriptor_semantics_version,
        )?;
        fields.end()
    }
}

impl PersistedCanonicalCommit {
    pub(crate) fn from_positioned(commit: &PositionedCanonicalCommit) -> Self {
        Self {
            position: commit.position(),
            canonical: commit.envelope().clone(),
        }
    }

    /// Checkpoint envelopes carry canonical history, not rebuildable index
    /// caches. The versioned checkpoint artifact carries retained generations.
    #[cfg(test)]
    pub(crate) fn from_checkpoint_positioned(commit: &PositionedCanonicalCommit) -> Self {
        let mut persisted = Self::from_positioned(commit);
        persisted.canonical.derived_index_artifacts = Default::default();
        persisted
    }

    pub(crate) fn into_receipt(self) -> crate::history::data::RelationalCommitReceipt {
        self.canonical.commit
    }

    #[cfg(test)]
    pub(crate) fn envelope_mut_for_test(&mut self) -> &mut CanonicalCommitEnvelope {
        &mut self.canonical
    }

    pub(crate) fn readmit(
        self,
    ) -> Result<crate::durability::migration::ReadmittedCanonicalCommit, String> {
        crate::durability::migration::ReadmittedCanonicalCommit::readmit_current(
            self.position,
            self.canonical,
        )
    }
}
