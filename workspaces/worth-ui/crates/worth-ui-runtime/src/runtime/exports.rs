//! Lifecycle-grouped runtime exports.

mod replacement;

pub use replacement::*;

// --- graph-owned allocation locality ---

// --- launch ---
pub use super::active::WorthUiActiveRuntimeObservation;
pub use super::allocation_frame_dispatch::{
    UiAdmittedAllocationStreamIngress, UiAllocationFrameDispatchDenial,
    UiAllocationFrameDispatcherCounters, UiAllocationFrameDuplicateWitness,
    UiAllocationFrameGatewayOutcome, UiAllocationFrameIngressDescriptor,
    UiAllocationFrameQueryWarningPosture, UiAllocationFrameSourceFact,
    UiAllocationFrameSourceFactPosture, UiAllocationFrameSubmissionOutcome,
    UiFrameworkTransitionExecutionDenial, UiFrameworkTransitionPlanningCounters,
    UiFrameworkTransitionPlanningDenial, WorthUiFrameworkTurn, WorthUiFrameworkTurnCompletion,
    WorthUiFrameworkTurnExecution, WorthUiInteractionTurnSource, WorthUiQueryFrameIngressCounters,
    WorthUiQueryFrameIngressDenial, WorthUiQueryFrameIngressOutcome,
    WorthUiQueryProjectionTurnSource,
};
pub(crate) use super::allocation_frame_dispatch::{
    UiAllocationFrameQueueDisposition, UiAllocationFrameReplacementTransition,
};
#[cfg(test)]
pub(crate) use super::allocation_frame_dispatch::{
    WorthUiDurableResizeSubmission, WorthUiHostMeasurementSubmission, WorthUiInteractionSubmission,
};
pub use super::allocation_frame_dispatch::{
    WorthUiMountedPreviewFollowOn, WorthUiPendingMountedPreviewProjection,
};
#[cfg(test)]
pub use super::launch::WorthUiRuntimeFrameworkLoop;
pub use super::launch::{
    WorthUiLastValidObservation, WorthUiPendingActivation, WorthUiRuntime,
    WorthUiRuntimeFrameEpoch, WorthUiRuntimeLaunch, WorthUiRuntimeLaunchDenial,
    WorthUiRuntimeLifecycle, WorthUiRuntimeShutdownBlocker, WorthUiRuntimeShutdownReceipt,
    WorthUiRuntimeShutdownRecovery,
};

