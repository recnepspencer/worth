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
mod construction;
mod live_managed_source_history;
pub(crate) use authored_index_writes::{CanonicalAuthoredIndexWrite, CanonicalAuthoredIndexWrites};
pub(crate) use live_managed_source_history::LiveManagedSourceHistory;
mod authoritative_artifact_encoding;
mod embedded_checkpoint;
mod lineage_projection;
mod payload_digest;
mod retained_entity_selection;
pub(crate) use authoritative_artifact_encoding::AuthoritativeArtifactEnvelopeRef;
pub(crate) use embedded_checkpoint::EmbeddedCheckpointEnvelopeRef;
pub use retained_entity_selection::CanonicalRetainedEntitySelection;
#[path = "canonical_commit_envelope/descriptive_touches.rs"]
mod descriptive_touches;
mod payload_encoding;
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
pub(crate) use required_output_origin::validate_authored_restoration_candidate;
pub(crate) use required_output_origin::{
    CanonicalAuthoredRestoration, CanonicalAuthoredRetirement, CanonicalCoupledAuthoredPublication,
    CanonicalCoupledAuthoredSources, CanonicalOutputMember, CanonicalOutputPublication,
    CanonicalProducerRetirement, CanonicalRequiredOutputOrigin, CanonicalRequiredOutputRelation,
    OutputOriginHistory, OwnedValidatedOutputOrigins, ValidatedOutputOrigins,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalCommitEnvelope {
    pub(crate) content: crate::content::ContentPublication,
    /// Live exact-envelope acceleration; deep Clone and all codecs omit it.
    #[serde(skip)]
    pub(crate) live_managed_source_history: LiveManagedSourceHistory,
    pub(crate) required_output_codec_retirements:
        Vec<std::sync::Arc<CanonicalRequiredOutputCodecRetirement>>,
    #[serde(default)]
    pub(crate) required_output_origin: Option<std::sync::Arc<CanonicalRequiredOutputOrigin>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) producer_cascade_retirement: Option<std::sync::Arc<CanonicalProducerRetirement>>,
    pub(crate) authored_cascade_retirement: Option<std::sync::Arc<CanonicalAuthoredRetirement>>,
    pub(crate) authored_cascade_restoration: Option<std::sync::Arc<CanonicalAuthoredRestoration>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) coupled_authored_publication:
        Option<std::sync::Arc<CanonicalCoupledAuthoredPublication>>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalCommitAuthorityKind {
    VersionedTransaction,
    /// A canonical branch-reference movement that carries no new relational
    /// version. Recovery may readmit legacy metadata publications here after
    /// translating their obsolete lineage vocabulary.
    BranchReferenceMovement,
    /// A fresh version selecting one retained authoritative entity payload.
    RetainedEntitySelection(std::sync::Arc<CanonicalRetainedEntitySelection>),
}

impl CanonicalCommitAuthorityKind {
    pub fn advances_version(&self) -> bool {
        match self {
            Self::VersionedTransaction | Self::RetainedEntitySelection(_) => true,
            Self::BranchReferenceMovement => false,
        }
    }

    pub fn retained_entity_selection(&self) -> Option<&CanonicalRetainedEntitySelection> {
        match self {
            Self::RetainedEntitySelection(selection) => Some(selection),
            Self::VersionedTransaction | Self::BranchReferenceMovement => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommittedRecordChange<'a> {
    pub commit: &'a RelationalCommitReceipt,
    pub record: &'a PublishedAuthoritativeRecordPatch,
}

impl CanonicalCommitEnvelope {
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
    pub fn authority_kind(&self) -> CanonicalCommitAuthorityKind {
        self.authority_kind.clone()
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
        if let Some(selection) = self.authority_kind.retained_entity_selection() {
            touched.insert(RecordRef::Entity(selection.entity()));
        }
        touched
    }

    /// Borrow authored targets too: an equal-valued authored update can have
    /// no surviving field patch and still be a distinct record occurrence.
    pub(crate) fn authored_intent_targets(&self) -> impl Iterator<Item = Option<RecordRef>> + '_ {
        self.authority_kind
            .retained_entity_selection()
            .map(|selection| Some(RecordRef::Entity(selection.entity())))
            .into_iter()
            .chain(
                self.merged_plan
                    .merged_intents
                    .iter()
                    .map(intent_record_ref),
            )
    }

    /// Canonical bytes for owner truth only. Diagnostics and optional derived
    /// indexes have distinct lifecycle lanes and cannot perturb commit-byte
    /// accounting.
    pub(crate) fn encode_authoritative_payload(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        let mut bytes = Vec::new();
        self.write_authoritative_payload(&mut bytes)?;
        Ok(bytes)
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
pub(crate) use text_wire::{CanonicalEnvelopeWire, PositionedEnvelopeSeed};
