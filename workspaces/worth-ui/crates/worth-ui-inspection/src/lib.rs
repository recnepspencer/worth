//! Public inspection surfaces grouped by lifecycle authority:
//! target identity → query admission → receipt projection → evidence contract → posture → scope.

mod allocation;
mod appearance;
mod evidence_contract;
mod facade;
mod intent;
mod posture;
mod query;
mod receipt;
mod scope;
mod service;
mod target;

// Target identity lane
pub use target::{
    UiAuthoredSourceProvenanceRef, UiInspectionAspectName, UiInspectionDeclarationIdentity,
    UiInspectionTarget, UiSourceArtifactGeneration, UiSourceArtifactIdentity,
};

// Appearance inspection lane. These are inert evidence contracts; runtime
// owns production and no contract below can authorize a runtime mutation.
pub use appearance::{
    UiAppearanceInspectionCost, UiAppearanceInspectionDecisionCell, UiAppearanceInspectionEvidence,
    UiAppearanceInspectionExplanation, UiAppearanceInspectionInvalidationCause,
    UiAppearanceInspectionMountedMechanic, UiAppearanceInspectionOutcome,
    UiAppearanceInspectionPhysicalSuppression, UiAppearanceInspectionQuery,
    UiAppearanceInspectionSourceSpan, UiAppearanceInspectionSupport, UiAppearanceInspectionValue,
    UiAppearanceInspectionWorld,
};

// Evidence contract lane
pub use allocation::{
    UiAllocationInspectionAnchorPosture, UiAllocationInspectionAttemptResult,
    UiAllocationInspectionAxis, UiAllocationInspectionBounds,
    UiAllocationInspectionCoordinateSpace, UiAllocationInspectionDenialFamily,
    UiAllocationInspectionDeniedAttempt, UiAllocationInspectionEdgeReference,
    UiAllocationInspectionEvidenceFamily, UiAllocationInspectionEvidenceRef,
    UiAllocationInspectionFreshnessPosture, UiAllocationInspectionGeometry,
    UiAllocationInspectionGraphNodeIdentity, UiAllocationInspectionInvalidationFamily,
    UiAllocationInspectionKnowledge, UiAllocationInspectionNeighborhoodIdentity,
    UiAllocationInspectionPlanningBasisIdentity, UiAllocationInspectionPortalAnchorTargetIdentity,
    UiAllocationInspectionReceipt, UiAllocationInspectionReceiptIdentity,
    UiAllocationInspectionReceiptProjection, UiAllocationInspectionReuseDenialPosture,
    UiAllocationInspectionReusePosture, UiAllocationInspectionSelection,
    UiAllocationInspectionStreamFamily,
};
pub use evidence_contract::{
    UiEvidenceAuthorityArtifactIdentity, UiEvidenceAuthorityBinding, UiEvidenceAuthorityGeneration,
    UiEvidenceAuthorityKind, UiEvidenceExpansionOutcome, UiEvidenceFamily,
    UiEvidenceMaterializationPosture, UiEvidenceRetentionPosture,
    UiInspectionForeignEvidenceCitation, UiInspectionForeignEvidenceRef,
    UiInspectionQueryForeignEvidenceArtifactKind, UiInspectionQueryForeignEvidenceCitation,
    UiInspectionQueryForeignEvidenceKind, UiInspectionQueryForeignEvidenceRef,
};

// Intent evidence lane
pub use intent::{
    UiIntentCausalTraceAdmissionEvidence, UiIntentCausalTraceAttemptEvidence,
    UiIntentCausalTraceAttemptPosture, UiIntentCausalTraceCompletionEvidence,
    UiIntentCausalTraceEvidence, UiIntentCausalTraceOperabilityEvidence,
    UiIntentCausalTraceOperabilityPosture, UiIntentCausalTracePayloadEvidence,
    UiIntentCausalTraceRouteEvidence, UiIntentEvidenceLookup, UiIntentEvidenceReference,
    UiIntentEvidenceRetentionOmission, UiIntentEvidenceRetentionOutcome,
    UiIntentEvidenceRetirementCause, UiIntentEvidenceRetirementReport, UiIntentInteractionEvidence,
    UiIntentInteractionEvidenceFamily, UiIntentInteractionEvidenceInput,
    UiIntentInteractionEvidenceTargetInput, UI_INTENT_CAUSAL_TRACE_EVIDENCE_BYTE_CAPACITY,
    UI_INTENT_INTERACTION_EVIDENCE_ENTRY_CAPACITY,
};

