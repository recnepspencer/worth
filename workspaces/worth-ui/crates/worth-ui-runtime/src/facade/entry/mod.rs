//! Application entry and builder surfaces — first lifecycle capability.

mod active_application_admission;
mod active_application_inspection;
mod active_application_session;
mod active_framework_turn;
mod app;
mod app_builder;
mod app_inspection_routing;
mod appearance_generation_succession;
mod application_replacement;
mod builder;
#[cfg(any(test, feature = "certification-support"))]
mod certification_application_transition;
mod focus_placement;
pub use crate::mounting::{
    UiFocusHostPlacementReconciliationDenial, UiFocusHostPlacementReconciliationOutcome,
    UiFocusHostPlacementReconciliationReceipt, UiFocusHostPlacementShutdownReport,
};
pub use focus_placement::{
    UiFocusPlacementReconciliationExecutionDenial, UiSemanticFocusParticipantObservation,
    UiSemanticFocusPhysicalPlacementOutcome, UiSemanticFocusPublicationCause,
    UiSemanticFocusPublicationOutcome, UiSemanticFocusPublicationReceipt,
};
mod host_neutral_app;
mod intent_admission;
mod intent_confirmation;
mod intent_consequence;
mod intent_consequence_observation;
mod intent_consequence_publication;
mod intent_consequence_rebind;
mod intent_evidence;
mod intent_execution;
mod intent_payload;
mod intent_resource_census;
mod intent_routing;
mod interaction;
mod local_interaction_recipient;
mod measurement_exchange;
mod mounted_allocation_denial;
mod mounted_allocation_establishment;
mod mounted_allocation_inspection;
mod mounted_application_presentation;
mod mounted_content_rebind;
mod mounted_frame_execution;
mod mounted_identity;
mod mounted_inspection;
mod mounted_interaction_lifecycle;
mod mounted_preview;
mod mounted_publication;
#[cfg(test)]
mod native_application_identity_trace_test_support;
#[cfg(test)]
mod native_application_identity_trace_tests;
#[cfg(test)]
mod native_application_presentation_attribution_tests;
mod native_application_program;
mod native_application_shell;
#[cfg(test)]
mod native_identity_trace_audit;
#[cfg(test)]
mod native_identity_trace_host;
mod native_intent;
mod native_intent_evidence;
mod native_intent_execution;
mod native_intent_posture;
mod native_intent_terminal_posture;
mod native_managed_rebind;
mod native_observation_settlement;
pub(crate) mod portal_dismissal;
#[cfg(any(test, feature = "certification-support"))]
mod portal_test_access;
pub(crate) use native_observation_settlement::UiNativeObservationIngressSettlement;
#[cfg(test)]
mod native_observation_tests;
mod native_projection_rebind;
#[cfg(test)]
mod native_projection_rebind_tests;
mod native_replacement_allocation;
mod native_source_rebind;
#[cfg(test)]
mod native_source_rebind_tests;
mod observation;
mod observation_report;
#[cfg(test)]
mod pointer_device_admission_tests;
#[cfg(test)]
mod presentation_currentness_tests;
mod rebind_execution;
mod rebind_recovery;
mod selection_interaction;
#[cfg(test)]
mod typed_pointer_observation_tests;
mod visual_overlay;
mod visual_snapshot;
pub use crate::lifecycle::WorthUiActiveApplicationSessionIdentity;
pub use crate::runtime::exports::WorthUiAllocationCatalogActivationDenial;
pub use crate::runtime::WorthUiActiveApplicationGenerationIdentity;
pub use active_application_inspection::WorthUiActiveInspectionReceipt;
pub use active_application_session::WorthUiActiveApplicationSession;
pub use active_framework_turn::{
    WorthUiActiveCanvasSpatialFrameCompletion, WorthUiActiveFrameworkTurnCompletion,
    WorthUiActiveFrameworkTurnExecution, WorthUiActiveOrdinaryFrameCompletion,
    WorthUiActiveRealtimeFrameCompletion, WorthUiActiveVirtualizedDataFrameCompletion,
    WorthUiMountedLaneProjectionDenial,
};
pub use app::{WorthUi, WorthUiApp};
#[cfg(test)]
pub(crate) use app_builder::WorthUiCertificationApplicationBuilder;
pub use app_builder::{
    UiChangeProfileInstalled, UiChangeProfileMissing, UiIntentProviderRequired,
    UiIntentWiringSatisfied, WorthUiApplicationBuilder, WorthUiProjectionRegistrationError,
    WorthUiQueryViewRegistrationError,
};
pub use application_replacement::{
    WorthUiApplicationCutoverDenial, WorthUiApplicationCutoverReceipt,
    WorthUiApplicationCutoverRetry, WorthUiApplicationPublicationObservation,
    WorthUiApplicationReplacementLoweringDenial, WorthUiApplicationReplacementOutcome,
    WorthUiApplicationReplacementPreparationDenial, WorthUiApplicationReplacementStagingDenial,
    WorthUiApplicationSemanticNoOpReceipt, WorthUiCandidateInspectionReceipt,
    WorthUiLoweredApplicationReplacement, WorthUiMountedApplicationReplacementInFlight,
    WorthUiMountedApplicationReplacementIndeterminate, WorthUiMountedApplicationReplacementOutcome,
    WorthUiMountedReplacementAdmissionDenial, WorthUiMountedReplacementCompletionDenial,
    WorthUiMountedReplacementPreparationOutcome, WorthUiMountedReplacementRetentionDenial,
    WorthUiPendingApplicationCutover, WorthUiPortalExitRetentionPendingKind,
    WorthUiPreparedApplicationReplacement, WorthUiPreparedMountedApplicationReplacement,
    WorthUiReplacementCandidateSummary, WorthUiReplacementPlannedCostEnvelope,
};
pub(crate) use application_replacement::{
    WorthUiDetachedMountedApplicationReplacementInFlight,
    WorthUiDetachedPreparedMountedApplicationReplacement,
};
pub use builder::CapabilityRegistrationBuilder;
#[cfg(any(test, feature = "certification-support"))]
pub use certification_application_transition::WorthUiCertificationApplicationTransition;
pub use host_neutral_app::WorthUiHostNeutralApp;
pub use intent_consequence_publication::{
    UiIntentConsequencePublicationCompletion, UiIntentConsequencePublicationOutcome,
    UiIntentConsequencePublicationReceipt, UiIntentConsequencePublicationRecovery,
};
#[cfg(any(test, feature = "certification-support"))]
pub use local_interaction_recipient::WorthUiLocalInputRecipientCertificationExt;
pub use mounted_allocation_denial::{
    WorthUiMountedAllocationEstablishmentDenial, WorthUiMountedAllocationRuntimeStage,
};
pub use mounted_allocation_establishment::WorthUiMountedAllocationCertificationExt;
pub use mounted_allocation_establishment::{
    UiMountedAllocationMeasurementRequest, WorthUiMountedAllocationEstablishmentReceipt,
};
pub use mounted_allocation_inspection::{
    WorthUiMountedAllocationInspectionCertificationExt,
    WorthUiMountedAllocationProjectionInspectionDenial,
};
pub use mounted_application_presentation::{
    UiMountedHostMeasurementTransitionDenial, UiMountedHostMeasurementUnexpectedTransition,
};
pub(crate) use mounted_content_rebind::{
    WorthUiDetachedMountedContentRebindInFlight, WorthUiDetachedPreparedMountedContentRebind,
    WorthUiMountedContentPublicationReceipt, WorthUiMountedContentRebindInFlight,
    WorthUiMountedContentRebindIndeterminate, WorthUiMountedContentRebindOutcome,
    WorthUiPreparedMountedContentRebind,
};
pub use mounted_frame_execution::{
    WorthUiMountedFrameExecutionStop, WorthUiMountedFrameFrameworkTransitionStop,
};
pub use mounted_identity::WorthUiMountedIdentityCertificationExt;
pub use mounted_interaction_lifecycle::{
    UiSurfaceRebindInteractionDenial, UiSurfaceRebindInteractionReceipt,
    WorthUiMountedInteractionLifecycleCertificationExt,
};
pub use mounted_preview::{
    WorthUiMountedPreviewAdmissionRejection, WorthUiMountedPreviewCompletionRejection,
    WorthUiMountedPreviewDisposition, WorthUiMountedPreviewInFlight, WorthUiMountedPreviewOutcome,
    WorthUiMountedPreviewPreparationDenial, WorthUiMountedPreviewPreparationRejection,
    WorthUiMountedPreviewRetentionRejection, WorthUiPendingMountedPreview,
    WorthUiPreparedMountedPreview, WorthUiResolvedMountedPreview,
};
pub use native_application_program::{
    UiNativeApplicationFrame, UiNativeApplicationProgram, UiNativeApplicationProgramDenial,
    UiNativeComponentPresenceChange, UiNativeComponentSemanticTextChange,
    UiNativeThemeTokenValueChange,
};
pub(crate) use native_application_shell::{
    UiNativeApplicationQueryCloseObservation, UiNativeComponentPresenceProgress,
};
pub use native_application_shell::{
    WorthUiNativeApplicationCleanup, WorthUiNativeApplicationShell,
    WorthUiNativeApplicationShellLaunchDenial, WorthUiNativeApplicationShutdownReceipt,
    WorthUiNativePhysicalPresentationRecovery, WorthUiNativePresentationRecoveryDenial,
    WorthUiNativeReducedMotionPosture,
};
pub use native_intent::{
    WorthUiNativeIntentAttemptPrepared, WorthUiNativeIntentConfirmationRequired,
    WorthUiNativeIntentIngress, WorthUiNativeIntentPosture, WorthUiNativeIntentPostureKind,
    WorthUiNativeIntentStop, WorthUiNativeIntentStopped, WorthUiNativeIntentTransition,
    WorthUiNativeInteractionIngressStop,
};
pub use native_intent_execution::{
    WorthUiNativeManagedIntentConsequencePublicationDenial,
    WorthUiNativeManagedIntentConsequencePublicationOutcome,
};
pub use native_intent_posture::{
    WorthUiNativeIntentPosturePublicationCompletion, WorthUiNativeIntentPosturePublicationOutcome,
    WorthUiNativeIntentPosturePublicationRecovery, WorthUiNativeIntentPosturePublicationRetry,
    WorthUiNativeIntentPosturePublicationStop, WorthUiNativeManagedIntentPosturePublicationDenial,
    WorthUiNativeManagedIntentPosturePublicationOutcome,
};
pub use native_intent_terminal_posture::WorthUiNativeIntentTerminalPostureOutcome;
pub use native_managed_rebind::{
    WorthUiNativeManagedPortalDismissalOutcome, WorthUiNativeManagedRebindDenial,
    WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindStop,
    WorthUiNativePortalDismissalStop, WorthUiNativePredecessorRecovery,
};
pub use native_projection_rebind::{
    WorthUiNativeManagedProjectionRebindOutcome, WorthUiNativeProjectionRebindDenial,
};
pub use native_source_rebind::{
    WorthUiNativeManagedSourceRebindOutcome, WorthUiNativeSourceRebindDenial,
};
pub use portal_dismissal::UiPortalDismissalPublicationReceipt;
pub(crate) use rebind_execution::WorthUiPreparedEvidenceOnlyApplicationRebind;
pub(crate) use rebind_recovery::WorthUiRebindRecoveryAuthority;
pub use selection_interaction::UiCurrentProjectionOptionStop;
