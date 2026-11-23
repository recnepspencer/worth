pub(crate) mod ancestry;
pub(crate) mod artifacts;
pub(crate) mod causal;
pub(crate) mod conflicts;
pub(crate) mod decisions;
pub(crate) mod execution;
pub(crate) mod execution_artifacts;
pub(crate) mod identity;
pub(crate) mod inspection_digest;
pub(crate) mod plans;
pub(crate) mod policy;
mod readmission_memory;
pub(crate) mod requests;
pub(crate) mod resolved_value_strategy;
pub(crate) use readmission_memory::{
    attach_policy_name, MergeBasisMemory, MergeIdentityMemory, MergeLeafMemory,
    MergePolicyRowMemory, MergeReadmissionMemory, MergeStrategyMemory,
};

pub use crate::history::data::{MergeBaseSelectionRule, ResolvedMergeBase};
pub use ancestry::{BranchDeltaSummary, MergeAncestrySummary};
pub(crate) use artifacts::MergeWitnessRows;
pub(crate) use artifacts::RelationalSchemaReconciliationWitnessRowInput;
pub(crate) use artifacts::{
    correspondence_posture_for_candidate, row_for_candidate, schema_declared_correspondence_posture,
};
pub use artifacts::{
    MergeArtifactDigestBasis, MergeBaseDigestBasis, MergeCausalDigestBasis,
    MergeConflictDigestBasis, MergeExecutionAuthorityContract, MergeExecutionAuthorizationRule,
    MergeExecutionConsumptionRule, MergeExecutionDecisionSurface, MergeIdentityDigestBasis,
    MergeLoweredAspectDigestRow, MergeLoweredPlanDigestBasis, MergePlanningArtifactCore,
    MergePlanningSummary, MergePolicyAspectDigestRow, MergePolicyDigestBasis,
    MergeRequestDigestBasis, MergeSchemaKindClass, MergeSchemaKindSemanticSnapshot,
    MergeSchemaSnapshotDigestBasis, RelationalMergeAdmittedSurfaceRow,
    RelationalMergeAspectPolicyWitnessRow, RelationalMergeCorrespondenceWitness,
    RelationalMergeCorrespondenceWitnessPosture, RelationalMergeCorrespondenceWitnessRow,
    RelationalMergeDeletionStrategyWitnessRow, RelationalMergeInspectionAdmission,
    RelationalMergeInspectionArtifact, RelationalMergeInspectionInput,
    RelationalMergeInspectionRow, RelationalMergeProofPacket,
    RelationalMergeProofPacketAdmissionPosture, RelationalMergeProofPacketCanonicalBasis,
    RelationalMergeStrategyWitness, RelationalMergeTopologyStrategyWitnessRow,
    RelationalSchemaReconciliationBasisRow, RelationalSchemaReconciliationCorrespondenceLinkRow,
    RelationalSchemaReconciliationWitness, RelationalSchemaReconciliationWitnessDenial,
    RelationalSchemaReconciliationWitnessPosture, RelationalSchemaReconciliationWitnessRow,
};
pub use causal::{
    BranchCausalDot, CausalAnnotationSummary, CausalFrontier, CommitCausalMetadata,
    CommitCausalRelation, MergeCausalEvidenceModel, MergeRecordCausalAnnotation,
    MergeRecordCausalDisposition,
};
pub use conflicts::{
    AspectConflictEvidence, ConflictClassificationSummary, DeletionMergeClass,
    EndpointContinuityClass, MergeConflictClass, MergeConflictClassification,
    MergeVisibilityEvidence, MergeVisibilityEvidenceKind, MergeVisibilityState,
    RelationConflictEvidence, RelationConflictPropagation, RelationContinuityClass,
    StrategyConflictClass, StrategyConflictEvidence, TopologyRegionConflictReason,
};
pub use decisions::{
    MergePlanningDecisionKind, MergePlanningDecisionLog, MergePlanningDecisionLogDigestBasis,
    MergePlanningDecisionRecord,
};
pub(crate) use execution::{
    aspect_reference, bound_parent_order, compiled_executable_plan_digest, equality_witness_digest,
    materialized_value_aspect_key, schema_snapshot_digest, visible_record_snapshot,
};
pub use execution::{
    AdoptSourceRecordPlan, BoundExecutableMergePlan, BoundExecutableMergeRecordPlan,
    ConvergeDeletedOnBothSidesRecordPlan, DeletedOnBothSidesSemantics, ExecutableAspectPlan,
    MaterializedAspectValue, MaterializedAspectValueEvidence, MergeExecutableRecordProvenance,
    MergeExecutionAuthorityBinding, MergeExecutionCompilationError, MergeExecutionDeniedRecord,
    MergeExecutionError, MergeExecutionFreshnessPolicy, MergeExecutionMutationPlanError,
    MergeExecutionPreparationError, MergeExecutionReadinessReport, MergeLineageContinuityVerdict,
    MergeValueMaterialization, MergeValueSourceSide, PreserveSharedRecordPlan, ReconcileRecordPlan,
    ReconciledIdentityBasis, RuntimeInstanceId, SharedTruthWitness, VisibleMergeRecordSnapshot,
};
pub(crate) use execution_artifacts::diagnostics_plan_from_record_plans;
pub use execution_artifacts::{
    ExecutedMergeAspectClass, ExecutedMergeAspectDiagnosticRow, ExecutedMergeRecordClass,
    ExecutedMergeRecordDiagnosticRow, MergeExecutionDiagnosticsPlan,
};
pub use identity::{
    CustomIdentityBasisIdentity, DeclaredIdentityKeys, IdentityBasisDeclaration, IdentityBasisKind,
    IdentityBasisName, IdentityBasisScope, IdentityDiscoverySummary, IdentityMatchCandidate,
    IdentityMatchClass, IdentityResolutionReason, MergeRecordIdentity,
    SchemaDeclaredCorrespondenceValidationSummary,
};
pub(crate) mod planning_denial;
pub use planning_denial::MergePlanningError;
pub(crate) use plans::{
    BranchCommitDelta, BranchTouchedRecordDelta, CausallyAnnotatedMergePlan,
    ConflictClassifiedMergePlan, HistoryScopedMergePlan, IdentityScopedMergePlan, LoweredMergePlan,
    PolicyResolvedMergePlan, ValidatedSchemaDeclaredCorrespondence,
};
pub use policy::{
    AspectComparisonState, AspectMergePolicyDeclaration, AspectMergePolicyKind,
    AspectPolicyResolutionRecord, AuthorizedAspectValueSurface, AuthorizedAspectValueUsage,
    CustomMergePolicyIdentity, DeletionExecutionClass, LoweredAspectAction,
    LoweredAspectDenialIntent, LoweredAspectExecutionIntent, LoweredAspectOutcome,
    LoweredMergeAction, LoweredMergeBlockedReason, LoweredMergePlanRecord, LoweredMergePlanSummary,
    LoweredMergeRejectedReason, LoweredRecordDecision, LoweredRecordDecisionKind,
    LoweredRecordDenialAspectIntent, LoweredRecordDenialBundle, LoweredRecordDenialKind,
    LoweredRecordExecutionAspectIntent, LoweredRecordExecutionBundle,
    LoweredRecordExecutionIntentKind, MergeExecutableClass, MergeExecutionReadiness,
    MergeManualResolutionClass, MergePolicyDecisionBoundary, MergePolicyName,
    MergePolicyOwnershipClass, MergePolicyOwnershipSurface, MergePolicyProofBoundary,
    MergePolicyRejectClass, MergePolicyResolution, MergePolicyResolutionRecord,
    MergePolicyResolutionSummary, MergeResolutionClass, ResolvedAspectMergePolicy,
    TopologyExecutionClass, TopologyRewireAdmissionPolicy,
};
pub use requests::{
    MergeExecutionRequest, MergeIntent, MergePlanningRequest, NormalizedRelationalMergeRequest,
    OwnerBoundMergeExecutionRequest, OwnerBoundMergePlanningRequest,
    RelationalFoundationalMergeRequest, RelationalMergeCorrespondencePosture,
    RelationalMergeRequestBindingDenial, RelationalMergeRequestFamily,
    RelationalMergeRequestNormalizationDenial, RelationalMergeSchemaReconciliationPosture,
    RelationalMergeScope, RelationalMergeTopologyIntent,
};
pub use resolved_value_strategy::MergeResolvedAspectValueStrategy;

pub(crate) mod blocked_posture;
pub(crate) mod execution_authority_contract;
pub(crate) mod policy_meaning;

pub(crate) mod prepared_execution;
pub use prepared_execution::PreparedMergeExecution;
pub(crate) use prepared_execution::{
    CompiledMergeExecution, ExecutionReadyLoweredMergePlan, PreparedMergeMutationPlan,
};
pub(crate) mod visible_record;
pub(crate) use visible_record::{VisibleMergeRecord, VisibleMergeRecordKind};
