mod configuration;
mod execution;
mod identity;
mod planning;
mod scope;
#[cfg(test)]
mod tests;

pub use configuration::{
    UiChangeProfile, UiRebindBudgetInput, UiRebindConcurrencyInput, UiRebindLimit, UiRebindProfile,
    UiRebindProfileConstructionDenial,
};
pub(crate) use execution::{admit_plan, UiRebindFinalAdmissionBasis};
pub(crate) use execution::{
    UiDetachedRebindCompletion, UiDetachedRebindRecovery, UiDetachedRebindRetry,
    UiRebindComparisonReservation, UiRebindReservation, UiRebindRuntimeState,
};
pub use execution::{
    UiDuplicateObservationReceipt, UiEffectingRebind, UiEffectingRebindCompletion,
    UiPreparedRebind, UiPreparedRebindPosture, UiProjectionRebindRequest,
    UiRebindCancellationReceipt, UiRebindCompletionHandle, UiRebindDenialCause,
    UiRebindDenialReceipt, UiRebindDisposition, UiRebindExecutionRequest,
    UiRebindInternalDefectKind, UiRebindInternalDefectOutcome, UiRebindOutcome,
    UiRebindPreparationDenial, UiRebindReceipt, UiRebindReconciliation,
    UiRebindReconciliationRequest, UiRebindRecoveryCompletionHandle, UiRebindRecoveryDenial,
    UiRebindRecoveryDenialCause, UiRebindRecoveryHandle, UiRebindRecoveryInternalDefect,
    UiRebindRecoveryInternalDefectKind, UiRebindRecoveryOutcome, UiRebindRecoveryReceipt,
    UiRebindRecoverySurfaceDenial, UiRebindReservationDenial, UiRebindShutdownReport,
    UiRebindStoppedPhase, UiRebindSupersededReceipt, UiRebindTimeoutReceipt,
    UiRebindValidNextAction, UiSourceRebindRequest,
};
#[cfg(any(test, feature = "certification-support"))]
pub(crate) use identity::decision_from_transition;
pub use identity::{
    UiIdentityLifecycleDecision, UiIdentityLifecycleDenial, UiIdentityLifecycleEntry,
    UiResolvedIdentityLifecycle,
};
pub(crate) use identity::{UiIdentityLifecycleResolver, UiSourceIdentityLifecycleIndex};
pub(crate) use planning::{
    UiAuthoredContentRebindSemanticProof, UiChangedRebindSemanticProof,
    UiProjectionSchemaTransitionInput, UiRebindPlanCompiler, UiRebindPlanningContext,
    UiRebindPlanningRecoveryStop, UiRebindSemanticProof,
};
pub use planning::{
    UiProjectionPredecessorValuePolicy, UiProjectionSchemaRequirement,
    UiProjectionSchemaTransition, UiProjectionSchemaTransitionKind, UiRebindArtifactPolicy,
    UiRebindCancellationPolicy, UiRebindCancellationRequest, UiRebindCandidatePreparationDenial,
    UiRebindConflictFootprint, UiRebindDeadlinePolicy, UiRebindDeclarativeEffect,
    UiRebindDisclosurePolicy, UiRebindEffectSet, UiRebindExecutionPolicy, UiRebindIdempotency,
    UiRebindParallelAdmission, UiRebindPlan, UiRebindPlanBasis, UiRebindPlanCost,
    UiRebindPlanTarget, UiRebindPlanningDenial, UiRebindResourceAccess, UiRebindRetryTolerance,
    UiRebindSafePoint, UiRebindSafePointPolicy, UiRebindSessionDeadline, UiRebindSubsystemKind,
    UiRebindSubsystemPlan,
};
pub(crate) use scope::UiAffectedScopeResolver;
pub use scope::{
    UiAffectedConsumer, UiAffectedFactLookup, UiAffectedScopeBasis, UiAffectedScopeCost,
    UiAffectedScopeDenial, UiAffectedScopeGeneration, UiResolvedAffectedScope,
};