// --- planning ---
#[cfg(test)]
pub use super::allocation_frame_dispatch::{
    UiAdmittedAllocationSourceOrder, UiAllocationFrameIngressIdentity,
    UiAllocationFrameIngressSequence, UiAllocationFramePauseReason,
    UiAllocationFrameSourceIdentity, UiAllocationFrameSourceLane,
};
pub use super::allocation_frame_dispatch::{
    UiAllocationFrameDispatcherState, UiAllocationFrameEpoch, UiAllocationFrameIngressKey,
    UiAllocationFrameSourceGeneration,
};
pub use super::allocation_receipt::{
    UiAllocationAnchorPosture, UiAllocationAuthoritySuccessionDenial, UiAllocationAxis,
    UiAllocationAxisAlignedBounds, UiAllocationCandidate, UiAllocationCounterName,
    UiAllocationDenialFamily, UiAllocationDurableSemanticState, UiAllocationEdgeReference,
    UiAllocationFreshnessConsumptionDenial, UiAllocationGeometryKnowledge,
    UiAllocationPreviewCandidate, UiAllocationReceipt, UiAllocationReceiptCommitDenial,
    UiAllocationReceiptCommitOutcome, UiAllocationReceiptDenialReport,
    UiAllocationReceiptEquivalenceBasis, UiAllocationReceiptFreshnessPosture,
    UiAllocationReceiptGeneration, UiAllocationReceiptIdentity, UiAllocationReceiptReport,
    UiAllocationReplanTransaction, UiAllocationReplanTransactionCommitDenial,
    UiAllocationReplanTransactionOutcome, UiAllocationReuseDenial, UiAllocationReuseVerdict,
    UiCommittedAllocationGeometryEvidence, UiCommittedAllocationLoweringInput,
    UiCommittedAllocationReplan, UiPortalAllocationCommitBindDenial,
    UiPreviewPaintIsolationOutcome, UiPreviewPaintIsolationViolation,
};
#[cfg(test)]
pub use super::allocation_receipt::{
    UiAllocationAuthorityCounter, UiCommittedAllocationEvidenceSet,
};
pub(crate) use super::invalidation_narrowing::UiAdmittedAllocationPlanReference;
pub use super::invalidation_narrowing::UiAdmittedPortalMovement;
pub use super::invalidation_narrowing::{
    UiAllocationActivationCatalogDenial, UiAllocationInvalidationNarrowingDenial,
    UiAllocationInvalidationNarrowingRejection, UiAllocationInvalidationTarget,
    UiNarrowedAllocationFramePlan, UiScrollCatalogSwapEvidence, UiScrollOwnerAcquisitionDenial,
    UiScrollOwnerCatalogDenialReport, UiScrollOwnerCatalogReceipt,
};
pub(crate) use super::planning::allocation_planning::WorthUiRetainedAllocationPlanningEvidenceRegistry;
pub use super::planning::allocation_planning::{
    WorthUiAllocationPlanning, WorthUiAllocationPlanningBasis, WorthUiAllocationPlanningCounters,
    WorthUiAllocationPlanningDenial, WorthUiAllocationPlanningDenialReason,
    WorthUiAllocationPlanningInspection,
};
pub use super::planning::execution_plan_input::{
    WorthUiComponentLoweringHook, WorthUiExecutionPlanInput, WorthUiPlanLoweringBasis,
    WorthUiPlanLoweringContext, WorthUiPlanLoweringCounters, WorthUiPlanLoweringDenial,
    WorthUiPlanLoweringDenialReason, WorthUiPlanNodeInput, WorthUiPlanNodeInputFamily,
    WorthUiPlanNodeTopologyInput,
};
pub use super::planning::plan_equivalence::{
    WorthUiExecutablePlanDecision, WorthUiExecutablePlanDecisionKind,
    WorthUiExecutablePlanEquivalenceDenial, WorthUiExecutionPlanDigest,
    WorthUiExecutionPlanEquivalence, WorthUiExecutionPlanEquivalenceBasis,
    WorthUiExecutionPlanEquivalenceCounters, WorthUiPlanEquivalenceEvidenceReference,
    WorthUiPlanEquivalenceSummary,
};
pub use super::planning::plan_inspection::{
    WorthUiArtifactToPlanProvenance, WorthUiExecutionPlanInspection, WorthUiLaneInspection,
    WorthUiPlanInspectionCounters, WorthUiPlanInspectionDenial, WorthUiPlanNodeInspection,
    WorthUiQueryInspectionLinks,
};
#[cfg(test)]
pub use super::planning::plan_inspection::{
    WorthUiPlanInspectionDenialReason, WorthUiPlanProvenanceSource,
};
pub use super::planning::plan_topology::{
    WorthUiExecutionPlan, WorthUiPlanChildRange, WorthUiPlanConstructionCounters,
    WorthUiPlanExecutionLane, WorthUiPlanLanePartition, WorthUiPlanLookupIndex, WorthUiPlanNode,
    WorthUiPlanNodeFamily, WorthUiPlanRegionIdentity, WorthUiPlanRegionStorageCounters,
    WorthUiPlanRegionStructure, WorthUiPlanRegionTransition, WorthUiPlanRegionalEvidence,
    WorthUiPlanTopology, WorthUiPlanTopologyCounters, WorthUiPlanTopologyDenial,
    WorthUiPlanTopologyDenialReason, WorthUiRenderResourceRef,
};
pub use super::planning::query_binding::WorthUiQuerySettledFactLink;
pub use super::portal::anchored_allocation::UiPortalActivationBindingDenial;
pub use super::portal::anchored_allocation::{
    UiAdmittedPortalAnchorObservation, UiPortalAllocationPlanningBasis, UiPortalAnchorIdentity,
    UiPortalAnchorIdentityTransition, UiPortalAnchorSuccessorDenial,
};
pub use super::stream_policy::{
    UiAllocationCadenceKind, UiAllocationDuplicatePosture, UiAllocationFrameCadenceVerdict,
    UiAllocationFramePlanIdentity, UiAllocationFrameRejection, UiAllocationFrameResolutionCounters,
    UiAllocationFrameResolutionDenial, UiAllocationIngressPolicyVerdict,
    UiAllocationIntermediatePolicyVerdict, UiAllocationInvalidationFamily,
    UiAllocationInvalidationIntent, UiAllocationInvalidationReferenceDenial,
    UiAllocationSourceOrderVerdict, UiAllocationStreamCompositionCounters,
    UiAllocationStreamCompositionDenial, UiAllocationStreamFamily, UiResolvedAllocationFramePlan,
    UiResolvedAllocationPolicyBranch, UiResolvedAllocationStreamPolicy,
};
pub(crate) use super::viewport_resize::UiViewportResizeCommitBasis;
pub use super::viewport_resize::{
    UiViewportReceiptCommitStrategy, UiViewportResizeDenial, UiViewportResizeOutcome,
};