// Posture lane
pub use posture::{
    UiInspectionAdmissionPosture, UiInspectionDeferredPosture, UiInspectionDiagnosticOnlyPosture,
    UiInspectionMilestoneExpectation, UiInspectionPosture, UiInspectionSupportPosture,
    UiInspectionSupportReason, UiInspectionSupportStatus, UiInspectionSupportWorld,
    UiInspectionUnsupportedPosture, UiInspectionWrongWorldPosture,
};

// Query admission lane
pub use query::{
    SealedPixelArtifactPolicy, UiAllocationPlanningQuestion, UiEvidenceBudget, UiEvidenceLinkKind,
    UiEvidenceRichness, UiGeometryOnly, UiInspectionAspectRelevanceDetail,
    UiInspectionEvidenceSource, UiInspectionObligationRelevanceDetail, UiInspectionQuery,
    UiInspectionRelevance, UiInspectionRelevanceAdmission, UiInspectionRelevanceOutcome,
    UiInspectionTargetClass, UiPixelsOptional, UiPixelsRequired, UiRelevanceFamily,
    UiRelevanceFilter, UiVisualArtifactPolicy, UiVisualCaptureCancellation,
    UiVisualCaptureDeadline, UiVisualInspectionAudience, UiVisualInspectionByteBudget,
    UiVisualInspectionCapacity, UiVisualInspectionDisclosure, UiVisualInspectionPolicy,
    UiVisualInspectionPolicyDenial, UiVisualInspectionRegionCapacity, UiVisualPixelRedaction,
    UiVisualSnapshotRequest,
};

