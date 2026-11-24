pub use crate::contracts::merge::data::execution_authority_contract;
pub use crate::contracts::merge::data::{artifacts, identity, policy, readmission_memory};
pub mod plans;
pub mod requests;

pub use crate::contracts::history::data::merge_branch_basis::{
    MergeBaseSelectionRule, ResolvedMergeBase,
};
pub use crate::contracts::merge::data::ancestry::{BranchDeltaSummary, MergeAncestrySummary};
pub use crate::contracts::merge::data::artifacts::aspect_digest_rows::{
    MergeLoweredAspectDigestRow, MergePolicyAspectDigestRow,
};
pub(crate) use crate::contracts::merge::data::artifacts::correspondence_witness::correspondence_posture_for_candidate;
pub use crate::contracts::merge::data::artifacts::correspondence_witness::row_for_candidate;
pub(crate) use crate::contracts::merge::data::artifacts::correspondence_witness::schema_declared_correspondence_posture;
pub use crate::contracts::merge::data::artifacts::correspondence_witness::RelationalMergeCorrespondenceWitness;
pub use crate::contracts::merge::data::artifacts::correspondence_witness_rows::{
    RelationalMergeCorrespondenceWitnessPosture, RelationalMergeCorrespondenceWitnessRow,
};
pub use crate::contracts::merge::data::artifacts::execution_authority_contract::{
    MergeExecutionAuthorityContract, MergeExecutionAuthorizationRule,
    MergeExecutionConsumptionRule, MergeExecutionDecisionSurface,
};
pub use crate::contracts::merge::data::artifacts::inspection_artifact::{
    RelationalMergeInspectionAdmission, RelationalMergeInspectionArtifact,
    RelationalMergeInspectionInput, RelationalMergeInspectionRow,
};
pub use crate::contracts::merge::data::artifacts::planning_artifact_core::{
    MergePlanningArtifactCore, MergePlanningSummary,
};
pub use crate::contracts::merge::data::artifacts::planning_digest_basis::{
    MergeArtifactDigestBasis, MergeBaseDigestBasis, MergeCausalDigestBasis,
    MergeConflictDigestBasis, MergeIdentityDigestBasis, MergeLoweredPlanDigestBasis,
    MergePolicyDigestBasis, MergeRequestDigestBasis,
};
pub use crate::contracts::merge::data::artifacts::proof_packet::{
    RelationalMergeAdmittedSurfaceRow, RelationalMergeProofPacket,
    RelationalMergeProofPacketAdmissionPosture,
};
pub use crate::contracts::merge::data::artifacts::proof_packet_canonical::RelationalMergeProofPacketCanonicalBasis;
pub use crate::contracts::merge::data::artifacts::schema_reconciliation_witness::RelationalSchemaReconciliationWitness;
pub use crate::contracts::merge::data::artifacts::schema_reconciliation_witness_rows::RelationalSchemaReconciliationWitnessRowInput;
pub use crate::contracts::merge::data::artifacts::schema_reconciliation_witness_rows::{
    RelationalSchemaReconciliationBasisRow, RelationalSchemaReconciliationCorrespondenceLinkRow,
    RelationalSchemaReconciliationWitnessDenial, RelationalSchemaReconciliationWitnessPosture,
    RelationalSchemaReconciliationWitnessRow,
};
pub use crate::contracts::merge::data::artifacts::schema_snapshot_digest_basis::{
    MergeSchemaKindClass, MergeSchemaKindSemanticSnapshot, MergeSchemaSnapshotDigestBasis,
};
pub use crate::contracts::merge::data::artifacts::strategy_witness::RelationalMergeStrategyWitness;
pub use crate::contracts::merge::data::artifacts::strategy_witness_policy_rows::RelationalMergeAspectPolicyWitnessRow;
pub use crate::contracts::merge::data::artifacts::strategy_witness_posture_rows::{
    RelationalMergeDeletionStrategyWitnessRow, RelationalMergeTopologyStrategyWitnessRow,
};
pub use crate::contracts::merge::data::causal::{
    BranchCausalDot, CausalAnnotationSummary, CausalFrontier, CommitCausalMetadata,
    CommitCausalRelation, MergeCausalEvidenceModel, MergeRecordCausalAnnotation,
    MergeRecordCausalDisposition,
};
pub use crate::contracts::merge::data::conflicts::{
    AspectConflictEvidence, ConflictClassificationSummary, DeletionMergeClass,
    EndpointContinuityClass, MergeConflictClass, MergeConflictClassification,
    MergeVisibilityEvidence, MergeVisibilityEvidenceKind, MergeVisibilityState,
    RelationConflictEvidence, RelationConflictPropagation, RelationContinuityClass,
    StrategyConflictClass, StrategyConflictEvidence, TopologyRegionConflictReason,
};
pub use crate::contracts::merge::data::decisions::{
    MergePlanningDecisionKind, MergePlanningDecisionLog, MergePlanningDecisionLogDigestBasis,
    MergePlanningDecisionRecord,
};
pub(crate) use crate::contracts::merge::data::execution::canonical_digest::compiled_executable_plan_digest;
pub(crate) use crate::contracts::merge::data::execution::canonical_digest::equality_witness_digest;
pub use crate::contracts::merge::data::execution::canonical_digest::schema_snapshot_digest;
pub(crate) use crate::contracts::merge::data::execution::executable_plan::{
    bound_parent_order, visible_record_snapshot,
};
pub use crate::contracts::merge::data::execution::executable_plan::{
    AdoptSourceRecordPlan, BoundExecutableMergePlan, BoundExecutableMergeRecordPlan,
    ConvergeDeletedOnBothSidesRecordPlan, DeletedOnBothSidesSemantics, ExecutableAspectPlan,
    MergeExecutableRecordProvenance, MergeLineageContinuityVerdict, PreserveSharedRecordPlan,
    ReconcileRecordPlan, ReconciledIdentityBasis, SharedTruthWitness, VisibleMergeRecordSnapshot,
};
pub(crate) use crate::contracts::merge::data::execution::materialized_aspect_values::{
    aspect_reference, materialized_value_aspect_key,
};
pub use crate::contracts::merge::data::execution::materialized_aspect_values::{
    MaterializedAspectValue, MaterializedAspectValueEvidence, MergeValueMaterialization,
    MergeValueSourceSide,
};
pub use crate::contracts::merge::data::execution::{
    MergeExecutionAuthorityBinding, MergeExecutionCompilationError, MergeExecutionDeniedRecord,
    MergeExecutionError, MergeExecutionFreshnessPolicy, MergeExecutionMutationPlanError,
    MergeExecutionPreparationError, MergeExecutionReadinessReport, RuntimeInstanceId,
};
pub(crate) use crate::contracts::merge::data::execution_artifacts::diagnostics_plan_from_record_plans;
pub use crate::contracts::merge::data::execution_artifacts::{
    ExecutedMergeAspectClass, ExecutedMergeAspectDiagnosticRow, ExecutedMergeRecordClass,
    ExecutedMergeRecordDiagnosticRow, MergeExecutionDiagnosticsPlan,
};
pub use crate::contracts::merge::data::identity::custom_name::IdentityBasisName;
pub use crate::contracts::merge::data::identity::declared_keys::DeclaredIdentityKeys;
pub use crate::contracts::merge::data::identity::{
    CustomIdentityBasisIdentity, IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope,
    IdentityDiscoverySummary, IdentityMatchCandidate, IdentityMatchClass, IdentityResolutionReason,
    MergeRecordIdentity, SchemaDeclaredCorrespondenceValidationSummary,
};
pub use crate::contracts::merge::data::planning_denial::MergePlanningError;
pub use crate::contracts::merge::data::policy::custom_name::MergePolicyName;
pub use crate::contracts::merge::data::policy::{
    AspectComparisonState, AspectMergePolicyDeclaration, AspectMergePolicyKind,
    AspectPolicyResolutionRecord, AuthorizedAspectValueSurface, AuthorizedAspectValueUsage,
    CustomMergePolicyIdentity, DeletionExecutionClass, LoweredAspectAction,
    LoweredAspectDenialIntent, LoweredAspectExecutionIntent, LoweredAspectOutcome,
    LoweredMergeAction, LoweredMergeBlockedReason, LoweredMergePlanRecord, LoweredMergePlanSummary,
    LoweredMergeRejectedReason, LoweredRecordDecision, LoweredRecordDecisionKind,
    LoweredRecordDenialAspectIntent, LoweredRecordDenialBundle, LoweredRecordDenialKind,
    LoweredRecordExecutionAspectIntent, LoweredRecordExecutionBundle,
    LoweredRecordExecutionIntentKind, MergeExecutableClass, MergeExecutionReadiness,
    MergeManualResolutionClass, MergePolicyDecisionBoundary, MergePolicyOwnershipClass,
    MergePolicyOwnershipSurface, MergePolicyProofBoundary, MergePolicyRejectClass,
    MergePolicyResolution, MergePolicyResolutionRecord, MergePolicyResolutionSummary,
    MergeResolutionClass, ResolvedAspectMergePolicy, TopologyExecutionClass,
    TopologyRewireAdmissionPolicy,
};
pub use crate::contracts::merge::data::resolved_value_strategy::MergeResolvedAspectValueStrategy;
pub use plans::BranchCommitDelta;
pub use plans::BranchTouchedRecordDelta;
pub use plans::CausallyAnnotatedMergePlan;
pub use plans::ConflictClassifiedMergePlan;
pub use plans::HistoryScopedMergePlan;
pub use plans::IdentityScopedMergePlan;
pub use plans::LoweredMergePlan;
pub use plans::PolicyResolvedMergePlan;
pub(crate) use plans::ValidatedSchemaDeclaredCorrespondence;
pub use requests::{
    MergeExecutionRequest, MergeIntent, MergePlanningRequest, NormalizedRelationalMergeRequest,
    OwnerBoundMergeExecutionRequest, OwnerBoundMergePlanningRequest,
    RelationalFoundationalMergeRequest, RelationalMergeCorrespondencePosture,
    RelationalMergeRequestBindingDenial, RelationalMergeRequestFamily,
    RelationalMergeRequestNormalizationDenial, RelationalMergeSchemaReconciliationPosture,
    RelationalMergeScope, RelationalMergeTopologyIntent,
};

pub mod prepared_execution;
pub use crate::contracts::merge::data::visible_record::VisibleMergeRecord;
pub(crate) use crate::contracts::merge::data::visible_record::VisibleMergeRecordKind;
pub use prepared_execution::CompiledMergeExecution;
pub use prepared_execution::ExecutionReadyLoweredMergePlan;
pub use prepared_execution::PreparedMergeExecution;
pub use prepared_execution::PreparedMergeMutationPlan;