// --- activation ---
pub use super::activation::activation_staging::{
    WorthUiActivationReadiness, WorthUiActivationStagingCounters, WorthUiActivationStagingDenial,
    WorthUiActivationStagingDenialReason, WorthUiActivationStagingReport, WorthUiStagedReplacement,
};
pub use super::activation::frame_activation_gate::{
    WorthUiActivationGateCounters, WorthUiActivationGateDenial, WorthUiActivationGateDenialReason,
    WorthUiActivationGateReceipt, WorthUiFrameBoundary,
};
#[cfg(test)]
pub use super::activation::UiCommittedAllocationActivationInspectionOutcome;
pub use super::activation::{
    UiCommittedAllocationActivationCounters, UiCommittedAllocationActivationDenial,
    UiCommittedAllocationActivationDenialReason, WorthUiAllocationCatalogActivationDenial,
    WorthUiNoOpProvenancePosture, WorthUiNoOpQueryPosture, WorthUiSemanticNoOpReceipt,
    WorthUiSemanticNoOpWork,
};
pub use super::activation::{WorthUiPlanSwapReceipt, WorthUiPriorValidPlanObservation};
pub use super::allocation_catalog_successor::{
    UiAllocationCatalogDeltaCounters, UiAllocationCatalogRowDisposition,
    UiAllocationCatalogRowTransition,
};

