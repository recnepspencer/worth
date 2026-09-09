//! Progressive ordinary surface for runtime-installed operations.
//!
//! The root teaches only world → typed family → bound operation. Advanced
//! transitions remain discoverable by the guarantee they govern. Package
//! construction, provider registration, replay, raw Foundational carriers, and
//! generic proof machinery are deliberately absent.

pub use crate::domain_installation::{
    WorthQueryBoundDomainOperation, WorthQueryBranchHeadIdentity,
    WorthQueryBranchHeadIdentityError, WorthQueryInstalledOperatingWorld,
    WorthQueryOperatingWorldEntryDenial, WorthQueryOperatingWorldEntryDenialKind,
    WorthQueryOperatingWorldProductDenial, WorthQueryOperationBindingDenial,
    WorthQueryOperationBindingDenialKind, WorthQueryOperationFamilyView,
};

pub mod transition {
    pub use super::super::installed_transitions::{
        collection_capability, collection_window_admission, collection_window_resolution,
        consumption, execution, publication, resource_admission, settlement,
        WorthQueryCollectionCapabilityTransition, WorthQueryCollectionWindowTransition,
        WorthQueryConsumptionTransition, WorthQueryExecutionTransition,
        WorthQueryPublicationTransition, WorthQueryResourceAdmissionStop,
        WorthQueryResourceAdmissionTransition, WorthQuerySettlementTransition,
    };
}

pub mod operation {
    pub use crate::domain_installation::{
        WorthQueryAdmittedDirectOperation, WorthQueryAdmittedExecutionResourcePlan,
        WorthQueryAdmittedWorkflowOperation, WorthQueryAdmittedWorkflowResourcePlan,
        WorthQueryBoundExecutionDenial, WorthQueryBoundExecutionDenialKind,
        WorthQueryBoundExecutionReceipt, WorthQueryBoundProjectionRequest,
        WorthQueryConsumedDomainProjection, WorthQueryConsumerAllocationPosture,
        WorthQueryConsumerBoundary, WorthQueryConsumerBoundaryRequirements,
        WorthQueryConsumerPresentationPosture, WorthQueryConsumerProjectionContract,
        WorthQueryConsumerProjectionContractDenial, WorthQueryConsumerSupportDimension,
        WorthQueryConsumerSupportPosture, WorthQueryDeferredDomainOperation,
        WorthQueryDerivedPublicationReceipt, WorthQueryDirectResourceAdmissionOutcome,
        WorthQueryExecutedDomainOperation, WorthQueryExecutionProviderSession,
        WorthQueryExecutionResourceAdmissionCounters, WorthQueryExecutionResourceAdmissionDenial,
        WorthQueryExecutionResourceAdmissionDenialKind,
        WorthQueryExecutionResourceAdmissionPosture, WorthQueryExecutionResourceAttemptEvidence,
        WorthQueryExecutionResourceSupport, WorthQueryExecutionResourceSupportSnapshot,
        WorthQueryNativeAccessCounters, WorthQueryNativeAccessDenial,
        WorthQueryNativeAccessDenialKind, WorthQueryNativeAccessKey, WorthQueryNativeFieldAccess,
        WorthQueryNativeProjectionRequestDenial, WorthQueryNativeProjectionRequestDenialKind,
        WorthQueryOperationExecutionCounters, WorthQueryOperationExecutionWarning,
        WorthQueryOperationLineageContract, WorthQueryOperationResultState,
        WorthQueryProgressionDenial, WorthQueryProjectionRequestBuilder,
        WorthQueryPublicationDenial, WorthQueryPublishedDomainOperation,
        WorthQuerySettledDomainProjection, WorthQueryWorkflowResourceAdmissionOutcome,
    };
    pub use crate::ordinary::read::project_facts;
    pub use worth_query_declaration::facade::domain_computation::{
        WorthQueryCancellationSafePointFamily, WorthQueryExecutionDegradation,
        WorthQueryExecutionMode, WorthQueryExecutionResourceRequest,
        WorthQueryPartialEffectPosture, WorthQueryResourceDimension,
        WorthQueryResourceLimitRequest, WorthQueryRetainedProgressPosture,
        WorthQuerySemanticScaleAxis, WorthQuerySemanticScaleRequest, WorthQueryYieldedStatePosture,
    };
}

