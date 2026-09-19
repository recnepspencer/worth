mod admission;
mod authoring_basis;
mod batch;
mod batch_admission;
mod batch_artifact_metrics;
mod batch_execution;
mod certification;
mod counter_profiles;
mod counters;
mod diagnostics;
mod eligibility;
mod envelope;
mod exact_snapshot_closeout;
mod execution;
mod execution_artifacts;
mod execution_bridge;
mod execution_control_stop;
mod execution_deferred_kind;
mod execution_relational_batch;
mod execution_relational_scalar;
mod intent;
mod inventory;
mod inventory_rows;
mod lowering;
mod normalized;
mod oracle;
mod planning;
mod receipt;
mod receipt_snapshot_release;
mod receipt_transitions;
mod relational_execution_deferred;
mod relational_execution_failure;
mod settlement_repair;
mod support_contract;
mod support_matrix;
mod support_matrix_rows;
mod taxonomy;

pub use admission::{admit_effect_intent, evaluate_effect_eligibility};
pub use authoring_basis::EffectAuthoringBasis;
pub use batch::{
    LoweredEffectBatchExecutionArtifact, LoweredEffectBatchExecutionPlan,
    LoweredRelationalMutationBatchExecutionArtifact,
};
#[cfg(test)]
pub(crate) use batch_admission::admit_effect_batch_components;
pub use batch_admission::{
    effect_batch, AdmittedEffectBatch, EffectBatchAdmissionDenial, EffectBatchAdmissionDenialKind,
    EffectBatchIntentDraft, EffectBatchIntentDraftWithBasis,
};
pub use batch_execution::{
    EffectBatchExecutionControlStopped, EffectBatchExecutionDeferred, EffectBatchExecutionDenial,
    EffectBatchExecutionDenialKind, EffectBatchExecutionStop, EffectBatchSettlementDeferred,
};
pub use certification::{
    certify_effect_execution_pipeline, EffectExecutionCertificationBundle,
    EffectExecutionCertificationLane, EffectExecutionCertificationOutputDigest,
    EffectExecutionCertificationRow,
};
pub use counters::EffectLifecycleCounters;
pub use diagnostics::{EffectDiagnosticsMaterialization, EffectDiagnosticsRequest};
pub use eligibility::{
    AdmittedEffectIntent, AdvisoryEffectEligibility, DeferredEffectEligibility,
    DeniedEffectEligibility, EffectEligibility, EffectEligibilityDecisionTrace,
    EffectEligibilityOutcome, RebindRequiredEffectEligibility,
};
pub use envelope::{
    EffectEnvelopePrimaryResult, EffectEnvelopeSourceRefs, SelfDescribingEffectEnvelope,
};
pub use execution::{
    EffectExecutionAuthority, EffectExecutionControlStopped, EffectExecutionDeferred,
    EffectExecutionDenial, EffectExecutionDenialKind, EffectExecutionSettlementDeferred,
    EffectExecutionStop, ExecutedEffectAuthorityArtifact, ExecutedEffectPlan,
};
pub use execution_control_stop::EffectExecutionControlStopKind;
pub use execution_deferred_kind::EffectExecutionDeferredKind;
pub use intent::{normalize_raw_effect_intent, RawEffectIntent};
pub use inventory::{
    effect_lifecycle_family_inventory, effect_lifecycle_family_row_for_key,
    effect_lifecycle_public_surface_inventory, effect_lifecycle_support_row_matches_inventory,
    effect_lifecycle_supported_basis_families, EffectLifecycleFamilyInventory,
    EffectLifecycleFamilyInventoryRow, EffectLifecycleFamilyKey,
    EffectLifecyclePublicSurfaceInventory, EffectLifecyclePublicSurfaceRow,
    EffectLoweredArtifactKind, EffectPublicSurfaceAvailability, EffectPublicSurfaceKind,
    EffectReceiptArtifactKind,
};
pub use lowering::{
    EffectLoweringDenial, EffectLoweringDenialKind, LoweredEffectExecutionArtifact,
    LoweredEffectExecutionPlan,
};
pub use normalized::{EffectIntentDenial, EffectOperationInput, NormalizedEffectIntent};
#[cfg(test)]
pub(crate) use oracle::bridge_observation_execution_receipt_subject_identity;
pub use oracle::{
    bridge_observation_execution_record_subject_identity,
    bridge_observation_outcome_subject_identity, bridge_observation_receipt_subject_identity,
    bridge_observation_request_subject_identity, BridgeExecutionOracle, EffectExecutionOracleError,
    EffectExecutionOracleErrorKind, EffectExecutionOracleVerification,
    EffectExecutionOracleVerificationKind, RelationalExecutionOracle,
};
pub use planning::{
    scope_admitted_effect_plan, AuthorityScopedEffectPlan, EffectArtifactPolicy,
    EffectAuthorityOwner, EffectConflictFootprint, EffectInvariantScope,
    EffectPermittedLoweringFamily, EffectPolicyPosture, EffectPreviewPosture,
    EffectStrategyIdentityTarget,
};
pub use receipt::{
    EffectExecutionReceipt, EffectReceiptDecisionTrace, EffectReceiptIntegrityMarkers,
    EffectReceiptTargetEvidence,
};
pub use receipt_transitions::{
    EffectReceiptTransitionKind, EffectReceiptTransitionPosture, EffectReceiptTransitionRule,
    EffectReceiptTransitionRules,
};
pub use relational_execution_failure::{
    RelationalEffectExecutionFailure, RelationalEffectSettlementDeferred,
};
pub use settlement_repair::EffectSettlementRepairError;
pub use support_contract::{
    EffectDeferredNeighborFamily, EffectDeferredResiduePosture, EffectDeferredSupportContract,
};
pub use support_matrix::{
    discover_effect_lifecycle_support, effect_lifecycle_support_matrix,
    EffectLifecycleSupportDiscovery, EffectLifecycleSupportMatrix, EffectLifecycleSupportRow,
    EffectSupportCause, EffectSupportPosture,
};
pub use taxonomy::{
    DeniedEffectEligibilityKind, EffectAuthorityLane, EffectFamily, EffectIntentDenialKind,
};

pub(crate) use execution::execute_lowered_merge;
pub(crate) use execution_bridge::execute_lowered_writeback;

#[cfg(test)]
mod tests;