// --- execution ---
pub use super::execution::canvas_spatial_lane::{
    WorthUiCanvasSpatialCertification, WorthUiCanvasSpatialCounters,
    WorthUiCanvasSpatialFrameDenial, WorthUiCanvasSpatialFrameDenialReason,
    WorthUiCanvasSpatialFrameReceipt, WorthUiCanvasSpatialFrameTarget,
    WorthUiCanvasSpatialInspectionDenial, WorthUiCanvasSpatialLane, WorthUiCanvasSpatialNode,
    WorthUiCanvasSpatialPlan, WorthUiCanvasSpatialPlanAvailability, WorthUiCanvasSpatialPlanDenial,
    WorthUiCanvasSpatialPlanDenialReason, WorthUiCanvasSpatialTargetSummary,
    WorthUiCanvasViewportRequest, WorthUiCanvasViewportRequestDenial,
    WorthUiCanvasViewportRequestDenialReason, WorthUiSpatialHitTestRequest,
    WorthUiSpatialIndexStrategy, WorthUiSpatialViewportPoint,
};
pub use super::execution::handle_allocation::{
    WorthUiChildRangeHandle, WorthUiCommandHandle, WorthUiComponentHandle,
    WorthUiHandleArenaIdentity, WorthUiHandleCapacityExhaustion, WorthUiHandleResolutionEvidence,
    WorthUiHandleResolutionOutcome, WorthUiHandleSlotGeneration, WorthUiLaneHandle,
    WorthUiRuntimeHandle, WorthUiRuntimeHandleAllocation, WorthUiRuntimeHandleAllocationBasis,
    WorthUiRuntimeHandleAllocationCounters, WorthUiRuntimeHandleAllocationDenial,
    WorthUiRuntimeHandleAllocationDenialReason, WorthUiRuntimeHandleAllocationReceipt,
    WorthUiRuntimeHandleFamilyWidths, WorthUiRuntimeHandleLocator, WorthUiStateSlotHandle,
    WorthUiTokenHandle, WorthUiViewBindingHandle,
};
pub use super::execution::lane_admission::{
    WorthUiExecutionLane, WorthUiExecutionLaneDescriptor, WorthUiExecutionLaneSupport,
    WorthUiExtensionHookAdmission, WorthUiLaneAdapterHook, WorthUiLaneAdapterHookKind,
    WorthUiLaneAdmission, WorthUiLaneAdmissionCounters, WorthUiLaneAdmissionDenial,
    WorthUiLaneAdmissionDenialReason, WorthUiLaneCostRegime, WorthUiLaneFailureMode,
    WorthUiLaneSupportDiagnostic, WorthUiLaneSupportRow, WorthUiLaneSupportStatus,
    WorthUiQueryLaneFactLink, WorthUiUnsupportedHookDenial, WorthUiUnsupportedHookDenialReason,
};
pub use super::execution::lane_frame_cost_certification::{
    WorthUiBroadScanRegressionDenial, WorthUiFrameCostCertification,
    WorthUiLaneAndFrameCostCertification, WorthUiLaneCertification,
    WorthUiLaneFrameCostCertificationCounters, WorthUiLaneFrameCostCertificationDenial,
    WorthUiLaneFrameCostCertificationDenialReason, WorthUiLaneFrameCostCertificationScenario,
    WorthUiLaneFrameCostFoundationalReadiness, WorthUiLaneScaleVariationProof,
    WorthUiNoSourceFrameProof,
};
pub use super::execution::lane_meaning_parity::{
    WorthUiCrossLaneSemanticAuthority, WorthUiCrossLaneSemanticFamily,
    WorthUiCrossLaneSemanticReference, WorthUiLaneMeaningParity, WorthUiLaneParityCertification,
    WorthUiLaneParityCounters, WorthUiLaneParityDenial, WorthUiLaneParityDenialReason,
    WorthUiLaneParityReport, WorthUiLaneTransitionParity,
};
pub use super::execution::ordinary_lane::{
    WorthUiOrdinaryExecutionLane, WorthUiOrdinaryFrameTarget, WorthUiOrdinaryLaneCertification,
    WorthUiOrdinaryLaneCounters, WorthUiOrdinaryLaneFrameDenial,
    WorthUiOrdinaryLaneFrameDenialReason, WorthUiOrdinaryLaneFrameReceipt, WorthUiOrdinaryLaneNode,
    WorthUiOrdinaryLanePlan, WorthUiOrdinaryLanePlanDenial, WorthUiOrdinaryLanePlanDenialReason,
    WorthUiOrdinaryLaneTouchReceipt, WorthUiOrdinaryPlanAvailability, WorthUiOrdinaryPlanSummary,
    WorthUiOrdinaryPlanSummaryDenial, WorthUiOrdinaryPlanSummaryRequest,
    WorthUiOrdinarySummaryTarget, WorthUiOrdinaryTouchBreadth,
};
pub use super::execution::realtime_overlay_lane::{
    WorthUiHighFrequencyFramePolicy, WorthUiHighFrequencyFramePolicyDenial,
    WorthUiHighFrequencyFramePolicyDenialReason, WorthUiHudNode, WorthUiHudPlan,
    WorthUiHudPlanDenial, WorthUiHudPlanDenialReason, WorthUiRealtimeCertification,
    WorthUiRealtimeFrameDenial, WorthUiRealtimeFrameDenialReason, WorthUiRealtimeFramePriority,
    WorthUiRealtimeFrameReceipt, WorthUiRealtimeFrameTarget, WorthUiRealtimeInspectionDenial,
    WorthUiRealtimeLaneCounters, WorthUiRealtimeOverlayLane, WorthUiRealtimePlanAvailability,
    WorthUiRealtimeTargetSummary, WorthUiRendererSurfaceAdmission, WorthUiRendererSurfaceHandle,
};
pub(crate) use super::execution::reload_counter_boundary::WorthUiReloadCostSeed;
pub use super::execution::reload_counter_boundary::{
    WorthUiCertifiedReloadLoweringCounterReceipt, WorthUiReloadCostContext,
    WorthUiReloadCounterBoundaryDenial, WorthUiReloadLoweringCounterReceipt,
    WorthUiReloadLoweringCounterReceiptBuilder, WorthUiReloadLoweringFoundationalEvidence,
};
#[cfg(test)]
pub use super::execution::reload_counter_boundary::{
    WorthUiReloadCounterBoundary, WorthUiReloadCounterBoundaryDenialReason,
    WorthUiReloadCounterStopStage, WorthUiReloadLoweringFoundationalBridge,
};
pub use super::execution::steady_frame_counter_boundary::{
    WorthUiCertifiedFrameExecutionReceipt, WorthUiFrameExecutionBasis,
    WorthUiFrameExecutionReceipt, WorthUiFrameReportMaterializationBoundary, WorthUiFrameWorkScope,
    WorthUiLaneFrameReceipt, WorthUiLaneFrameReceiptKind, WorthUiSteadyFrameCounterBoundary,
    WorthUiSteadyFrameCounterDenial, WorthUiSteadyFrameCounterDenialReason,
    WorthUiSteadyFrameCounters, WorthUiSteadyFrameFoundationalBridge,
    WorthUiSteadyFrameFoundationalEvidence, WorthUiSteadyFrameReportPlanner,
};
pub use super::execution::virtualized_data_lane::{
    WorthUiVirtualizedDataCertification, WorthUiVirtualizedDataCounters,
    WorthUiVirtualizedDataFrameDenial, WorthUiVirtualizedDataFrameDenialReason,
    WorthUiVirtualizedDataFrameReceipt, WorthUiVirtualizedDataFrameTarget,
    WorthUiVirtualizedDataLane, WorthUiVirtualizedDataNode, WorthUiVirtualizedDataPlan,
    WorthUiVirtualizedDataPlanDenial, WorthUiVirtualizedDataPlanDenialReason,
    WorthUiVirtualizedPlanAvailability, WorthUiVirtualizedPlanSummary,
    WorthUiVirtualizedPlanSummaryDenial, WorthUiVirtualizedPlanSummaryRequest, WorthUiVisibleRange,
    WorthUiVisibleRangeDenial, WorthUiVisibleRangeDenialReason,
};

