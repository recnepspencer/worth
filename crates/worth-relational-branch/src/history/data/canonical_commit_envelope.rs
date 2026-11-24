use crate::internal_port::history::data::canonical_commit_envelope::descriptive_touches::RelationalDescriptiveTouchGraphOwnerPort as _;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::indexes::data::DerivedIndexArtifacts;
use crate::lineage::data::{
    LineageArtifactCounters, LineageDecisionLogDigestBasis, LineageDecisionRecord,
    LineageDigestBasis, LineageEventBatchDigestBasis, LineageEventRecord, PublishedLineageArtifact,
};
use crate::publication::patch::data::CanonicalAuthoritativePatch;
use crate::schema::data::{
    DescriptorSemanticsVersion, SchemaAuthoritySnapshot, SchemaContinuationDescriptor,
    SchemaReconciliationDescriptor, SchemaTransitionArtifact, SchemaVersionId,
};
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::transactions::data::outcomes::plan_artifacts::PublishedMergeExecutionAuthority;
use crate::transactions::data::primitives::ExistingRecordTarget;
use crate::transactions::data::primitives::RecordRef;

pub mod authored_index_writes;
pub mod index_catalog;
pub mod live_managed_source_history;
pub use authored_index_writes::{CanonicalAuthoredIndexWrite, CanonicalAuthoredIndexWrites};
pub use index_catalog::CanonicalIndexDefinitionDelta;
pub mod descriptive_decode;
pub mod original_inputs;
pub mod recorded_inputs;
pub mod schema_carrier;
pub use live_managed_source_history::LiveManagedSourceHistory;
pub use original_inputs::{
    RelationalIntroducedSchemaObservation, RelationalOriginalInputObservation,
};
pub mod authoritative_artifact_encoding;
pub mod control_allocation;
mod embedded_checkpoint;
pub mod lineage_projection;
pub mod owned_graph_selection;
pub mod payload_digest;
pub mod retained_entity_selection;
pub mod shared;
pub use authoritative_artifact_encoding::AuthoritativeArtifactEnvelopeRef;
pub use control_allocation::CanonicalEnvelopeControlAllocation;
pub use embedded_checkpoint::EmbeddedCheckpointEnvelopeRef;
pub use owned_graph_selection::CanonicalOwnedGraphSelection;
pub use retained_entity_selection::CanonicalRetainedEntitySelection;
pub use shared::SharedCanonicalCommitEnvelope;
#[path = "canonical_commit_envelope/descriptive_touches.rs"]
pub mod descriptive_touches;
pub mod payload_encoding;
pub mod required_output_codec_retirement;
pub mod required_output_materialization;
pub mod required_output_origin;
pub use descriptive_touches::{
    RelationalDescriptiveTouch, RelationalDescriptiveTouchGraph,
    RelationalDescriptiveTouchPrecision, RelationalTouchAdjacencyDirection,
};
pub use required_output_codec_retirement::{
    CanonicalCodecRetirementKind, CanonicalRequiredOutputCodecRetirement,
};
pub use required_output_materialization::{
    CanonicalRequiredOutputMaterialization, CanonicalRequiredOutputMaterializationRecord,
};
pub use required_output_origin::validate_authored_restoration_candidate;
pub use required_output_origin::CanonicalHistoricalOutputSelection;
pub use required_output_origin::{
    CanonicalAuthoredRestoration, CanonicalAuthoredRetirement, CanonicalCoupledAuthoredPublication,
    CanonicalCoupledAuthoredSources, CanonicalOutputMember, CanonicalOutputPublication,
    CanonicalProducerRetirement, CanonicalRequiredOutputOrigin, CanonicalRequiredOutputRelation,
    OutputOriginHistory, OwnedValidatedOutputOrigins, ValidatedOutputOrigins,
};

pub mod draft;
pub mod readmission_memory;
pub use readmission_memory::{
    CanonicalEnvelopeReadmissionMemory, CodecDeclarationsReadmissionMemory,
    StrategyReadmissionMemory,
};
pub mod wire_projection;
pub use draft::CanonicalCommitEnvelopeDraft;
pub use wire_projection::EnvelopeProjection;

