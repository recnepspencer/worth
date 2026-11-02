use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::indexes::data::DerivedIndexArtifacts;
use crate::lineage::data::{
    LineageArtifactCounters, LineageDecisionLogDigestBasis, LineageDecisionRecord,
    LineageDigestBasis, LineageEventBatchDigestBasis, LineageEventRecord, PublishedLineageArtifact,
};
use crate::publication::patch::data::{
    CanonicalAuthoritativePatch, PublishedAuthoritativeRecordPatch,
};
use crate::schema::data::{
    DescriptorSemanticsVersion, SchemaAuthoritySnapshot, SchemaContinuationDescriptor,
    SchemaReconciliationDescriptor, SchemaTransitionArtifact, SchemaVersionId,
};
use crate::transactions::data::{
    ExistingRecordTarget, MergedCommitPlan, MutationIntent, PublishedMergeExecutionAuthority,
    RecordRef,
};

mod authored_index_writes;
mod index_catalog;
pub(crate) use authored_index_writes::{CanonicalAuthoredIndexWrite, CanonicalAuthoredIndexWrites};
pub(crate) use index_catalog::CanonicalIndexDefinitionDelta;
mod live_managed_source_history;
pub(crate) use live_managed_source_history::LiveManagedSourceHistory;
mod authoritative_artifact_encoding;
mod authoritative_payload;
mod control_allocation;
#[cfg(test)]
mod embedded_checkpoint;
mod payload_digest;
mod shared;
pub(crate) use authoritative_artifact_encoding::AuthoritativeArtifactEnvelopeRef;
pub(crate) use control_allocation::CanonicalEnvelopeControlAllocation;
#[cfg(test)]
pub(crate) use embedded_checkpoint::EmbeddedCheckpointEnvelopeRef;
pub use shared::SharedCanonicalCommitEnvelope;
#[path = "canonical_commit_envelope/descriptive_touches.rs"]
mod descriptive_touches;
mod required_output_codec_retirement;
mod required_output_materialization;
mod required_output_origin;
pub use descriptive_touches::{
    RelationalDescriptiveTouch, RelationalDescriptiveTouchGraph,
    RelationalDescriptiveTouchPrecision, RelationalTouchAdjacencyDirection,
};
pub(crate) use required_output_codec_retirement::{
    CanonicalCodecRetirementKind, CanonicalRequiredOutputCodecRetirement,
};
pub(crate) use required_output_materialization::{
    CanonicalRequiredOutputMaterialization, CanonicalRequiredOutputMaterializationRecord,
};
pub(crate) use required_output_origin::{
    CanonicalOutputMember, CanonicalOutputPublication, CanonicalRequiredOutputOrigin,
    CanonicalRequiredOutputRelation,
};

mod draft;
mod readmission_memory;
pub(crate) use readmission_memory::{
    CanonicalEnvelopeReadmissionMemory, CodecDeclarationsReadmissionMemory,
    StrategyReadmissionMemory,
};
mod wire_projection;
pub(crate) use draft::CanonicalCommitEnvelopeDraft;
pub(crate) use wire_projection::EnvelopeProjection;

/// A complete immutable envelope. Its semantic image exists before it can be
/// serialized, shared or registered for publication.
///
/// ```compile_fail,E0594
/// use worth_relational::facade::replay::CanonicalCommitEnvelope;
/// fn cannot_replace_receipt(envelope: &mut CanonicalCommitEnvelope) {
///     envelope.commit.version_id.0 += 1;
/// }
/// ```
///
/// The shared unsealed body has no encoder without its complete semantic image:
/// ```compile_fail,E0277
/// use worth_relational::facade::replay::CanonicalCommitEnvelope;
/// fn cannot_encode_an_incomplete_body(envelope: &CanonicalCommitEnvelope) {
///     let _ = rmp_serde::to_vec_named(&**envelope);
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct CanonicalCommitEnvelope {
    #[serde(skip)]
    pub(crate) live_managed_source_history: LiveManagedSourceHistory,
    #[serde(flatten)]
    body: CanonicalCommitEnvelopeBody,
    semantic_result: std::sync::Arc<crate::transactions::data::RelationalCommitSemanticResult>,
    // Intrinsic input and body admission outlive every decoded body field.
    #[serde(skip)]
    pub(crate) decoded_bytes: Option<worth_execution::ExecutionImmutableBytes>,
    #[serde(skip)]
    pub(crate) decoded_memory: Option<worth_execution::ExecutionMemoryReservation>,
    // LAST: real weak-position observers share only this fixed allocation bill.
    #[serde(skip)]
    pub(crate) control_allocation: Option<std::sync::Arc<CanonicalEnvelopeControlAllocation>>,
}

