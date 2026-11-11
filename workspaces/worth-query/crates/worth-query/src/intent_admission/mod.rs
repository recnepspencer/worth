use crate::runtime::{WorthQueryIntentSourceLane, WorthQueryRuntimeFacadeFamily};

mod certification;
mod decisions;
pub(crate) mod dx;
mod eligibility;
mod families;
mod handoffs;
mod inventory;
mod plans;
mod stops;
mod support;
mod surface_catalog;
mod trace;

pub(crate) use certification::{certification_bridge, certification_runtime};
pub use certification::{
    certify_intent_admission, worth_query_intent_admission_certification_output_manifest,
    worth_query_intent_admission_closeout_extension_outputs,
    worth_query_intent_admission_legacy_parity_report, worth_query_intent_admission_oracle_report,
    worth_query_intent_admission_representative_family_report,
    worth_query_intent_admission_representative_output_report,
    worth_query_intent_admission_required_certification_outputs,
    worth_query_intent_admission_seeded_certification_report,
    worth_query_intent_admission_slope_report,
    worth_query_intent_admission_support_traceability_report,
    WorthQueryIntentAdmissionCertificationBundle,
    WorthQueryIntentAdmissionCertificationCounterSnapshot,
    WorthQueryIntentAdmissionCertificationOutput, WorthQueryIntentAdmissionLegacyParityCheck,
    WorthQueryIntentAdmissionLegacyParityLane, WorthQueryIntentAdmissionLegacyParityReport,
    WorthQueryIntentAdmissionLegacyParityRow, WorthQueryIntentAdmissionOracleComparisonRow,
    WorthQueryIntentAdmissionOracleLane, WorthQueryIntentAdmissionOracleManifestRow,
    WorthQueryIntentAdmissionOracleReport, WorthQueryIntentAdmissionProofShapeAudit,
    WorthQueryIntentAdmissionPublicBoundaryAudit,
    WorthQueryIntentAdmissionRepresentativeFamilyLane,
    WorthQueryIntentAdmissionRepresentativeFamilyReport,
    WorthQueryIntentAdmissionRepresentativeFamilyRow,
    WorthQueryIntentAdmissionRepresentativeOutputReport,
    WorthQueryIntentAdmissionSeedGeneratorClass, WorthQueryIntentAdmissionSeedReplayRow,
    WorthQueryIntentAdmissionSeededCertificationReport, WorthQueryIntentAdmissionSlopeLane,
    WorthQueryIntentAdmissionSlopeReport, WorthQueryIntentAdmissionSupportTraceabilityReport,
    WorthQueryIntentAdmissionSupportTraceabilityRow, WorthQueryIntentAdmissionTopologyAudit,
    WorthQueryIntentAdmissionTopologyAuditRow, WorthQueryIntentAdmissionTopologyDomain,
    WorthQueryIntentAdmissionWidthRunRow, WorthQueryIntentAdmissionWidthRunScale,
};
pub(crate) use certification::{
    INTENT_ADMISSION_CERTIFICATION_CHILD_MODULES, INTENT_ADMISSION_CERTIFICATION_EXPORTED_SURFACE,
    INTENT_ADMISSION_CERTIFICATION_MODULE_ROOT,
};
pub(crate) use decisions::{
    admit_runtime_intent_request, INTENT_ADMISSION_DECISIONS_CHILD_MODULES,
    INTENT_ADMISSION_DECISIONS_EXPORTED_SURFACE, INTENT_ADMISSION_DECISIONS_MODULE_ROOT,
};
pub use decisions::{
    WorthQueryIntentAdmissionDecision, WorthQueryIntentAdvisoryDecision,
    WorthQueryIntentViolationDecision,
};
pub use dx::{
    worth_query_basis_observation_intent, worth_query_projection_consumption_intent,
    WorthQueryAdmittedRuntimeEffectWriteIntent, WorthQueryAdmittedRuntimeExistingTruthProbeIntent,
    WorthQueryAdmittedRuntimeInspectionIntent, WorthQueryAdmittedRuntimeIntent,
    WorthQueryAdmittedWorkspaceReadIntent, WorthQueryBasisObservationAdmittedIntent,
    WorthQueryBasisObservationIntentAuthoring, WorthQueryBasisObservationIntentReview,
    WorthQueryProjectionConsumptionAdmittedIntent, WorthQueryProjectionConsumptionIntentAuthoring,
    WorthQueryProjectionConsumptionIntentReview, WorthQueryRuntimeEffectWriteIntentAdmissionReview,
    WorthQueryRuntimeEffectWriteIntentAuthoring,
    WorthQueryRuntimeExistingTruthProbeIntentAdmissionReview,
    WorthQueryRuntimeExistingTruthProbeIntentAuthoring,
    WorthQueryRuntimeInspectionIntentAdmissionReview, WorthQueryRuntimeInspectionIntentAuthoring,
    WorthQueryRuntimeIntentAdmissionReview, WorthQueryRuntimeIntentAuthoring,
    WorthQueryRuntimeWriteBatchIntentAuthoring, WorthQueryRuntimeWriteIntentAuthoring,
    WorthQueryWorkspaceDerivedInspectionIntentAdmissionReview,
    WorthQueryWorkspaceDerivedInspectionIntentAuthoring,
    WorthQueryWorkspaceDerivedMaterializationIntentAdmissionReview,
    WorthQueryWorkspaceDerivedMaterializationIntentAuthoring,
    WorthQueryWorkspaceLiveReadIntentAdmissionReview, WorthQueryWorkspaceLiveReadIntentAuthoring,
    WorthQueryWorkspaceReadIntentAdmissionReview, WorthQueryWorkspaceReadIntentAuthoring,
};
pub(crate) use dx::{
    INTENT_ADMISSION_DX_CHILD_MODULES, INTENT_ADMISSION_DX_EXPORTED_SURFACE,
    INTENT_ADMISSION_DX_MODULE_ROOT,
};
pub(crate) use eligibility::{
    authoritative_mutation_input_identity, INTENT_ADMISSION_ELIGIBILITY_CHILD_MODULES,
    INTENT_ADMISSION_ELIGIBILITY_EXPORTED_SURFACE, INTENT_ADMISSION_ELIGIBILITY_MODULE_ROOT,
};
pub use eligibility::{
    WorthQueryAuthoritativeMutationBatchIntentSeed, WorthQueryAuthoritativeMutationIntentSeed,
    WorthQueryAuthoritativeMutationPreflight, WorthQueryDerivedViewIntentSeed,
    WorthQueryExistingTruthProbeIntentSeed, WorthQueryExistingTruthProbeRoutingPreflight,
    WorthQueryGenericInspectionIntentSeed, WorthQueryGenericInspectionIntentTarget,
    WorthQueryGenericInspectionIntentTargetSeed, WorthQueryGenericInspectionRequestLabel,
    WorthQueryIntentAdmissionAuthorityLaneEligibility, WorthQueryIntentAdmissionBasisEligibility,
    WorthQueryIntentAdmissionCapabilityEligibility, WorthQueryIntentAdmissionEligibility,
    WorthQueryIntentAdmissionInvariantEligibility, WorthQueryIntentAdmissionPolicyEligibility,
    WorthQueryIntentAdmissionPreDecisionPosture,
    WorthQueryIntentAdmissionProjectionSourceEligibility,
    WorthQueryIntentAdmissionRoutingSupportEligibility,
    WorthQueryIntentAdmissionSourceLaneEligibility, WorthQueryIntentAdmissionSupportEligibility,
    WorthQueryLiveReadIntentSeed, WorthQueryRawIntentAdmissionRequest,
    WorthQueryReadExecutionIntentSeed,
};
pub use families::{
    worth_query_intent_admission_family_inventory, WorthQueryIntentAdmissionFamily,
    WorthQueryIntentAdmissionFamilyInventory, WorthQueryIntentAdmissionFamilyInventoryRow,
};
pub(crate) use families::{
    INTENT_ADMISSION_FAMILIES_CHILD_MODULES, INTENT_ADMISSION_FAMILIES_EXPORTED_SURFACE,
    INTENT_ADMISSION_FAMILIES_MODULE_ROOT,
};
pub(crate) use handoffs::{admit_authoritative_execution, admit_effect_execution};
pub use handoffs::{
    WorthQueryAdmittedIntentExecutionHandoff, WorthQueryAuthoritativeIntentExecutionBinding,
    WorthQueryAuthoritativeIntentExecutionHandoff,
    WorthQueryAuthoritativeMutationBatchExecutionBinding,
    WorthQueryAuthoritativeMutationBatchExecutionHandoff,
    WorthQueryAuthoritativeMutationExecutionBinding,
    WorthQueryAuthoritativeMutationExecutionHandoff, WorthQueryDerivedInspectionExecutionBinding,
    WorthQueryDerivedInspectionExecutionHandoff, WorthQueryDerivedMaterializationExecutionBinding,
    WorthQueryDerivedMaterializationExecutionHandoff,
    WorthQueryEffectTriggeredIntentExecutionBinding,
    WorthQueryEffectTriggeredIntentExecutionHandoff, WorthQueryExistingTruthProbeExecutionBinding,
    WorthQueryExistingTruthProbeExecutionHandoff, WorthQueryLiveReadExecutionBinding,
    WorthQueryLiveReadExecutionHandoff, WorthQueryReadExecutionBinding,
    WorthQueryReadExecutionHandoff, WorthQueryUnifiedInspectionExecutionBinding,
    WorthQueryUnifiedInspectionExecutionHandoff,
};
pub(crate) use handoffs::{
    INTENT_ADMISSION_HANDOFFS_CHILD_MODULES, INTENT_ADMISSION_HANDOFFS_EXPORTED_SURFACE,
    INTENT_ADMISSION_HANDOFFS_MODULE_ROOT,
};
pub use inventory::{
    worth_query_intent_admission_coverage_inventory, worth_query_intent_admission_mutation_audit,
    WorthQueryIntentAdmissionCoverageInventory, WorthQueryIntentAdmissionCoverageRow,
    WorthQueryIntentAdmissionCoverageStatus, WorthQueryIntentAdmissionCoveredEntrypoint,
    WorthQueryIntentAdmissionDecisionClass, WorthQueryIntentAdmissionEligibilityAuthority,
    WorthQueryIntentAdmissionExecutionBoundary, WorthQueryIntentAdmissionExecutionHandoffInventory,
    WorthQueryIntentAdmissionExecutionSeam, WorthQueryIntentAdmissionMutationAudit,
    WorthQueryIntentAdmissionMutationAuditRow, WorthQueryIntentAdmissionMutationProofCase,
    WorthQueryIntentAdmissionPlanKind, WorthQueryIntentAdmissionResultArtifact,
    WorthQueryIntentAdmissionSurfaceDescriptor,
};
pub use plans::{
    WorthQueryAdmittedIntentPlan, WorthQueryAuthoritativeIntentExecutionPlan,
    WorthQueryAuthoritativeMutationBatchExecutionPlan,
    WorthQueryAuthoritativeMutationExecutionPlan, WorthQueryBasisObservationPlan,
    WorthQueryDerivedInspectionExecutionPlan, WorthQueryDerivedMaterializationExecutionPlan,
    WorthQueryEffectTriggeredIntentExecutionPlan, WorthQueryExistingTruthProbeExecutionPlan,
    WorthQueryLiveReadExecutionPlan, WorthQueryProjectionConsumptionPlan,
    WorthQueryReadExecutionPlan, WorthQueryUnifiedInspectionExecutionPlan,
};
pub use stops::{
    WorthQueryIntentAdvisoryStop, WorthQueryIntentNonAdmittedStop, WorthQueryIntentViolationStop,
};
pub use support::{
    worth_query_intent_admission_support_matrix, WorthQueryIntentAdmissionSupportDetail,
    WorthQueryIntentAdmissionSupportMatrix, WorthQueryIntentAdmissionSupportPosture,
    WorthQueryIntentAdmissionSupportRow,
};
pub(crate) use support::{
    INTENT_ADMISSION_SUPPORT_CHILD_MODULES, INTENT_ADMISSION_SUPPORT_EXPORTED_SURFACE,
    INTENT_ADMISSION_SUPPORT_MODULE_ROOT,
};
pub use trace::{
    WorthQueryIntentDecisionTraceEnvelope, WorthQueryIntentDecisionTraceEnvelopeKind,
    WorthQueryIntentDecisionTraceEvidence, WorthQueryIntentDecisionTraceEvidenceOwner,
    WorthQueryIntentDecisionTraceRow, WorthQueryIntentDecisionTraceStage,
    WorthQueryIntentEligibilityTraceEvidence,
};
pub(crate) use trace::{
    INTENT_ADMISSION_TRACE_CHILD_MODULES, INTENT_ADMISSION_TRACE_EXPORTED_SURFACE,
    INTENT_ADMISSION_TRACE_MODULE_ROOT,
};

