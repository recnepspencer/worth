mod assembly;
mod identity;
mod presentation;
mod presentation_cost;
mod presentation_work;
mod protocol;
mod surface_registration;
mod surface_stop;

#[cfg(test)]
mod presentation_work_tests;

pub use assembly::{
    UiMountedFrameCanonicalCore, UiMountedFrameIntegrity, UiMountedFrameManifest,
    UiMountedLaneParticipation, UiMountedSurfaceBindingRequirement, UiRequiredLaneContribution,
    UiRequiredLaneContributionStatus,
};
pub use identity::{
    UiHostPresentationLineageIdentity, UiHostSurfaceIdentity, UiHostSurfacePresentationMode,
    UiMountIncarnation, UiMountedContentGeneration, UiMountedContractIdentityExhaustion,
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedNodeReceiptAffinity,
    UiMountedNodeReceiptIdentity, UiMountedNodeReceiptIssuer, UiMountedPresentationAttemptIdentity,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};
pub use presentation::{
    UiHostPresentationCompletionToken, UiHostPresentationProgressClass,
    UiHostSurfaceCancellationOutcome, UiHostSurfaceInFlightCompletion,
    UiHostSurfacePresentationDenial, UiHostSurfacePresentationOutcome, UiMountedCompletedEffects,
    UiMountedEffectFamily, UiMountedFrameConsumptionInput, UiMountedFrameConsumptionView,
    UiMountedSurfacePresentationCompletion, UiMountedSurfacePresentationSupersession,
    UiMountedTextDemandValidationCost, UiMountedTextDemandValidationDenial,
    UiMountedTextRasterCallback, UiMountedTextRasterWork, UiPresentationDeadline,
};
pub use presentation_cost::{
    UiHostPresentationCostInput, UiHostPresentationCostOverflow, UiHostPresentationCostReport,
    UiMountedPresentationProductionCost, UiMountedPresentationProductionCostInput,
};
pub use presentation_work::{
    UiMountedLogicalDamage, UiMountedPaintCommand, UiMountedPaintCommandChange,
    UiMountedPaintCommandIdentity, UiMountedPaintOrderEdit, UiMountedPaintOrderIdentity,
    UiMountedPaintOrderIntegrity, UiMountedPresentationAffinity,
    UiMountedPresentationAuxiliaryState, UiMountedPresentationDelta,
    UiMountedPresentationDeltaInput, UiMountedPresentationInitial,
    UiMountedPresentationInitialInput, UiMountedPresentationNodeChange,
    UiMountedPresentationNodeHitTest, UiMountedPresentationNodePaint,
    UiMountedPresentationNodeState, UiMountedPresentationNodeStateInput,
    UiMountedPresentationOpacity, UiMountedPresentationReconstruction,
    UiMountedPresentationReconstructionDenial, UiMountedPresentationReconstructionInput,
    UiMountedPresentationSample, UiMountedPresentationSampleChange,
    UiMountedPresentationSampleConstructionDenial, UiMountedPresentationSampleInput,
    UiMountedPresentationTransform, UiMountedPresentationUnchanged,
    UiMountedPresentationUnchangedInput, UiMountedPresentationWorkView,
};
pub use protocol::{
    UiHostMeasurementSchemaVersion, UiHostObservationSchemaVersion, UiHostProtocolAgreement,
    UiHostProtocolContract, UiHostProtocolDenial, UiHostProtocolIdentity,
    UiHostProtocolNegotiation, UiHostProtocolSchemaFamily, UiHostProtocolVersion,
    UiHostSolicitedEffectSchemaVersion, UiMountedFrameSchemaVersion,
    UiMountedPresentationSchemaVersion,
};
pub use surface_registration::{
    UiHostSurfaceBaselineIdentity, UiHostSurfaceDeregistrationIndeterminate,
    UiHostSurfaceDeregistrationOutcome, UiHostSurfaceDeregistrationReceipt,
    UiHostSurfaceRegistrationDenial, UiHostSurfaceRegistrationIndeterminate,
    UiHostSurfaceRegistrationInput, UiHostSurfaceRegistrationOutcome,
    UiHostSurfaceRegistrationRequest,
};
pub use surface_stop::UiHostSurfaceStopReason;
