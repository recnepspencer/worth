use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::transactions::data::RecordRef;
use worth_foundational::facade::AspectKey;

use super::MergeResolvedAspectValueStrategy;

mod custom_name;
pub use custom_name::MergePolicyName;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomMergePolicyIdentity {
    pub name: MergePolicyName,
    pub semantic_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectMergePolicyKind {
    FailOnConflict,
    LastWriterWins,
    MonotonicCounter,
    AdditiveSet,
    PreferRicher,
    Custom(CustomMergePolicyIdentity),
}

impl AspectMergePolicyKind {
    pub const fn ownership_class(&self) -> MergePolicyOwnershipClass {
        match self {
            AspectMergePolicyKind::Custom(_) => MergePolicyOwnershipClass::CustomPolicy,
            AspectMergePolicyKind::FailOnConflict
            | AspectMergePolicyKind::LastWriterWins
            | AspectMergePolicyKind::MonotonicCounter
            | AspectMergePolicyKind::AdditiveSet
            | AspectMergePolicyKind::PreferRicher => MergePolicyOwnershipClass::RuntimeBuiltIn,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectMergePolicyDeclaration {
    pub aspect_key: AspectKey,
    pub policy: AspectMergePolicyKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergePolicyResolution {
    AutoResolved,
    RequiresManualResolution,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeManualResolutionClass {
    GenericRuntimeConflict,
    StrategyIntentConflict,
    MissingVisibleState,
    MissingAncestorValueBasis,
    UnvalidatedSchemaCorrespondence,
    MixedAspectManualResolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergePolicyRejectClass {
    BuiltInFailOnConflict,
    LastWriterWinsCausalConflict,
    InvalidBuiltInPolicyValueShape,
    CustomPolicyRejected,
    MixedAspectRejectClasses,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergePolicyDecisionBoundary {
    AutoResolved,
    RequiresManualResolution { class: MergeManualResolutionClass },
    Reject { class: MergePolicyRejectClass },
}

impl MergePolicyDecisionBoundary {
    pub const fn resolution(self) -> MergePolicyResolution {
        match self {
            MergePolicyDecisionBoundary::AutoResolved => MergePolicyResolution::AutoResolved,
            MergePolicyDecisionBoundary::RequiresManualResolution { .. } => {
                MergePolicyResolution::RequiresManualResolution
            }
            MergePolicyDecisionBoundary::Reject { .. } => MergePolicyResolution::Reject,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeExecutionReadiness {
    Admitted,
    Blocked,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectComparisonState {
    Equal,
    SourceOnly,
    TargetOnly,
    Divergent,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredMergeAction {
    KeepSourceAddition,
    KeepExactSharedTruth,
    ReconcileSchemaCorrespondence,
    ReconcileDivergentVisibleState,
    ConvergeDeletedOnBothSides,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionExecutionClass {
    SourceDeletedTargetLive,
    SourceLiveTargetDeleted,
    DeletedOnBothSides,
    DeletedVsModified,
    DeletedVsRewired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopologyExecutionClass {
    RelationEndpointStable,
    RelationEndpointRewiredLocal,
    RelationEndpointRewiredEscalated,
    TopologyRegionConflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopologyRewireAdmissionPolicy {
    AlwaysEscalateToTopologyRegion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeResolutionClass {
    SourceOnlyAddition,
    ExactSharedTruth,
    SchemaDeclaredCorrespondence,
    Deletion(DeletionExecutionClass),
    Topology(TopologyExecutionClass),
    DivergentVisibleState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeExecutableClass {
    AdoptSourceRecord,
    PreserveSharedRecord,
    ReconcileRecord,
    ConvergeDeletedOnBothSides,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredRecordExecutionIntentKind {
    AdoptSourceRecord,
    PreserveSharedRecord,
    ReconcileRecord,
    ConvergeDeletedOnBothSides,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredAspectAction {
    AdoptSourceAspect,
    KeepSharedAspect,
    ReconcileCorrespondedAspect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizedAspectValueUsage {
    NotAuthorized,
    ConsumeVisibleValue,
    ConsumeBaseValue,
    EqualityWitnessOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedAspectValueSurface {
    pub source: AuthorizedAspectValueUsage,
    pub target: AuthorizedAspectValueUsage,
    pub base: AuthorizedAspectValueUsage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredAspectExecutionIntent {
    AdoptSourceValue {
        authorized_values: AuthorizedAspectValueSurface,
    },
    PreserveSharedValue {
        authorized_values: AuthorizedAspectValueSurface,
    },
    ReconcileVisibleValues {
        authorized_values: AuthorizedAspectValueSurface,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredMergeBlockedReason {
    ManualConflictResolutionRequired,
    StrategyIntentConflictRequiresManualResolution,
    MissingVisibleState,
    MissingAncestorValueBasis,
    UnvalidatedSchemaCorrespondence,
    RelationEndpointRewiredLocal,
    RelationEndpointRewiredEscalated,
    TopologyRegionConflict,
    SourceDeletedTargetLive,
    SourceLiveTargetDeleted,
    DeletedOnBothSides,
    DeletedVsModified,
    DeletedVsRewired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredMergeRejectedReason {
    FailOnConflictPolicy,
    CustomPolicyRejected,
    MixedPolicyRejectClasses,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredAspectDenialIntent {
    BlockedSourceDeletedTargetLive,
    BlockedSourceLiveTargetDeleted,
    BlockedDeletedOnBothSides,
    BlockedDeletedVsModified,
    BlockedDeletedVsRewired,
    BlockedStrategyIntentConflict,
    BlockedMissingVisibleState,
    BlockedMissingAncestorValueBasis,
    BlockedUnvalidatedSchemaCorrespondence,
    BlockedRelationEndpointRewiredLocal,
    BlockedRelationEndpointRewiredEscalated,
    BlockedTopologyRegionConflict,
    BlockedManualResolution,
    RejectedPolicy,
    RejectedCustomPolicy,
    RejectedMixedPolicyClasses,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredAspectOutcome {
    pub aspect_key: AspectKey,
    pub applied_policy: Option<AspectMergePolicyKind>,
    pub readiness: MergeExecutionReadiness,
    pub lowered_action: Option<LoweredAspectAction>,
    pub authorized_values: Option<AuthorizedAspectValueSurface>,
    pub execution_intent: Option<LoweredAspectExecutionIntent>,
    pub resolved_value_strategy: Option<MergeResolvedAspectValueStrategy>,
    pub denial_intent: Option<LoweredAspectDenialIntent>,
    pub blocked_reason: Option<LoweredMergeBlockedReason>,
    pub rejected_reason: Option<LoweredMergeRejectedReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredRecordExecutionAspectIntent {
    pub aspect_key: AspectKey,
    pub intent: LoweredAspectExecutionIntent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredRecordExecutionBundle {
    pub kind: LoweredRecordExecutionIntentKind,
    pub aspects: Arc<[LoweredRecordExecutionAspectIntent]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredRecordDecisionKind {
    Execute,
    Block,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredRecordDenialKind {
    BlockedSourceDeletedTargetLive,
    BlockedSourceLiveTargetDeleted,
    BlockedDeletedOnBothSides,
    BlockedDeletedVsModified,
    BlockedDeletedVsRewired,
    BlockedMissingVisibleState,
    BlockedMissingAncestorValueBasis,
    BlockedUnvalidatedSchemaCorrespondence,
    BlockedRelationEndpointRewiredLocal,
    BlockedRelationEndpointRewiredEscalated,
    BlockedTopologyRegionConflict,
    BlockedManualResolution,
    RejectedPolicy,
    RejectedCustomPolicy,
    RejectedMixedPolicyClasses,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredRecordDenialAspectIntent {
    pub aspect_key: AspectKey,
    pub intent: LoweredAspectDenialIntent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredRecordDenialBundle {
    pub kind: LoweredRecordDenialKind,
    pub aspects: Arc<[LoweredRecordDenialAspectIntent]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredRecordDecision {
    Execute(LoweredRecordExecutionBundle),
    Block(LoweredRecordDenialBundle),
    Reject(LoweredRecordDenialBundle),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedAspectMergePolicy {
    pub aspect_key: AspectKey,
    pub policy: AspectMergePolicyKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergePolicyOwnershipClass {
    RuntimeBuiltIn,
    CustomPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergePolicyOwnershipSurface {
    RuntimeOnly,
    ContainsCustomPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergePolicyProofBoundary {
    pub ownership_surface: MergePolicyOwnershipSurface,
    pub decision_boundary: MergePolicyDecisionBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergePolicyResolutionRecord {
    pub record: RecordRef,
    pub target_record: Option<RecordRef>,
    pub classification: crate::merge::data::MergeConflictClass,
    pub aspect_resolutions: Arc<[AspectPolicyResolutionRecord]>,
    pub applied_policies: Arc<[ResolvedAspectMergePolicy]>,
    pub proof_boundary: MergePolicyProofBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectPolicyResolutionRecord {
    pub aspect_key: AspectKey,
    pub comparison: AspectComparisonState,
    pub applied_policy: Option<AspectMergePolicyKind>,
    pub decision_boundary: MergePolicyDecisionBoundary,
    pub resolved_value_strategy: Option<MergeResolvedAspectValueStrategy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergePolicyResolutionSummary {
    pub resolved_record_count: usize,
    pub auto_resolved_count: usize,
    pub requires_manual_resolution_count: usize,
    pub reject_count: usize,
    pub runtime_only_record_count: usize,
    pub custom_policy_record_count: usize,
    pub records: Arc<[MergePolicyResolutionRecord]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredMergePlanRecord {
    pub record: RecordRef,
    pub target_record: Option<RecordRef>,
    pub classification: crate::merge::data::MergeConflictClass,
    pub resolution_class: MergeResolutionClass,
    pub executable_class: Option<MergeExecutableClass>,
    pub causal_disposition: crate::merge::data::MergeRecordCausalDisposition,
    pub applied_policies: Arc<[ResolvedAspectMergePolicy]>,
    pub policy_proof_boundary: MergePolicyProofBoundary,
    pub readiness: MergeExecutionReadiness,
    pub record_decision: LoweredRecordDecision,
    pub lowered_action: Option<LoweredMergeAction>,
    pub blocked_reason: Option<LoweredMergeBlockedReason>,
    pub rejected_reason: Option<LoweredMergeRejectedReason>,
    pub aspect_outcomes: Arc<[LoweredAspectOutcome]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredMergePlanSummary {
    pub record_count: usize,
    pub admitted_count: usize,
    pub blocked_count: usize,
    pub rejected_count: usize,
    pub fully_execution_ready: bool,
    pub records: Arc<[LoweredMergePlanRecord]>,
}