pub mod observation {
    pub use crate::domain_installation::{
        WorthQueryAdmittedConsumerInvalidation, WorthQueryConsumerGranularMaintenanceStop,
        WorthQueryConsumerInvalidationAdmissionStop, WorthQueryConsumerInvalidationCause,
        WorthQueryConsumerInvalidationContinuation, WorthQueryConsumerInvalidationCounters,
        WorthQueryConsumerInvalidationDelta, WorthQueryConsumerInvalidationDeltaStop,
        WorthQueryConsumerInvalidationDeltaStopKind, WorthQueryConsumerInvalidationDisposition,
        WorthQueryConsumerInvalidationLocality, WorthQueryLiveBoundDomainProjection,
        WorthQueryProjectionLeaseAdmissionDenialKind, WorthQueryProjectionLeaseAdmissionOutcome,
        WorthQueryProjectionLeaseAdmissionStop, WorthQueryProjectionPromotionDenialKind,
        WorthQueryProjectionPromotionOutcome, WorthQueryProjectionPromotionStop,
        WorthQueryPublishedConsumerInvalidation, WorthQuerySharedLiveProjectionLease,
        WorthQuerySharedProjectionDelivery, WorthQuerySharedProjectionDisposalOutcome,
        WorthQuerySharedProjectionDisposalStop, WorthQuerySharedProjectionDrainStop,
    };
}

pub mod collection {
    pub use crate::domain_installation::{
        WorthQueryAdmittedCollectionWindow, WorthQueryBoundCollection,
        WorthQueryBoundCollectionWindow, WorthQueryCollectionCapabilityDenial,
        WorthQueryCollectionCapabilityOutcome, WorthQueryCollectionCapabilityStop,
        WorthQueryCollectionConsumerPreparationDenial, WorthQueryCollectionConsumerWindow,
        WorthQueryCollectionContinuation, WorthQueryCollectionCursor,
        WorthQueryCollectionDeliveryCounters, WorthQueryCollectionDeliveryDenial,
        WorthQueryCollectionDeliveryDenialKind, WorthQueryCollectionDeliveryOutcome,
        WorthQueryCollectionNativeAccessCounters, WorthQueryCollectionNativeFactAccess,
        WorthQueryCollectionPatch, WorthQueryCollectionPatchApplicationReceipt,
        WorthQueryCollectionPatchFact, WorthQueryCollectionPatchOperation,
        WorthQueryCollectionRowAccessDenial, WorthQueryCollectionRowHandle,
        WorthQueryCollectionWindowAdmissionOutcome, WorthQueryCollectionWindowBreadth,
        WorthQueryCollectionWindowBreadthDenial, WorthQueryCollectionWindowOutcome,
        WorthQueryCollectionWindowWarning,
    };
}

pub mod compatibility {
    pub use crate::domain_installation::{
        classify_owner_delivered_impact, WorthQueryArtifactReuseEquivalence,
        WorthQueryBasisCompatibilityDenial, WorthQueryBasisCompatibilityWitness,
        WorthQueryCompatibilityCounters, WorthQueryCompatibilityDenialKind,
        WorthQueryDependencyClosureReuseDenial, WorthQueryDependencyClosureReuseWitness,
        WorthQueryExecutionSharingDenial, WorthQueryExecutionSharingWitness, WorthQueryImpactClass,
        WorthQueryImpactCounters, WorthQueryImpactDecision,
        WorthQueryInvalidationCompatibilityOutcome, WorthQuerySameInstallationDenial,
        WorthQuerySameInstallationWitness,
    };
}

