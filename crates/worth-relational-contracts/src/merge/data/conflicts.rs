use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::commit_strategies::data::artifacts::merge_descriptor::StrategyMergeDescriptor;
use crate::history::data::coordinates::CommitId;
use crate::identity::data::VersionId;
use crate::merge::data::identity::IdentityResolutionReason;
use crate::merge::data::policy::AspectComparisonState;
use crate::storage::data::read_records::RecordLifecycleState;
use crate::transactions::data::primitives::RecordRef;
use worth_foundational::facade::AspectKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionMergeClass {
    SourceDeletedTargetLive,
    SourceLiveTargetDeleted,
    DeletedOnBothSides,
    DeletedVsModified,
    DeletedVsRewired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeConflictClass {
    ExactSharedTruth,
    SourceOnlyAddition,
    SchemaDeclaredCorrespondence,
    Deletion(DeletionMergeClass),
    DivergentVisibleState,
    StrategyIntentConflict,
    RelationEndpointDivergence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyConflictClass {
    SameStrategyDivergentOutput,
    DifferentStrategyOverlappingIntent,
    SourceStrategyOnly,
    TargetStrategyOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyConflictEvidence {
    pub class: StrategyConflictClass,
    pub source_commit_ids: Arc<[CommitId]>,
    pub target_commit_ids: Arc<[CommitId]>,
    pub source_descriptors: Arc<[StrategyMergeDescriptor]>,
    pub target_descriptors: Arc<[StrategyMergeDescriptor]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationContinuityClass {
    PreserveRelationIdentity,
    RetireAndIntroduceSuccessor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndpointContinuityClass {
    EndpointsStable,
    SourceEndpointRewired,
    TargetEndpointRewired,
    BothEndpointsRewired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationConflictPropagation {
    RelationLocalOnly,
    RelationLocalRewireCandidate,
    EscalatesToTopologyRegionConflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopologyRegionConflictReason {
    ConnectedRewireNeighborhood,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationConflictEvidence {
    pub endpoint_continuity: EndpointContinuityClass,
    pub relation_continuity: RelationContinuityClass,
    pub propagation: RelationConflictPropagation,
    pub topology_neighborhood_records: Arc<[RecordRef]>,
    pub topology_neighborhood_rewired_records: Arc<[RecordRef]>,
    pub topology_region_conflict_reason: Option<TopologyRegionConflictReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectConflictEvidence {
    pub aspect_key: AspectKey,
    pub comparison: AspectComparisonState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeVisibilityEvidenceKind {
    SourceEmbeddedSurface,
    TargetCandidateViewLookup,
    TargetEmbeddedSurface,
    BaseResolvedViewLookup,
    BaseHistoricalWindow,
    BaseLastResortViewLookup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeVisibilityState {
    Visible,
    NotVisible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeVisibilityEvidence {
    pub observed_record: RecordRef,
    pub kind: MergeVisibilityEvidenceKind,
    pub state: MergeVisibilityState,
    pub embedded_surface_state: Option<MergeVisibilityState>,
    pub lifecycle: Option<RecordLifecycleState>,
    pub created_at_version: Option<VersionId>,
    pub retired_at_version: Option<VersionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeConflictClassification {
    pub record: RecordRef,
    pub class: MergeConflictClass,
    pub identity_reason: IdentityResolutionReason,
    pub validated_schema_correspondence: bool,
    pub aspect_evidence: Arc<[AspectConflictEvidence]>,
    pub strategy_evidence: Option<StrategyConflictEvidence>,
    pub relation_evidence: Option<RelationConflictEvidence>,
    pub target_record: Option<RecordRef>,
    pub base_record_visible: bool,
    pub source_record_visible: bool,
    pub target_record_visible: bool,
    pub base_visibility_evidence: MergeVisibilityEvidence,
    pub source_visibility_evidence: MergeVisibilityEvidence,
    pub target_visibility_evidence: MergeVisibilityEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictClassificationSummary {
    pub classified_record_count: usize,
    pub exact_shared_truth_count: usize,
    pub source_only_addition_count: usize,
    pub schema_declared_correspondence_count: usize,
    pub deletion_conflict_count: usize,
    pub divergent_visible_state_count: usize,
    pub strategy_intent_conflict_count: usize,
    pub relation_endpoint_divergence_count: usize,
    pub classifications: Arc<[MergeConflictClassification]>,
}