pub(crate) fn intent_runtime_facade_family(
    source_lane: WorthQueryIntentSourceLane,
) -> WorthQueryRuntimeFacadeFamily {
    match source_lane {
        WorthQueryIntentSourceLane::EffectTriggered => WorthQueryRuntimeFacadeFamily::Effect,
        WorthQueryIntentSourceLane::UserAuthored
        | WorthQueryIntentSourceLane::PreviewLocal
        | WorthQueryIntentSourceLane::BranchLocal
        | WorthQueryIntentSourceLane::DerivedRuntime => WorthQueryRuntimeFacadeFamily::Intent,
    }
}

pub(crate) fn intent_family_for_entrypoint(
    entrypoint: WorthQueryIntentAdmissionCoveredEntrypoint,
) -> WorthQueryIntentAdmissionFamily {
    match entrypoint {
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteIntent => {
            WorthQueryIntentAdmissionFamily::AuthoritativeUserIntent
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteNextEffectWriteIntent => {
            WorthQueryIntentAdmissionFamily::EffectTriggeredWriteIntent
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteScalarWrite => {
            WorthQueryIntentAdmissionFamily::AuthoritativeMutationIntent
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteBatchWrite => {
            WorthQueryIntentAdmissionFamily::AuthoritativeMutationIntent
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::BasisObservation => {
            WorthQueryIntentAdmissionFamily::BasisUseIntent
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ProjectionConsumption => {
            WorthQueryIntentAdmissionFamily::ProjectionConsumptionIntent
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteReadFamily
        | WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteReadFamilyInBasisContext
        | WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteLiveRead => {
            WorthQueryIntentAdmissionFamily::ReadExecutionIntent
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedMaterialization
        | WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedInspection
        | WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteUnifiedInspection
        | WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteInspectionNeighborDeferred => {
            WorthQueryIntentAdmissionFamily::InspectionMaterializationIntent
        }
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteExistingTruthProbeRouting => {
            WorthQueryIntentAdmissionFamily::LowerRuntimeCapabilityRoutingIntent
        }
    }
}
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