pub mod support {
    pub use crate::domain_installation::{
        WorthQueryConsumerSupportAdmissionCounters, WorthQueryConsumerSupportCompatibilityDenial,
        WorthQueryConsumerSupportDimension, WorthQueryConsumerSupportPosture,
    };
}

pub mod impact {
    pub use crate::domain_installation::{
        admit_current_invalidation_impact, admit_primary_runtime_granular_batch,
        admit_primary_runtime_granular_invalidations, select_invalidation_candidates,
        WorthQueryAdmittedInvalidationBatch, WorthQueryAdmittedInvalidationImpact,
        WorthQueryCompiledSemanticAspectDependency,
        WorthQueryCompiledSemanticAspectDependencyClosure,
        WorthQueryConditionalObservationEvidence, WorthQueryDependencyClosureReuseDenial,
        WorthQueryDependencyClosureReuseWitness, WorthQueryDependencyClosureSemanticComparison,
        WorthQueryGranularAdmissionCounters, WorthQueryImpactAdmissionDenial,
        WorthQueryImpactAdmissionDenialKind, WorthQueryImpactClass, WorthQueryImpactCounters,
        WorthQueryImpactDecision, WorthQueryInstalledInvalidationManifest,
        WorthQueryInvalidationCandidateSet, WorthQuerySemanticAspectDependencyCompilationCounters,
        WorthQuerySemanticAspectDependencyCompilationDenial,
        WorthQuerySemanticAspectDependencyCompilationDenialKind,
        WorthQuerySemanticAspectDependencyView, WorthQuerySemanticDependencyClosureEvidence,
        WorthQuerySemanticDependencyEdge, WorthQuerySemanticDependencyRole,
    };
}

pub mod invalidation {
    pub use crate::domain_installation::{
        admit_current_invalidation_impact, admit_primary_runtime_granular_batch,
        admit_primary_runtime_granular_invalidations, select_invalidation_candidates,
        WorthQueryAdmittedInvalidationBatch, WorthQueryAdmittedInvalidationImpact,
        WorthQueryInstalledInvalidationManifest, WorthQueryInvalidationCandidateSet,
    };
    pub use crate::live::{
        bind_primary_runtime_granular_invalidations,
        bind_shared_primary_runtime_granular_invalidations,
        maintain_granular_invalidation_deliveries, maintain_primary_runtime_granular_batch,
        maintain_primary_runtime_granular_collection_batch,
        maintain_primary_runtime_granular_invalidations,
        maintain_shared_primary_runtime_granular_batch,
        perform_prepared_shared_primary_runtime_granular_maintenance,
        prepare_shared_primary_runtime_granular_batch, WorthQueryCoalescedMaintenancePlan,
        WorthQueryGranularMaintenanceCounters, WorthQueryLivePublicationDenial,
        WorthQueryMaintenanceDenial, WorthQueryMaintenanceScope, WorthQueryMaintenanceStrategy,
        WorthQueryPerformedIndexedLivePatch, WorthQueryPerformedLiveMaintenanceWork,
        WorthQueryPerformedMaintenance, WorthQueryPerformedMaintenanceEffect,
        WorthQueryPerformedProjectionPatch, WorthQueryPreparedSharedPrimaryGranularMaintenance,
        WorthQueryPrimaryGranularMaintenanceDenial, WorthQueryPrimaryGranularMaintenanceOutcome,
        WorthQueryPrimaryGranularMaintenancePerformed, WorthQueryPrimaryRuntimeInvalidationBinding,
        WorthQueryPublishedLiveDelivery, WorthQueryPublishedSharedPrimaryInvalidation,
        WorthQuerySharedConsumerDeliveryAuthority, WorthQuerySharedConsumerDeliveryPolicy,
        WorthQuerySharedConsumerDeliveryPolicyAdmission,
        WorthQuerySharedPrimaryGranularMaintenanceDenial,
        WorthQuerySharedPrimaryGranularMaintenanceOutcome,
        WorthQuerySharedPrimaryGranularMaintenancePerformed,
        WorthQuerySharedPrimaryGranularSelectionOutcome,
    };
}

