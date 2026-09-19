mod command;
mod envelope;
mod focus;
mod focus_projection;
mod intent;
mod launch;
mod lifecycle;
mod native_input;
mod projection;
#[cfg(test)]
mod projection_tests;
mod query;
mod query_projection;
mod schema_transition;
mod terminal_projection;
mod theme;
pub use theme::{PlatformPulseThemeSettlementPosture, PlatformPulseThemeSwitchSettled};
mod visual;
mod visual_projection;
#[cfg(test)]
mod visual_tests;
mod visual_value_projection;

pub use command::{
    PlatformPulseCommandLosingCandidateInspection, PlatformPulseCommandLossReasonInspection,
    PlatformPulseCommandScopeInspection, PlatformPulseCommandTransitionInspection,
};
pub use envelope::{
    PlatformPulseDecodedLifecycleObservation, PlatformPulseInheritedLifecycleOnly,
    PlatformPulseLifecycleObservationCodecDenial, PlatformPulseLifecycleObservationEnvelope,
    PlatformPulseLifecycleObservationProtocol, PlatformPulseObservationRunIdentity,
    PlatformPulseObservationSequence, PLATFORM_PULSE_LIFECYCLE_OBSERVATION_IDENTITY,
    PLATFORM_PULSE_LIFECYCLE_OBSERVATION_SCHEMA_VERSION,
    PLATFORM_PULSE_LIFECYCLE_OBSERVATION_STDOUT_PREFIX,
};
pub use focus::{
    PlatformPulseFocusTransitionInspection, PlatformPulseSemanticFocusCause,
    PlatformPulseSemanticFocusOutcome, PlatformPulseSemanticFocusParticipant,
    PlatformPulseSemanticFocusPhysicalOutcome, PlatformPulseSemanticFocusPublished,
};
pub use intent::{
    PlatformPulseIntentAdmissionTrace, PlatformPulseIntentAttemptObservationReference,
    PlatformPulseIntentCausalTraceObservation, PlatformPulseIntentEvidenceReferenceObservation,
    PlatformPulseIntentExecutorGateObservation, PlatformPulseIntentExecutorStartedObservation,
    PlatformPulseIntentInputObservation, PlatformPulseIntentInteractionFamily,
    PlatformPulseIntentOperabilityObservation, PlatformPulseIntentOperabilityTrace,
    PlatformPulseIntentOutcomeTrace, PlatformPulseIntentPayloadTrace,
    PlatformPulseIntentPostureObservation, PlatformPulseIntentPosturePublished,
    PlatformPulseIntentRouteTrace, PlatformPulseIntentRoutingStoppedObservation,
    PlatformPulseIntentSourceTrace, PlatformPulseIntentTraceProjectionDenial,
    PlatformPulseIntentWatcherShutdownEvidence, PlatformPulseQueryActionObservation,
    PlatformPulseQueryActionPreconditionDenial,
};
pub use launch::{
    PlatformPulseLaunchConfigurationDenial, PlatformPulseLaunchConfigurationDenialKind,
};
pub use lifecycle::{
    PlatformPulseApplicationGenerationObservation, PlatformPulseFirstFramePublished,
    PlatformPulseLifecycleObservation, PlatformPulseMountedFrameObservation,
    PlatformPulseNativeRebindDenialStage, PlatformPulseNativeRebindPreparationDenial,
    PlatformPulsePortalDismissed, PlatformPulseProcessStarted,
    PlatformPulseReplacementDenialFamily, PlatformPulseReplacementPreserved,
    PlatformPulseReplacementPublished, PlatformPulseShutdownCompleted,
    PlatformPulseSourceSnapshotObservation, PlatformPulseTerminalFailure,
    PlatformPulseTerminalFailureFamily, PlatformPulseVisualComparison,
    PlatformPulseWatcherBackendObservation,
};
pub use native_input::{PlatformPulseNativeInputIngressPosture, PlatformPulseNativeInputReached};
pub use projection::{
    PlatformPulseLifecycleObservationProjectionDenial, PlatformPulseLifecycleObservationStream,
};
pub use query::{
    PlatformPulseQueryProjectionEvidence, PlatformPulseQueryProjectionPosture,
    PlatformPulseQueryProjectionPublished, PlatformPulseQueryShutdownEvidence,
    PlatformPulseQueryWatcherShutdownEvidence,
};
pub use schema_transition::{
    PlatformPulseProjectionSchemaField, PlatformPulseProjectionSchemaTransitionKind,
    PlatformPulseProjectionSchemaTransitionObservation,
};
pub use visual::{
    PlatformPulseVisualCoordinateObservation, PlatformPulseVisualCoordinateOrientationObservation,
    PlatformPulseVisualCoordinateRoundingObservation, PlatformPulseVisualEvidenceFamilyObservation,
    PlatformPulseVisualEvidenceObservation, PlatformPulseVisualIdentityTraceObservation,
    PlatformPulseVisualMountedNodeObservation, PlatformPulseVisualOverlayCleared,
    PlatformPulseVisualOverlayPublished, PlatformPulseVisualPixelColorSpaceObservation,
    PlatformPulseVisualPixelObservation, PlatformPulseVisualPointResolutionObservation,
    PlatformPulseVisualPointTrace, PlatformPulseVisualSnapshotAffinityObservation,
    PlatformPulseVisualSnapshotCaptured, PlatformPulseVisualSnapshotRelationObservation,
    PlatformPulseVisualSnapshotRetired,
};
pub use visual_projection::{
    PlatformPulseVisualPointObservation, PlatformPulseVisualPointTraceInput,
};