/// A complete immutable envelope. Its semantic image exists before it can be
/// serialized, shared or registered for publication.
#[cfg_attr(not(doctest), doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/facade_examples/canonical_envelope.md")))]
#[derive(Debug)]
pub struct CanonicalCommitEnvelope {
    original_inputs: std::sync::OnceLock<Option<recorded_inputs::RecordedCanonicalInputs>>,
    /// Live exact-envelope acceleration; deep Clone and all codecs omit it.
    pub(crate) live_managed_source_history: LiveManagedSourceHistory,
    body: CanonicalCommitEnvelopeBody,
    semantic_result: std::sync::Arc<
        crate::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult,
    >,
    // Intrinsic input and body admission outlive every decoded body field.
    #[allow(
        dead_code,
        reason = "Original diagnostic, proof and funding custody remains owned through retirement"
    )]
    pub(crate) decoded_bytes: Option<worth_execution::ExecutionImmutableBytes>,
    #[allow(
        dead_code,
        reason = "Original diagnostic, proof and funding custody remains owned through retirement"
    )]
    pub(crate) decoded_memory: Option<worth_execution::ExecutionMemoryReservation>,
    // LAST: real weak-position observers share only this fixed allocation bill.
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
    /// Index definitions this commit's canonical catalog delta adds.
    pub fn index_definition_additions(&self) -> &[crate::indexes::data::DerivedIndexDefinition] {
        self.index_definition_delta.definitions()
    }

    pub fn semantic_result(
        &self,
    ) -> &crate::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult
    {
        &self.semantic_result
    }

    pub(crate) fn body_mut_for_test(&mut self) -> &mut CanonicalCommitEnvelopeBody {
        &mut self.body
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct CanonicalCommitEnvelopeBody {
    pub(crate) content: crate::content::publication::ContentPublication,
    pub(crate) index_definition_delta: CanonicalIndexDefinitionDelta,
    pub(crate) caller_association: Option<super::RelationalCallerAssociation>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) historical_output_selection:
        Option<std::sync::Arc<required_output_origin::CanonicalHistoricalOutputSelection>>,
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
    pub(crate) branch_cell_checkpoint:
        Option<crate::branch::reference_checkpoint_data::RelationalBranchCellCheckpoint>,
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
    OwnedGraphSelection(std::sync::Arc<CanonicalOwnedGraphSelection>),
}

impl CanonicalCommitAuthorityKind {
    pub fn advances_version(&self) -> bool {
        match self {
            Self::VersionedTransaction
            | Self::RetainedEntitySelection(_)
            | Self::OwnedGraphSelection(_) => true,
            Self::BranchReferenceMovement => false,
        }
    }

    pub fn retained_entity_selection(&self) -> Option<&CanonicalRetainedEntitySelection> {
        match self {
            Self::RetainedEntitySelection(selection) => Some(selection),
            Self::VersionedTransaction
            | Self::BranchReferenceMovement
            | Self::OwnedGraphSelection(_) => None,
        }
    }
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
        if let CanonicalCommitAuthorityKind::OwnedGraphSelection(selection) = &self.authority_kind {
            touched.extend(selection.targets());
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
                match &self.authority_kind {
                    CanonicalCommitAuthorityKind::OwnedGraphSelection(selection) => Some(selection),
                    _ => None,
                }
                .into_iter()
                .flat_map(|selection| selection.targets().map(Some)),
            )
            .chain(
                self.merged_plan
                    .merged_intents
                    .iter()
                    .map(intent_record_ref),
            )
    }

    pub(crate) fn published_lineage_mut_for_test(&mut self) -> &mut PublishedLineageArtifact {
        &mut self.lineage
    }
}

fn intent_record_ref(intent: &MutationIntent) -> Option<RecordRef> {
    intent.existing_record_target().map(|target| match target {
        ExistingRecordTarget::Entity(entity_id) => RecordRef::Entity(entity_id),
        ExistingRecordTarget::Relation(relation_id) => RecordRef::Relation(relation_id),
    })
}

pub mod committed_record_changes;
pub use committed_record_changes::CommittedRecordChange;

pub mod replay_record;
pub use replay_record::{RelationalReplayRecord, ReplaySchemaVersion};

#[path = "canonical_commit_envelope/tests.rs"]
pub mod tests;
#[path = "canonical_commit_envelope/text_wire.rs"]
pub mod text_wire;
pub use text_wire::{CanonicalEnvelopeSeed, CanonicalEnvelopeWire};
#[cfg(test)]
#[path = "canonical_commit_envelope_inline_type_budget.rs"]
pub mod inline_type_budget;

pub mod lineage_rebuild_test;

pub mod fixture_image;

/// Operations used by the Relational owners.
pub trait CanonicalCommitEnvelopeOwnerPort: Sized {
    fn semantic_result_arc(
        &self,
    ) -> std::sync::Arc<
        crate::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult,
    >;
}

impl CanonicalCommitEnvelopeOwnerPort for CanonicalCommitEnvelope {
    fn semantic_result_arc(
        &self,
    ) -> std::sync::Arc<
        crate::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult,
    > {
        self.semantic_result.clone()
    }
}
mod body_fixture;
pub use body_fixture::{
    CanonicalCommitEnvelopeBodyFixturePort, CanonicalCommitEnvelopeFieldsPort,
    CanonicalCommitEnvelopeFixturePort,
};

mod body_owner_port;
pub use body_owner_port::{
    CanonicalCommitEnvelopeBodyFieldsPort, CanonicalCommitEnvelopeBodyOwnerPort,
};