// --- host observation ---
#[cfg(test)]
pub use super::host_observation::diagnostics::WorthUiDiagnosticRichnessTier;
pub use super::host_observation::diagnostics::{
    WorthUiDiagnosticProjectionHook, WorthUiDiagnosticRichnessPolicy, WorthUiDiagnosticSource,
    WorthUiRuntimeActivationStatus, WorthUiRuntimeDiagnostic, WorthUiRuntimeDiagnosticCode,
    WorthUiRuntimeDiagnosticCounters, WorthUiRuntimeDiagnosticFamily,
    WorthUiRuntimeDiagnosticPolicy, WorthUiRuntimeDiagnosticReport,
};
pub use super::host_observation::diagnostics_projection::{
    WorthUiBindingObservationRow, WorthUiBindingObservationSurface, WorthUiFrameCostSurface,
    WorthUiPlanInspectionSurface, WorthUiReloadStatusSurface,
};
#[cfg(test)]
pub use super::host_observation::diagnostics_projection::{
    WorthUiDiagnosticsProjectionDenialReason, WorthUiDiagnosticsProjectionHook,
    WorthUiFrameCostSurfaceKind,
};
pub use super::host_observation::identity_state_query_certification::{
    WorthUiIdentityStateCertification, WorthUiIdentityStateQueryCertificationCounters,
    WorthUiIdentityStateQueryCertificationDenial,
    WorthUiIdentityStateQueryCertificationDenialReason,
    WorthUiIdentityStateQueryCertificationScenario, WorthUiQueryDriftCertification,
    WorthUiQueryDriftCertificationScenarioStep, WorthUiStateCarryForwardReceipt,
    WorthUiStateCertificationScenarioStep, WorthUiStateLifecycleReceipt,
    WorthUiStateQueryResidueScan,
};
pub use super::host_observation::reload_failure::{
    WorthUiFailedActivationReport, WorthUiReloadCheckedStopPosture, WorthUiReloadDenial,
    WorthUiReloadFailure, WorthUiReloadFailureCounters, WorthUiReloadFailureStage,
    WorthUiReloadPreservationReceipt,
};
#[cfg(test)]
pub use super::host_observation::reload_storm_certification::{
    WorthUiReloadCertificationBundle, WorthUiReloadLatencyCounters,
    WorthUiReloadStormCandidateDenialReason, WorthUiReloadStormCandidateStep,
    WorthUiReloadStormCandidateStepKind, WorthUiReloadStormCertification,
    WorthUiReloadStormCertificationDenial, WorthUiReloadStormCertificationDenialReason,
    WorthUiReloadStormDeniedIteration, WorthUiReloadStormIterationOutcome,
    WorthUiReloadStormOrderedTruth, WorthUiReloadStormPreparedIteration,
    WorthUiReloadStormScenario,
};
pub use super::host_observation::{
    WorthUiRuntimeDiagnosticRequest, WorthUiRuntimeDiagnostics, WorthUiRuntimeInspectionAiHarness,
};

