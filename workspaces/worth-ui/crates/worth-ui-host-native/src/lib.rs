//! Qualified native mechanics profiles and the Worth-owned native host.

mod native;
pub use native::{UiNativeInputRecoveryAcknowledgement, UiNativeInputRecoveryGrant};
mod native_profile;
mod prepared_host;
#[cfg(feature = "certification-support")]
mod qualification;
mod text_profile;

#[cfg(feature = "certification-support")]
pub use native::appearance_capability_report;
#[cfg(feature = "certification-support")]
pub use native::{
    certify_client_close_with_queued_readiness, UiNativeQueuedReadinessCloseCertification,
};
#[cfg(feature = "certification-support")]
pub use native::{
    certify_mounted_surface_sample, certify_portal_sample_replay, classify_presentation_fault,
    UiNativePortalSampleReplayCertification, UiNativePortalSampleReplayCertificationDenial,
    UiNativePresentationFault, UiNativePresentationFaultDisposition,
    UiNativePresentationRecoveryClass, UiNativeSurfaceSampleCertification,
    UiNativeSurfaceSampleCertificationDenial,
};
pub use native::{
    UiNativeApplicationReadinessGrant, UiNativeApplicationReadinessOwnerCount,
    UiNativeApplicationReadinessOwnerCountDenial, UiNativeApplicationReadinessPort,
    UiNativeApplicationReadinessSignalDenial, UiNativeApplicationReadinessSignalDisposition,
    UiNativeClientAuthoredMountedInstanceObservation, UiNativeClientDerivedStateLossClass,
    UiNativeClientDerivedStateReconstructionObservation,
    UiNativeClientObservationIngressObservation, UiNativeClientPresentationAttribution,
    UiNativeClientPresentationMechanicIdentityObservation,
    UiNativeClientPresentationTransitionKind, UiNativeClientPresentationTransitionObservation,
    UiNativeClientResourceObservation, UiNativeClientShutdownAttemptDisposition,
    UiNativeClientShutdownAttemptObservation, UiNativeClientShutdownObservation,
    UiNativeClientTextPresentationWorkObservation, UiNativeClientVisualCoordinateOrientation,
    UiNativeClientVisualCoordinateRounding, UiNativeClientVisualPixelColorSpace,
    UiNativeClientVisualSnapshotInput, UiNativeClientVisualSnapshotObservation,
    UiNativeClientVisualSnapshotRelation, UiNativeDerivedStateLossClass,
    UiNativeDerivedStateReconstructionObservation, UiNativeEffectPosture, UiNativeEventLoopCleanup,
    UiNativeEventLoopClient, UiNativeEventLoopClientCallback, UiNativeEventLoopClientCleanup,
    UiNativeEventLoopClientClose, UiNativeEventLoopClientDenial, UiNativeEventLoopClientFailure,
    UiNativeEventLoopDirective, UiNativeEventLoopRunDenial, UiNativeEventLoopRunReport,
    UiNativeEventLoopShutdownOverlapObservation, UiNativeEventLoopStopReport,
    UiNativeEventLoopThreadPosture, UiNativeGlyphObservation, UiNativeGraphicsObservation,
    UiNativeInputObservationEventFamily, UiNativeInputObservationReport,
    UiNativeInputObservationStop, UiNativeInputReachability, UiNativeObservationClock,
    UiNativeObservationReadinessGrant, UiNativeObservationTimeProgress,
    UiNativePhysicalPresentationCorrelation, UiNativePhysicalProgressClass,
    UiNativePhysicalProgressGrant, UiNativePhysicalSignalExternalStatusClass,
    UiNativePhysicalSignalLifecycleObservation, UiNativePhysicalSignalObservationOriginClass,
    UiNativePhysicalSignalSettlementClass, UiNativePhysicalSignalTransitionObservation,
    UiNativePhysicalSignalWorkClass, UiNativePointerButtonObservation,
    UiNativePresentationEffectPhase, UiNativePresentationObservation, UiNativePresentationWorkKind,
    UiNativeReadinessGrant, UiNativeReducedMotionPosture, UiNativeResourceCensus,
    UiNativeRetainedFrameObservation, UiNativeScrollDeltaObservation,
    UiNativeTextAtlasPlanObservation, UiNativeTextPinObservation, WorthUiNativeEventLoop,
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
    UiNativeClientBackground, UiNativeCpuAdapterAdmission, UiNativeMechanicsCapacities,
    UiNativePlatformProfileIdentity, UiNativeQualifiedTarget, UiNativeSurfaceProfile,
    UiNativeWindowingSystem, WORTH_UI_NATIVE_CLIENT_BACKGROUND, WORTH_UI_NATIVE_PROFILE_IDENTITY,
    WORTH_UI_NATIVE_PROFILE_MANIFEST, WORTH_UI_NATIVE_SURFACE_PROFILE,
    WORTH_UI_NATIVE_WINDOWING_SYSTEM, WORTH_UI_QUALIFIED_PROFILE_IDENTITIES,
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

#[cfg(feature = "certification-support")]
pub use native::{
    UiNativeTextForegroundAtlasModel, UiNativeTextForegroundCoverageCertification,
    UiNativeTextForegroundFinalizationDenial, UiNativeTextForegroundJoinCost,
    UiNativeTextReplayOperation, UiNativeTextRetentionCertificationDenial,
};