// Receipt projection lane
pub use receipt::evidence::{
    UiEvidenceSliceOmission, UiInspectionAdmissionHostCapability, UiInspectionAdmissionQueryBasis,
    UiInspectionAdmissionStaleEvidence, UiInspectionObligationDecision,
    UiInspectionObligationDenialPosture, UiInspectionObligationDispatchPosture,
    UiInspectionObligationFamily, UiInspectionObligationLegalityReason,
    UiInspectionObligationNonSelectionReason, UiInspectionObligationSelectionReason,
    UiInspectionObligationSupportSelectionPosture, UiInspectionObligationVerdictClass,
    UiInspectionObligationVerdictPosture, UiInspectionObligationWorldProfileClass,
    UiInspectionSelectionBudget, UiInspectionSupportRowSchemaKind, UiInspectionTouchAspectPosture,
    UiInspectionTouchOriginClass, UiInspectionTouchRuntimeLane, UiInspectionTouchTargetClass,
};
pub use receipt::{
    UiClientPhysicalPixel, UiClientPhysicalRect, UiHitTestRegionIndexIdentity,
    UiHostSurfaceLogicalPoint, UiInspectionAiHarnessLane, UiInspectionClosedSemanticLane,
    UiInspectionCloseoutGuarantee, UiInspectionCloseoutNonGoal, UiInspectionCloseoutReport,
    UiInspectionClosureReport, UiInspectionCostLane, UiInspectionCostReceipt,
    UiInspectionDerivedIndexLane, UiInspectionMeasurementBasisInput,
    UiInspectionMeasurementBasisPosture, UiInspectionMeasurementBasisSource,
    UiInspectionMeasurementChildIntrinsicSource, UiInspectionMeasurementDenialPosture,
    UiInspectionMeasurementDependencyLineageEntry, UiInspectionMeasurementDependencyLineageKind,
    UiInspectionMeasurementEvidenceCategory, UiInspectionMeasurementEvidenceSlot,
    UiInspectionMeasurementEvidenceView, UiInspectionMeasurementEvidenceViewInput,
    UiInspectionMeasurementFailureSource, UiInspectionMeasurementGenerationCompatibility,
    UiInspectionMeasurementNeighborhoodClassHint, UiInspectionMeasurementOwnershipPosture,
    UiInspectionMeasurementQueryFactFamily, UiInspectionMeasurementQueryUnsupportedReason,
    UiInspectionQueryWorldCompatibilityFailure, UiInspectionRefLifecycleLane,
    UiInspectionScopeSupportRow, UiInspectionSliceLane, UiInspectionSupportReport,
    UiNativeScreenPhysicalPixel, UiRebindDecisionDisposition, UiRebindDecisionIndex,
    UiRebindDecisionIndexDenial, UiRebindDecisionKey, UiRebindDecisionLookup,
    UiRebindDecisionRecord, UiRebindDecisionRecordInput, UiRebindDecisionStopPoint,
    UiRebindStructuralCost, UiViewportLogicalPoint, UiVisibleRegionIndexIdentity,
    UiVisualAuthoredProvenance, UiVisualComparisonPixelPolicy, UiVisualContributorStack,
    UiVisualCoordinateDenial, UiVisualCoordinateObservation, UiVisualCoordinateObservationInput,
    UiVisualCoordinateOrientation, UiVisualCoordinateRounding, UiVisualDeclarationRef,
    UiVisualDerivedPixelArtifactInput, UiVisualEvidenceRef, UiVisualGraphNodeRef,
    UiVisualHitTestOutcome, UiVisualHitTestTarget, UiVisualIdentityContinuity,
    UiVisualIdentityTrace, UiVisualIdentityTraceInput, UiVisualInspectionCostLane,
    UiVisualInspectionCostReceipt, UiVisualMountedNodeRef, UiVisualNativePixelArtifactInput,
    UiVisualOverlayDenial, UiVisualPixelArtifact, UiVisualPixelArtifactValidity,
    UiVisualPixelCaptureSource, UiVisualPixelColorSpace, UiVisualPixelFormat,
    UiVisualPixelRetentionDisposition, UiVisualPointAdjudication, UiVisualQueryBudget,
    UiVisualRegionAdjudication, UiVisualRegionCompleteness, UiVisualRegionIntersection,
    UiVisualSnapshotAffinity, UiVisualSnapshotArtifactPosture, UiVisualSnapshotComparison,
    UiVisualSnapshotComparisonBudget, UiVisualSnapshotComparisonBudgetDenial,
    UiVisualSnapshotComparisonCost, UiVisualSnapshotComparisonDenial,
    UiVisualSnapshotComparisonDenialKind, UiVisualSnapshotComparisonExpiry,
    UiVisualSnapshotComparisonIncompatibility, UiVisualSnapshotComparisonInput,
    UiVisualSnapshotComparisonOmission, UiVisualSnapshotComparisonOutcome, UiVisualSnapshotDenial,
    UiVisualSnapshotEvidence, UiVisualSnapshotEvidenceInput, UiVisualSnapshotIndeterminate,
    UiVisualSnapshotOmission, UiVisualSnapshotRelation, UiVisualSnapshotSuperseded,
    UiVisualVisibleContributor, UiVisualVisibleOutcome,
};

// Runtime-service causal inspection lane
pub use service::{
    UiCommandRouteLossInspection, UiCommandRouteLossInspectionReason,
    UiCommandRouteScopeInspection, UiCommandWonInspectionSummary, UiFocusMoveInspectionCause,
    UiFocusMoveInspectionOutcome, UiFocusMovedInspectionSummary,
    UiFocusRestorationFailedInspectionSummary, UiFocusRestorationFailureInspectionReason,
    UiMotionInterruptedInspectionReason, UiMotionInterruptedInspectionSummary,
    UiPortalClosedInspectionReason, UiPortalClosedInspectionSummary,
    UiRuntimeServiceInspectionCost, UiRuntimeServiceInspectionFamily,
    UiRuntimeServiceInspectionSource, UiRuntimeServiceResourceCensus,
    UiScrollOwnerInspectionSummary, UiSelectionDropInspectionReason,
    UiSelectionDroppedInspectionSummary,
};

// Scope lane
pub use scope::UiInspectionScope;

// Facade inventory lane
pub use facade::{UiInspectionScopeInventory, RUNTIME_INSPECTION_SCOPE_INVENTORY};
