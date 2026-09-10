mod capture;
mod derived_state_reconstruction;
mod event_loop;
mod graphics;
mod host_state;
#[cfg(test)]
mod host_state_lifecycle_tests;
mod input;
mod lifecycle;
mod lifecycle_protocol;
mod mechanics_adapter;
mod observation;
mod physical_work_signal;
mod platform;
mod presentation;
mod readiness;
#[cfg(feature = "certification-support")]
mod readiness_certification;
mod solicited_effect;
mod text_atlas;

#[cfg(feature = "certification-support")]
pub use capture::{UiNativeCaptureExternalObservation, UiNativeCaptureProtocolWorld};
pub use derived_state_reconstruction::{
    UiNativeDerivedStateLossClass, UiNativeDerivedStateReconstructionObservation,
};
#[cfg(feature = "certification-support")]
pub use event_loop::{
    certify_client_close_with_queued_readiness, UiNativeQueuedReadinessCloseCertification,
};
pub use event_loop::{
    UiNativeApplicationReadinessGrant, UiNativeApplicationReadinessOwnerCount,
    UiNativeApplicationReadinessOwnerCountDenial, UiNativeClientAuthoredMountedInstanceObservation,
    UiNativeClientConditionalOutcome, UiNativeClientDerivedStateLossClass,
    UiNativeClientDerivedStateReconstructionObservation,
    UiNativeClientObservationIngressObservation, UiNativeClientPresentationAttribution,
    UiNativeClientPresentationMechanicIdentityObservation,
    UiNativeClientPresentationSemanticChange,
    UiNativeClientPresentationSemanticFrontierObservation,
    UiNativeClientPresentationSemanticSubscriberObservation,
    UiNativeClientPresentationTransitionKind, UiNativeClientPresentationTransitionObservation,
    UiNativeClientResourceObservation, UiNativeClientShutdownAttemptDisposition,
    UiNativeClientShutdownAttemptObservation, UiNativeClientShutdownObservation,
    UiNativeClientTextPresentationWorkObservation, UiNativeClientVisualCoordinateOrientation,
    UiNativeClientVisualCoordinateRounding, UiNativeClientVisualPixelColorSpace,
    UiNativeClientVisualSnapshotInput, UiNativeClientVisualSnapshotObservation,
    UiNativeClientVisualSnapshotRelation, UiNativeEventLoopCleanup, UiNativeEventLoopClient,
    UiNativeEventLoopClientCleanup, UiNativeEventLoopClientClose, UiNativeEventLoopClientFailure,
    UiNativeEventLoopDirective, UiNativeEventLoopRunDenial, UiNativeEventLoopRunReport,
    UiNativeEventLoopShutdownOverlapObservation, UiNativeEventLoopStopReport,
    UiNativeEventLoopThreadPosture, UiNativeInputReachability, UiNativeObservationClock,
    UiNativeObservationReadinessGrant, UiNativeObservationTimeProgress,
    UiNativePhysicalPresentationCorrelation, UiNativePhysicalProgressClass,
    UiNativePhysicalProgressGrant, UiNativeReadinessGrant, UiNativeReducedMotionPosture,
    WorthUiNativeEventLoop,
};
#[cfg(test)]
pub(crate) use graphics::QUALIFIED_DX12_PRESENTATION_SYSTEM;
pub(crate) use graphics::{
    UiNativeDeviceGeneration, UiNativeGraphicsRecovery, UiNativeOwnedDevice,
};
pub(crate) use host_state::UiNativeHostState;
pub use host_state::{UiNativeEffectPosture, UiNativePresentationEffectPhase};
#[cfg(feature = "certification-support")]
pub use input::{UiNativeInputObservationContract, UiNativeInputObservationContractDisposition};
pub(crate) use input::{
    UiNativeInputObservationDisposition, UiNativeInputObservationState,
    UiNativePointerPositionWitness,
};
pub use input::{
    UiNativeInputObservationEventFamily, UiNativeInputObservationReport,
    UiNativeInputObservationStop, UiNativePointerButtonObservation, UiNativeScrollDeltaObservation,
};
pub use lifecycle::UiNativeResourceCensus;
pub(crate) use lifecycle::{
    prepare_external_recovery, UiNativeLifecycleDirective, UiNativeOwnedResource,
    UiNativePresentationAccess, UiNativePresentationRetryFinalization,
    UiNativePresentationRetryWake, UiNativeRecoveryCause, UiNativeRecoveryLineage,
    UiNativeRecoveryRegistry, UiNativeRecoveryRequirement, UiNativeResourceClass,
    UiNativeResourceOwner, UiNativeResourceRegistry, UiNativeShutdownPhase,
    UiNativeSurfaceBasisTransition,
};
#[cfg(feature = "certification-support")]
pub use lifecycle::{
    UiNativeLifecycleProtocolReport, UiNativeLifecycleProtocolSchedule,
    UiNativeLifecycleProtocolWorld, UiNativeProtocolCloseDisposition, UiNativeProtocolClosePoint,
    UiNativeProtocolNextAction, UiNativeProtocolPredecessor, UiNativeProtocolReadback,
    UiNativeProtocolResourceCensus, UiNativeProtocolSurfaceTransition,
};
pub use lifecycle_protocol::{
    UiNativeLifecycleEffect, UiNativeLifecyclePhase, UiNativeLifecycleProtocol,
    UiNativeLifecycleRequiredAction, UiNativeLifecycleTransition,
};
#[cfg(feature = "certification-support")]
pub use mechanics_adapter::appearance_report as appearance_capability_report;
pub(crate) use mechanics_adapter::WorthUiNativeMechanicsAdapter;
pub(crate) use observation::UiNativePresentationInput;
pub use observation::{
    UiNativeGlyphObservation, UiNativeGraphicsObservation, UiNativePresentationObservation,
    UiNativePresentationWorkKind, UiNativeRetainedFrameObservation,
};
pub use physical_work_signal::{
    UiNativePhysicalSignalExternalStatusClass, UiNativePhysicalSignalLifecycleObservation,
    UiNativePhysicalSignalObservationOriginClass, UiNativePhysicalSignalSettlementClass,
    UiNativePhysicalSignalTransitionObservation, UiNativePhysicalSignalWorkClass,
};
pub(crate) use platform::UiNativePointerInputPort;
#[cfg(all(test, feature = "certification-support"))]
pub(crate) use presentation::appearance::APPEARANCE_MECHANICS;
#[cfg(test)]
pub(crate) use presentation::GPU_WAIT_DEADLINE;
#[cfg(feature = "certification-support")]
pub use presentation::{
    certify_mounted_surface_sample, certify_portal_sample_replay, classify_presentation_fault,
    UiNativePortalSampleReplayCertification, UiNativePortalSampleReplayCertificationDenial,
    UiNativePresentationFault, UiNativePresentationFaultDisposition,
    UiNativePresentationRecoveryClass, UiNativeSurfaceSampleCertification,
    UiNativeSurfaceSampleCertificationDenial,
};
pub(crate) use presentation::{
    UiNativeOwnedPresentationSurface, UiNativePendingPresentation, UiNativeRetainedDrawList,
};
#[cfg(feature = "certification-support")]
pub(crate) use readiness::UiNativeReadyWork;
pub use readiness::{
    UiNativeApplicationReadinessPort, UiNativeApplicationReadinessSignalDenial,
    UiNativeApplicationReadinessSignalDisposition,
};
pub(crate) use readiness::{UiNativeReadinessRegistry, UiNativeReadyOwner};
#[cfg(feature = "certification-support")]
pub use readiness_certification::{
    UiNativeReadinessContract, UiNativeReadinessContractDenial, UiNativeReadinessContractOutcome,
    UiNativeReadinessContractWork,
};
pub use text_atlas::UiNativeTextAtlasPlanObservation;
pub use text_atlas::UiNativeTextPinObservation;

#[cfg(feature = "certification-support")]
pub use mechanics_adapter::text_atlas::{
    UiNativeTextForegroundAtlasModel, UiNativeTextForegroundCoverageCertification,
    UiNativeTextForegroundFinalizationDenial, UiNativeTextForegroundJoinCost,
    UiNativeTextReplayOperation, UiNativeTextRetentionCertificationDenial,
};
