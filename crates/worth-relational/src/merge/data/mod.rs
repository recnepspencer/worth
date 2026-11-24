pub(crate) use worth_relational_contracts::internal_port::merge::data::ancestry;
pub(crate) use worth_relational_contracts::internal_port::merge::data::artifacts;
pub(crate) use worth_relational_contracts::internal_port::merge::data::causal;
pub(crate) use worth_relational_contracts::internal_port::merge::data::conflicts;
pub(crate) use worth_relational_contracts::internal_port::merge::data::decisions;
pub(crate) use worth_relational_contracts::internal_port::merge::data::execution;
pub(crate) use worth_relational_contracts::internal_port::merge::data::execution_artifacts;
pub(crate) use worth_relational_contracts::internal_port::merge::data::identity;
pub(crate) mod plans;
pub(crate) use worth_relational_contracts::internal_port::merge::data::policy;
pub(crate) use worth_relational_contracts::internal_port::merge::data::readmission_memory;
pub(crate) mod requests;
pub(crate) use worth_relational_contracts::internal_port::merge::data::resolved_value_strategy;

pub use crate::history::data::{MergeBaseSelectionRule, ResolvedMergeBase};
pub use ancestry::{BranchDeltaSummary, MergeAncestrySummary};
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
pub(crate) use worth_relational_contracts::internal_port::merge::data::planning_denial;

pub(crate) use worth_relational_contracts::internal_port::merge::data::blocked_posture;
pub(crate) use worth_relational_contracts::internal_port::merge::data::execution_authority_contract;
pub(crate) use worth_relational_contracts::internal_port::merge::data::policy_meaning;

pub(crate) mod prepared_execution;
pub use prepared_execution::PreparedMergeExecution;
pub(crate) use prepared_execution::{
    CompiledMergeExecution, ExecutionReadyLoweredMergePlan, PreparedMergeMutationPlan,
};
pub(crate) use visible_record::{VisibleMergeRecord, VisibleMergeRecordKind};
pub(crate) use worth_relational_contracts::internal_port::merge::data::visible_record;
