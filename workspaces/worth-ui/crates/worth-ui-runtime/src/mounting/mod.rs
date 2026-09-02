mod assembly;
mod backdrop;
mod counters;
mod delta;
mod denial;
mod focus_participation;
mod frame_assembler;
mod frame_manifest_validation;
mod host_truth;
mod identity;
mod identity_overlay;
mod identity_state;
mod identity_trace_basis;
mod identity_view;
mod portal_overlay;
pub(crate) mod presentation;
mod projection;
mod projection_changes;
mod publication;
#[cfg(any(test, feature = "certification-support"))]
pub(crate) mod qualified_text_test_support;
mod receipt_basis;
mod retention;
mod reuse;
mod semantic_content;
mod session_state;
mod surface_binding;
mod text_reuse;
mod theme_values;
mod visual_region_basis;

pub(crate) use assembly::{binding_requirement, UiPreparedMountedFrameAdmission};
pub(crate) use counters::{UiMountCostOverflow, UiMountStageCounters};
pub use counters::{UiMountCostReport, UiMountNamedCounters, UiMountWorkClass};
pub use delta::UiMountedFrameDelta;
pub use denial::UiMountedIdentityDenial;
pub(crate) use focus_participation::{
    UiMountedFocusParticipant, UiMountedFocusParticipationSnapshot,
};
pub(crate) use frame_assembler::{
    UiMountedFrameAssembler, UiMountedFrameAssemblyInput, UiMountedLaneAssembly,
    UiMountedPlanProjectionSource,
};
pub(crate) use frame_manifest_validation::validate_manifest;
pub(crate) use host_truth::UiMountedHostTruthCoordinator;
pub use identity::{UiMountedGraphNodeHandle, UiMountedGraphWorldIdentity, UiMountedIdentityBasis};
pub(crate) use identity_overlay::UiMountedVisualOverlayProjectionInput;
pub(crate) use identity_state::{
    UiAuthorityAdmittedMountedFrame, UiCurrentHitTarget, UiCurrentHitTargetAffinityDenial,
    UiCurrentInteractionAffinity, UiMountedIdentityState, UiMountedIncarnationAffinityInput,
    UiMountedInteractionAffinityInput,
};
pub(crate) use identity_trace_basis::UiMountedIdentityTraceBasis;
pub use identity_view::{
    UiMountedFrameIdentityView, UiMountedIdentityView, UiMountedInstanceIdentityView,
    UiSurfaceBindingIdentityView,
};
pub(crate) use portal_overlay::UiMountedPortalOverlayProjectionInput;
pub use presentation::{
    UiFocusHostPlacementReconciliationDenial, UiFocusHostPlacementReconciliationOutcome,
    UiFocusHostPlacementReconciliationReceipt, UiFocusHostPlacementShutdownReport,
    UiHostPresentationReconciliation, UiMountedIndeterminateFrame, UiMountedPresentationAdmission,
    UiMountedPresentationAdmissionDenial, UiMountedPresentationAdmissionRejection,
    UiMountedPresentationAttempt, UiMountedPresentationCompletionDenial,
    UiMountedPresentationInFlight, UiMountedPresentationOutcome, UiMountedPresentationReceipt,
    UiMountedPresentationShutdownAttempt, UiMountedPresentationShutdownDisposition,
    UiMountedPresentationShutdownReport, UiMountedPresentationWitness, UiMountedPresentedFrame,
    UiMountedRejectedFrame, UiMountedSupersededFrame, UiMountedSurfacePresentationReceipt,
    UiMountedSurfacePresentationRejection, UiMountedSurfaceReconciliationBinding,
    UiPresentationIndeterminateReport,
};
pub(crate) use presentation::{
    UiFocusHostPlacementSettlementDenial, UiMotionSamplePresentationOutcome,
    UiMountedFocusPlacementDenial, UiMountedFocusPlacementRequestBasis,
    UiMountedHostPresentationAuthority, UiMountedPresentationCoordinator,
    UiMountedSupersedingPresentationBasis,
};
#[allow(unused_imports)]
pub(crate) use projection::compile_presentation_sources;
pub(crate) use projection::UiMountedFocusScope;
pub(crate) use projection::{
    prepare_projection, UiIntentPostureCommit, UiIntentPostureObservation, UiIntentPostureTable,
    UiMountedPresentationDeltaSource, UiMountedPreviewProjectionInput, UiMountedProjectionInput,
    UiPreparedMountedProjection,
};
pub use projection::{
    UiMountedNodeReceipt, UiMountedProjectionDenial, UiMountedProjectionFrame,
    UiProjectedMountedFrameCandidate,
};
pub(crate) use projection_changes::{
    UiMountedProjectionChangeSnapshot, UiMountedProjectionChanges,
};
pub use publication::{
    UiMountedFrameOutcome, UiMountedFramePublicationReceipt, UiMountedPublicationLeaseDenial,
};
pub(crate) use publication::{
    UiMountedFramePublicationCandidate, UiMountedFramePublicationCommit,
    UiMountedFrameReconciliationCandidate,
};
pub(crate) use receipt_basis::UiMountedNodeReceiptBasis;
pub(crate) use retention::UiPresentedHitTestRow;
pub(crate) use retention::{
    UiMountedDiagnosticInspectionBasis, UiMountedDiagnosticInspectionDenial,
    UiMountedDiagnosticRetentionLease, UiMountedFrameInspectionBasis,
    UiMountedFrameInspectionDenial, UiMountedFrameInspectionSelection,
    UiMountedFrameInspectionTarget, UiMountedFrameRetentionCoordinator,
    UiMountedFrameRetentionSnapshot, UiMountedObservationBasisLease,
    UiMountedObservationBasisRetentionDenial, UiMountedRetentionUsageSnapshot,
    UiMountedVisualCaptureBasis, UiMountedVisualOverlayLease, UiMountedVisualRetentionDenial,
    UiMountedVisualSnapshotLease, UiPresentedFrameBasisDenial, UiPresentedFrameBasisRelation,
    UiPresentedHitTestBasis, UiRetainedMountedDiagnostics, DEFAULT_OBSERVATION_FRAME_CAPACITY,
};
pub use retention::{
    UiMountedFrameRetentionBudget, UiMountedFrameRetentionBudgetInput,
    UiMountedFrameRetentionDenial, UiMountedFrameRetentionRejection, UiMountedRetentionClass,
    UiMountedRetentionClassBudget, UiMountedRetentionLease,
};
pub(crate) use reuse::UiMountedFrameReuseExternalBasis;
pub use reuse::{
    UiMountedFrameExecutionPosture, UiMountedFrameReuse, UiMountedFrameReuseComparator,
    UiMountedFrameReuseContract, UiMountedFrameReuseDependency, UiMountedFrameReuseMintingStage,
    UiMountedFrameReuseWitness,
};
pub(crate) use semantic_content::{
    UiMountedCollectionRowIdentity, UiMountedCollectionSemanticTextContent,
    UiMountedCollectionTextChange, UiMountedCollectionTextDirective, UiMountedCollectionTextRow,
    UiMountedScalarSemanticTextContent, UiMountedSemanticContentInput,
    UiMountedSemanticTextContent, UiMountedSemanticTextFormattingDirective,
    UiMountedSemanticTextValueDirective,
};
pub(crate) use session_state::{
    UiMountedGraphReplacementAdmission, UiMountedGraphReplacementInFlight,
    UiMountedGraphReplacementPreparation, UiMountedGraphReplacementPresentation,
    UiMountedGraphReplacementSuccessor, UiMountedHostObservationTransition,
    UiMountedMotionSampleSettlement, UiMountedObservationValidationBasis,
    UiMountedPublicationTransition, WorthUiMountedSessionState,
};
pub use surface_binding::{UiSurfaceBindingCoordinatePosture, UiSurfaceBindingProfile};
pub(crate) use text_reuse::{
    UiMountedTextForegroundPresentationBasis, UiMountedTextForegroundReuseReceipt,
};
pub(crate) use visual_region_basis::{
    UiMountedHitTestPresentation, UiMountedUnsupportedPaintBasis, UiMountedVisualRegionBasis,
};

pub use assembly::{
    UiMountedFramePreparationDenial, UiMountedFrameReceipt, UiMountedFrameRequest,
    UiMountedSurfaceReceipt, UiPreparedMountedFrame,
};
pub(crate) use theme_values::{UiMountedPreviewThemeBinding, UiMountedThemeValueSource};
pub use worth_ui_host_contract::{
    UiHostSurfaceBaselineIdentity, UiHostSurfaceIdentity, UiHostSurfacePresentationMode,
    UiMountIncarnation, UiMountedFrameIdentity, UiMountedInstanceIdentity,
    UiMountedNodeReceiptIdentity, UiMountedProjectionAudience, UiSemanticSurfaceIdentity,
    UiSurfaceBindingGeneration,
};

#[cfg(test)]
pub(crate) fn prove_paint_only_mechanic_locality() {
    projection::prove_paint_only_mechanic_locality();
}
