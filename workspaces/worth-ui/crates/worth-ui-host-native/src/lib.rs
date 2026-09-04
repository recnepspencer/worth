//! Qualified native mechanics profiles and the Worth-owned native host.

mod native;
mod native_profile;
mod prepared_host;
#[cfg(feature = "certification-support")]
mod qualification;
mod text_profile;

#[cfg(feature = "certification-support")]
pub use native::staged_appearance_capability_report;
#[cfg(feature = "certification-support")]
pub use native::{
    certify_client_close_with_queued_readiness, UiNativeQueuedReadinessCloseCertification,
};
#[cfg(feature = "certification-support")]
pub use native::{
    certify_portal_sample_replay, classify_presentation_fault,
    UiNativePortalSampleReplayCertification, UiNativePortalSampleReplayCertificationDenial,
    UiNativePresentationFault, UiNativePresentationFaultDisposition,
    UiNativePresentationRecoveryClass,
};
pub use native::{
    UiNativeApplicationReadinessGrant, UiNativeApplicationReadinessOwnerCount,
    UiNativeApplicationReadinessOwnerCountDenial, UiNativeApplicationReadinessPort,
    UiNativeApplicationReadinessSignalDenial, UiNativeApplicationReadinessSignalDisposition,
    UiNativeClientAuthoredMountedInstanceObservation, UiNativeClientConditionalOutcome,
    UiNativeClientDerivedStateLossClass, UiNativeClientDerivedStateReconstructionObservation,
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
    UiNativeClientVisualSnapshotRelation, UiNativeDerivedStateLossClass,
    UiNativeDerivedStateReconstructionObservation, UiNativeEffectPosture, UiNativeEventLoopCleanup,
    UiNativeEventLoopClient, UiNativeEventLoopClientCleanup, UiNativeEventLoopClientClose,
    UiNativeEventLoopClientFailure, UiNativeEventLoopDirective, UiNativeEventLoopRunDenial,
    UiNativeEventLoopRunReport, UiNativeEventLoopShutdownOverlapObservation,
    UiNativeEventLoopStopReport, UiNativeEventLoopThreadPosture, UiNativeGlyphObservation,
    UiNativeGraphicsObservation, UiNativeInputObservationEventFamily,
    UiNativeInputObservationReport, UiNativeInputObservationStop, UiNativeInputReachability,
    UiNativeObservationReadinessGrant, UiNativePhysicalPresentationCorrelation,
    UiNativePhysicalProgressClass, UiNativePhysicalProgressGrant,
    UiNativePhysicalSignalExternalStatusClass, UiNativePhysicalSignalLifecycleObservation,
    UiNativePhysicalSignalObservationOriginClass, UiNativePhysicalSignalSettlementClass,
    UiNativePhysicalSignalTransitionObservation, UiNativePhysicalSignalWorkClass,
    UiNativePointerButtonObservation, UiNativePresentationEffectPhase,
    UiNativePresentationObservation, UiNativePresentationWorkKind, UiNativeReadinessGrant,
    UiNativeReducedMotionPosture, UiNativeResourceCensus, UiNativeRetainedFrameObservation,
    UiNativeScrollDeltaObservation, UiNativeTextAtlasPlanObservation, UiNativeTextPinObservation,
    WorthUiNativeEventLoop,
};
#[cfg(feature = "certification-support")]
pub use native::{UiNativeCaptureExternalObservation, UiNativeCaptureProtocolWorld};
#[cfg(feature = "certification-support")]
pub use native::{UiNativeInputObservationContract, UiNativeInputObservationContractDisposition};
#[cfg(feature = "certification-support")]
pub use native::{
    UiNativeLifecycleEffect, UiNativeLifecyclePhase, UiNativeLifecycleProtocol,
    UiNativeLifecycleRequiredAction, UiNativeLifecycleTransition,
};
#[cfg(feature = "certification-support")]
pub use native::{
    UiNativeLifecycleProtocolReport, UiNativeLifecycleProtocolSchedule,
    UiNativeLifecycleProtocolWorld, UiNativeProtocolCloseDisposition, UiNativeProtocolClosePoint,
    UiNativeProtocolNextAction, UiNativeProtocolPredecessor, UiNativeProtocolReadback,
    UiNativeProtocolResourceCensus, UiNativeProtocolSurfaceTransition,
};
#[cfg(feature = "certification-support")]
pub use native::{
    UiNativeReadinessContract, UiNativeReadinessContractDenial, UiNativeReadinessContractOutcome,
    UiNativeReadinessContractWork,
};
pub use native_profile::{
    UiNativeMechanicsCapacities, UiNativePlatformProfileIdentity, WORTH_UI_NATIVE_PROFILE_MANIFEST,
};
pub use prepared_host::{
    UiNativeWindowConfiguration, WorthUiPreparedNativeHost, WorthUiPreparedNativeMechanics,
};
#[cfg(feature = "certification-support")]
pub use qualification::{UiNativeQualificationPlan, UiNativeQualificationPlanDenial};
pub use text_profile::{
    UiBodyDefaultAtlasCapacities, UiBodyDefaultTextProfileIdentity,
    UiUnsupportedBodyDefaultCodePoint, WORTH_UI_BODY_DEFAULT_FONT, WORTH_UI_BODY_DEFAULT_LICENSE,
    WORTH_UI_TEXT_PROFILE_MANIFEST,
};

#[cfg(test)]
mod qualification_tests;