impl PartialEq for CanonicalCommitEnvelope {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body && self.semantic_result == other.semantic_result
    }
}
impl Eq for CanonicalCommitEnvelope {}

impl std::ops::Deref for CanonicalCommitEnvelope {
    type Target = CanonicalCommitEnvelopeBody;
    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl Serialize for CanonicalCommitEnvelope {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        wire_projection::serialize(self, serializer, wire_projection::EnvelopeProjection::Full)
    }
}

impl CanonicalCommitEnvelope {
    pub fn semantic_result(&self) -> &crate::transactions::data::RelationalCommitSemanticResult {
        &self.semantic_result
    }

    pub(crate) fn semantic_result_arc(
        &self,
    ) -> std::sync::Arc<crate::transactions::data::RelationalCommitSemanticResult> {
        self.semantic_result.clone()
    }

    #[cfg(test)]
    pub(crate) fn body_mut_for_test(&mut self) -> &mut CanonicalCommitEnvelopeBody {
        &mut self.body
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct CanonicalCommitEnvelopeBody {
    pub(crate) index_definition_delta: CanonicalIndexDefinitionDelta,
    pub(crate) caller_association: Option<super::RelationalCallerAssociation>,
    pub(crate) required_output_codec_retirements:
        Vec<std::sync::Arc<CanonicalRequiredOutputCodecRetirement>>,
    #[serde(default)]
    pub(crate) required_output_origin: Option<std::sync::Arc<CanonicalRequiredOutputOrigin>>,
    #[serde(default)]
    pub(crate) required_output_materialization:
        Option<std::sync::Arc<CanonicalRequiredOutputMaterialization>>,
    #[serde(
        default,
        skip_serializing_if = "CanonicalAuthoredIndexWrites::is_unavailable"
    )]
    pub(crate) authored_index_writes: CanonicalAuthoredIndexWrites,
    pub commit: RelationalCommitReceipt,
    pub branch_context: BranchId,
    /// Exact owner cell state immediately before this commit advances the
    /// branch. It lets recovery admit a branch created after a checkpoint
    /// without rebuilding currentness from a legacy head projection.
    #[serde(default)]
    pub(crate) branch_cell_checkpoint: Option<crate::branch::RelationalBranchCellCheckpoint>,
    pub authority_kind: CanonicalCommitAuthorityKind,
    pub strategy_artifacts: Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
    pub merge_execution_authority: Option<PublishedMergeExecutionAuthority>,
    pub merge_parent_branches: Vec<BranchId>,
    pub merge_base_commits: Vec<CommitId>,
    pub schema_version: SchemaVersionId,
    pub schema_authority: SchemaAuthoritySnapshot,
    pub merged_plan: MergedCommitPlan,
    pub(crate) symbol_texts: crate::symbols::data::SymbolTableSnapshot,
    /// Exact dictionary references made by native aspect and field revision stamps.
    pub(crate) revision_symbol_references: Vec<crate::symbols::data::Symbol>,
    #[serde(default)]
    pub(crate) record_allocations: Vec<crate::history::data::CanonicalRecordAllocation>,
    pub patch: CanonicalAuthoritativePatch,
    #[serde(default = "RelationalDescriptiveTouchGraph::unavailable")]
    descriptive_touches: RelationalDescriptiveTouchGraph,
    pub diagnostics_summary: RelationalDiagnosticArtifact,
    lineage: PublishedLineageArtifact,
    pub derived_index_artifacts: DerivedIndexArtifacts,
    pub schema_transition: Option<SchemaTransitionArtifact>,
    pub schema_continuation_descriptor: Option<SchemaContinuationDescriptor>,
    pub schema_reconciliation_descriptor: Option<SchemaReconciliationDescriptor>,
    pub descriptor_semantics_version: DescriptorSemanticsVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalCommitAuthorityKind {
    VersionedTransaction,
    /// A canonical branch-reference movement that carries no new relational
    /// version. Recovery may readmit legacy metadata publications here after
    /// translating their obsolete lineage vocabulary.
    BranchReferenceMovement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommittedRecordChange<'a> {
    pub commit: &'a RelationalCommitReceipt,
    pub record: &'a PublishedAuthoritativeRecordPatch,
}

impl CanonicalCommitEnvelopeBody {
    pub fn lineage_event_ids(&self) -> &[u64] {
        self.lineage.lineage_event_ids()
    }

    pub fn caller_association(&self) -> Option<&super::RelationalCallerAssociation> {
        self.caller_association.as_ref()
    }