pub mod lineage {
    pub use crate::domain_installation::{
        WorthQueryDurableReferenceIntent, WorthQueryPersistentNameAdmission,
        WorthQueryPersistentNameDenial, WorthQueryPersistentNameIntent,
        WorthQueryPersistentNameOutcome, WorthQueryPersistentNameTarget,
        WorthQueryPromotedGraphIdentity, WorthQueryPromotionOnReferenceCapability,
        WorthQueryPromotionOnReferenceCounters, WorthQueryPromotionOnReferenceDenial,
        WorthQueryPromotionOnReferenceOutcome, WorthQueryTraceLineageCounters,
        WorthQueryTraceLineageEvidence, WorthQueryTraceLineageReport,
    };
}

pub mod inspection {
    pub use crate::domain_installation::{
        WorthQueryConsumptionCostExportDenial, WorthQueryConsumptionCostExportDenialKind,
        WorthQueryConsumptionCostRow, WorthQueryConsumptionCostSnapshot,
    };
}

pub mod conditional {
    pub use crate::domain_installation::{
        WorthQueryArtifactPosture, WorthQueryArtifactReuseEquivalence, WorthQueryComparatorFamily,
        WorthQueryComparatorRequirement, WorthQueryConditionalEvaluationCondition,
        WorthQueryConditionalGraphReadRole, WorthQueryConditionalNodeRole,
        WorthQueryConditionalOutcomeClass, WorthQueryConditionalProvenance,
        WorthQueryConditionalTrigger, WorthQueryDeltaComparisonDomain, WorthQueryDeltaThreshold,
        WorthQueryDomainConditionFamily, WorthQueryMaintenancePosture,
        WorthQueryOnDemandTriggerFamily, WorthQueryOperationProjectionRole,
        WorthQueryOutputEquivalenceRequirement, WorthQueryOutputRelationship,
        WorthQueryPortableConditionParameter, WorthQueryPortableConditionalNodeDeclaration,
        WorthQueryQuantityUnit, WorthQueryQuantityValueFamily, WorthQuerySemanticLocality,
        WorthQueryTemporalCondition, WorthQueryTemporalWake,
    };
}

pub mod workflow {
    pub use crate::domain_installation::WorthQueryReplayComparison as WorthQueryWorkflowTraceComparison;
    pub use crate::domain_installation::{
        compare_exact_workflow_traces, WorthQueryCompletedWorkflowTrace,
        WorthQueryConsumedWorkflowProjection, WorthQueryDeferredWorkflowStage,
        WorthQueryDeferredWorkflowStart, WorthQueryPublishedWorkflow,
        WorthQuerySettledWorkflowProjection, WorthQueryWorkflowAdvanceDenial,
        WorthQueryWorkflowCompletionDenial, WorthQueryWorkflowProjectionPromotionOutcome,
        WorthQueryWorkflowPublicationDenial, WorthQueryWorkflowReexecutionOutcome,
        WorthQueryWorkflowRun, WorthQueryWorkflowStageAttempt, WorthQueryWorkflowStageReceipt,
        WorthQueryWorkflowStartOutcome,
    };
}

pub mod recovery {
    pub use crate::domain_installation::{
        WorthQueryDomainRebindDenial, WorthQueryDomainRebindNextAction,
        WorthQueryDomainRebindReceipt, WorthQueryDomainRebindRequest,
        WorthQueryProjectionCancellationOutcome, WorthQueryProjectionDisposalOutcome,
        WorthQueryProjectionRebindOutcome, WorthQueryProjectionReplacementOutcome,
        WorthQueryRebindRequiredDomainProjection, WorthQueryReplacementDenial,
    };
}