// --- source ingress ---
pub(crate) use super::source_ingress::WorthUiPreparedDeclarationMaterial;
#[cfg(test)]
pub use super::source_ingress::WorthUiSourceIngressHook;
pub(crate) use super::source_ingress::{
    prepare_rust_authored_handoff, WorthUiAuthoredCompositionPreparationDenial,
};
pub use super::source_ingress::{
    WorthUiAuthoredBackdropDeclaration, WorthUiAuthoredOverlayMaterial,
    WorthUiAuthoredPortalAnchorBinding, WorthUiAuthoredProjectionRequirement,
    WorthUiAuthoredServiceDeclaration, WorthUiCandidateComposition,
    WorthUiCandidateCompositionBasis, WorthUiCandidateOrderingReceipt,
    WorthUiFilesystemSourceAcquisitionDenial, WorthUiFilesystemSourceProvider,
    WorthUiFilesystemSourceWatcher, WorthUiFilesystemWatcherBackend,
    WorthUiFilesystemWatcherDenial, WorthUiFilesystemWatcherReadiness,
    WorthUiFilesystemWatcherShutdownReceipt, WorthUiProjectionContentEdge, WorthUiReloadDebounce,
    WorthUiSemanticHandoffEvidence, WorthUiSemanticHandoffPreparationDenial,
    WorthUiSemanticHandoffPreparationStop, WorthUiServiceDeclarationAdmissionCause,
    WorthUiSettledSourceSnapshot, WorthUiSourceEventIngress, WorthUiSourceEventIngressSession,
    WorthUiSourceIngressCounters, WorthUiSourceIngressDenial, WorthUiSourceIngressDenialReason,
    WorthUiSourcePackageRevision, WorthUiSourceProvider, WorthUiSourceProviderKind,
    WorthUiWatchedCandidateSubmission, WorthUiWatchedCandidateSubmissionDenial,
    WorthUiWatcherEvent,
};

// --- measurement ---
pub use super::measurement::{
    WorthUiCertifiedMeasurementPacket, WorthUiComplexityContract, WorthUiCounterCaptureRichness,
    WorthUiFoundationalCounterBridge, WorthUiFoundationalCounterEvidence, WorthUiFrameCostCounter,
    WorthUiMeasurementBoundary, WorthUiMeasurementCertificationDenial,
    WorthUiMeasurementCounterPacket, WorthUiRuntimeCounterFamily,
};
#[cfg(test)]
pub use super::measurement::{WorthUiCounterPacketBuilder, WorthUiMeasurementQueryEvidence};