    pub fn descriptive_touches(&self) -> &RelationalDescriptiveTouchGraph {
        &self.descriptive_touches
    }

    pub(crate) fn install_record_allocations(
        &mut self,
        allocations: Vec<crate::history::data::CanonicalRecordAllocation>,
    ) {
        self.record_allocations = allocations;
    }

    pub(crate) fn record_allocations(&self) -> &[crate::history::data::CanonicalRecordAllocation] {
        &self.record_allocations
    }
    pub fn lineage_events(&self) -> &[LineageEventRecord] {
        self.lineage.lineage_events()
    }
    pub fn lineage_decision_log(&self) -> &[LineageDecisionRecord] {
        self.lineage.lineage_decision_log()
    }
    pub fn lineage_decisions_for_event_id(&self, event_id: u64) -> Vec<&LineageDecisionRecord> {
        self.lineage.decisions_for_event_id(event_id).collect()
    }

    pub fn lineage_digest_basis(&self) -> &LineageDigestBasis {
        self.lineage.digest_basis()
    }
    pub fn event_batch_digest_basis(&self) -> &LineageEventBatchDigestBasis {
        self.lineage.event_batch_digest_basis()
    }

    pub fn decision_log_digest_basis(&self) -> &LineageDecisionLogDigestBasis {
        self.lineage.decision_log_digest_basis()
    }

    pub fn lineage_artifact_counters(&self) -> LineageArtifactCounters {
        self.lineage.counters()
    }

    pub fn has_lineage_authority(&self) -> bool {
        self.lineage.has_authority_content()
    }

    pub fn authority_kind(&self) -> CanonicalCommitAuthorityKind {
        self.authority_kind
    }

    pub(crate) fn published_lineage(&self) -> &PublishedLineageArtifact {
        &self.lineage
    }

    pub fn derived_index_artifacts(&self) -> &DerivedIndexArtifacts {
        &self.derived_index_artifacts
    }

    pub(crate) fn touched_record_refs(&self) -> BTreeSet<RecordRef> {
        let mut touched = self
            .patch
            .authoritative_record_patches
            .iter()
            .map(|record| record.target.clone())
            .collect::<BTreeSet<_>>();
        for intent in &self.merged_plan.merged_intents {
            if let Some(target) = intent_record_ref(intent) {
                touched.insert(target);
            }
        }
        touched
    }

    pub(crate) fn committed_record_changes(
        &self,
    ) -> impl Iterator<Item = CommittedRecordChange<'_>> {
        self.patch
            .authoritative_record_patches
            .iter()
            .map(|record| CommittedRecordChange {
                commit: &self.commit,
                record,
            })
    }

    pub(crate) fn committed_record_changes_for_target<'a>(
        &'a self,
        target: &'a RecordRef,
    ) -> impl Iterator<Item = CommittedRecordChange<'a>> + 'a {
        self.committed_record_changes()
            .filter(move |change| &change.record.target == target)
    }

    #[cfg(test)]
    pub(crate) fn published_lineage_mut_for_test(&mut self) -> &mut PublishedLineageArtifact {
        &mut self.lineage
    }

    #[cfg(test)]
    pub(crate) fn rebuild_lineage_basis_for_test(&mut self) {
        self.lineage = crate::lineage::data::LineageFinalizationArtifact::new(
            self.branch_context.clone(),
            crate::lineage::data::FinalizedLineageEventBatch::new(
                self.lineage.lineage_events().to_vec(),
            ),
            crate::lineage::data::LineageDecisionLog::new(
                self.lineage.lineage_decision_log().to_vec(),
            ),
        )
        .publish();
    }
}

fn intent_record_ref(intent: &MutationIntent) -> Option<RecordRef> {
    intent.existing_record_target().map(|target| match target {
        ExistingRecordTarget::Entity(entity_id) => RecordRef::Entity(entity_id),
        ExistingRecordTarget::Relation(relation_id) => RecordRef::Relation(relation_id),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaySchemaVersion(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalReplayRecord {
    pub schema_version: ReplaySchemaVersion,
    pub commit_id: crate::history::data::CommitId,
    pub version_id: crate::identity::data::VersionId,
    pub snapshot_id: crate::snapshots::data::SnapshotId,
    pub patch: crate::publication::patch::data::PublishedAuthoritativePatchEnvelope,
    pub schema_authority: SchemaAuthoritySnapshot,
}

#[cfg(test)]
#[path = "canonical_commit_envelope/tests.rs"]
mod tests;
#[path = "canonical_commit_envelope/text_wire.rs"]
mod text_wire;
pub(crate) use text_wire::CanonicalEnvelopeWire;
#[cfg(test)]
mod semantic_patch_fixture;
