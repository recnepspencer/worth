//! Public native-platform facade over the runtime-owned binding gate.

pub use worth_ui_runtime::native_platform::{
    UiNativeApplicationBuilder, UiNativeApplicationDefinition, UiNativeApplicationFrame,
    UiNativeApplicationObservationProgress, UiNativeApplicationPhysicalProgress,
    UiNativeApplicationPreparation, UiNativeApplicationPreparationDenial,
    UiNativeApplicationPreparationDenialCause, UiNativeApplicationPreparationOutcome,
    UiNativeApplicationProgram, UiNativeApplicationProgramDenial,
    UiNativeApplicationReadinessOwnerCount, UiNativeApplicationReadinessOwnerCountDenial,
    UiNativeApplicationReadinessPort, UiNativeApplicationReadinessSignalDenial,
    UiNativeApplicationReadinessSignalDisposition, UiNativeApplicationRuntime,
    UiNativeApplicationRuntimeActivationStopped, UiNativeApplicationRuntimeCloseIncomplete,
    UiNativeApplicationRuntimeClosed, UiNativeApplicationRuntimeDirective,
    UiNativeApplicationRuntimeProgressStopped, UiNativeClientVisualCoordinateOrientation,
    UiNativeClientVisualCoordinateRounding, UiNativeClientVisualPixelColorSpace,
    UiNativeClientVisualSnapshotObservation, UiNativeClientVisualSnapshotRelation,
    UiNativeComponentPresenceChange, UiNativeComponentSemanticTextChange,
    UiNativePlatformCloseReceipt, UiNativePlatformOutcome, UiNativePlatformPreparationDenial,
    UiNativePlatformProfile, UiNativePlatformStopReason, UiNativePlatformStopReport,
    UiNativeThemeTokenValueChange, UiNativeWindowSpec, UiPreparedNativeApplication,
    UiPreparedNativePlatform, WorthUiNativePlatform,
};
#[cfg(feature = "certification-support")]
pub use worth_ui_runtime::native_platform::{
    UiNativeClientAuthoredMountedInstanceObservation, UiNativeClientConditionalOutcome,
    UiNativeClientDerivedStateLossClass, UiNativeClientDerivedStateReconstructionObservation,
    UiNativeClientPresentationSemanticChange,
    UiNativeClientPresentationSemanticFrontierObservation,
    UiNativeClientPresentationSemanticSubscriberObservation,
    UiNativeClientPresentationTransitionKind, UiNativeClientPresentationTransitionObservation,
    UiNativeClientShutdownObservation, UiNativeClientTextPresentationWorkObservation,
    UiNativeDerivedStateLossClass, UiNativeDerivedStateReconstructionObservation,
    UiNativePhysicalSignalExternalStatusClass, UiNativePhysicalSignalObservationOriginClass,
    UiNativePhysicalSignalSettlementClass, UiNativePhysicalSignalTransitionObservation,
    UiNativePhysicalSignalWorkClass, UiNativePresentationObservation, UiNativePresentationWorkKind,
    UiNativeQualificationPlan, UiNativeQualificationPlanDenial, UiNativeRetainedFrameObservation,
    UiNativeRuntimeDerivedStateLossClass, UiNativeRuntimeQualificationPlan,
    UiNativeRuntimeQualificationPlanDenial,
};
