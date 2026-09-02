//! SUPPORT AUTHORITY — certification-consumer fixtures.
//!
//! Owned here (not `include!` from `runtime/tests`). External crates must consume fixtures only
//! through `worth-ui-test-support` (feature `certification-support`). Production law is not defined here.

mod activation_interruption;
mod active_session_observation;
mod application_builder;
mod application_graph;
mod application_replacement;
mod builder_host;
mod focus_observation;
mod framework_turn_execution;
mod identity_overlay_projection;
mod intent_evidence;
mod intent_execution_binding;
mod intent_execution_reservation;
mod intent_occupancy;
mod intent_operability_decision;
mod intent_resource_census;
mod intent_route_resolution;
mod layout_admission;
mod local_interaction_recipient;
mod motion_observation;
mod mounted_frame_execution;
mod planning;
mod portal_observation;
mod presentation_async_installation;
mod presentation_mechanics;
mod rebind_identity_lifecycle;
mod runtime_launch;
#[cfg(feature = "certification-support")]
mod runtime_service_scale;
mod scripted_presentation_host;
mod scroll_observation;
mod semantic_text_projection;
mod semantic_text_resolver;
mod service_installation_observation;
mod service_proposal_observation;
mod service_state_observation;
mod shutdown_attempt_observation;
mod touch_origin;
mod touch_origin_source;

pub use crate::admission::{
    UiMeasurementAdmissionPosture, UiMeasurementCapabilityGateReason,
    UiMeasurementUnsupportedReason, UiQueryMeasurementEligibilityPosture,
    UiQueryMeasurementSourceIdentity, UiQueryMeasurementUnsupportedQueryReason,
};
pub use crate::declaration::{UiDeclaredMeasurementBasisSource, UiDeclaredMeasurementMode};
pub use crate::facade::entry::{
    WorthUiLocalInputRecipientCertificationExt, WorthUiMountedAllocationCertificationExt,
    WorthUiMountedAllocationInspectionCertificationExt, WorthUiMountedIdentityCertificationExt,
    WorthUiMountedInteractionLifecycleCertificationExt,
};
pub use crate::graph::{
    UiGraphFactConsumerIdentity, UiGraphFactIndexBasis, UiGraphFactIndexEntry,
    UiGraphFactLookupCost, UiGraphFactLookupDenial, UiGraphFactLookupReceipt,
};
pub use crate::runtime::appearance::UiAppearanceOwnerSnapshot;
pub(crate) use activation_interruption::interrupt_if_armed;
pub use activation_interruption::{
    with_activation_precommit_interruption, WorthUiActivationPrecommitStage,
};
pub use active_session_observation::WorthUiActiveSessionCertificationExt;
pub use application_builder::WorthUiApplicationBuilderCertificationExt;
pub use application_graph::{
    UiRepeatedInstanceIdentityCertificationRow, WorthUiApplicationGraphCertificationExt,
};
pub use application_replacement::WorthUiApplicationReplacementCertificationExt;
pub(crate) use builder_host::UiCertificationBuilderHost;
pub use focus_observation::{
    UiFocusRuntimeCertificationSnapshot, WorthUiFocusRuntimeCertificationExt,
};
pub use framework_turn_execution::WorthUiFrameworkTurnCertificationExt;
pub use identity_overlay_projection::{
    identity_overlay_projection_for_certification, UiIdentityOverlayProjectionCertificationMutation,
};
pub use intent_evidence::WorthUiIntentEvidenceCertificationExt;
pub use intent_execution_binding::{
    UiIntentExecutionBindingRegistrationMetrics, WorthUiIntentExecutionBindingCertificationExt,
};
pub use intent_execution_reservation::{
    UiIntentExecutionCapacityCertificationProfile,
    UiIntentExecutionReservationCertificationMetrics,
    WorthUiIntentExecutionReservationCertificationExt,
};
pub use intent_occupancy::{
    UiIntentOccupancyReleasePosture, UiIntentOccupancyReservation,
    UiIntentOccupancyReservationDenial, WorthUiIntentOccupancyCertificationExt,
};
pub use intent_operability_decision::{
    classify_intent_operability_for_certification, UiIntentOperabilityDecisionCertificationInput,
};
pub use intent_resource_census::WorthUiIntentResourceCensusCertificationExt;
pub use intent_route_resolution::WorthUiIntentRouteResolutionCertificationExt;
#[cfg(test)]
pub(crate) use layout_admission::snapshot_after_layout_admission_support;
pub use local_interaction_recipient::draft_recipient_contract_for_certification;
pub use motion_observation::{
    UiMotionPresentationCertificationSnapshot, WorthUiMotionPresentationCertificationExt,
};
pub use mounted_frame_execution::{
    UiMountedVisualOverlayLeaseCertificationReceipt, WorthUiMountedFrameExecutionCertificationExt,
    WorthUiMountedPublicationCertificationExt,
};
pub use planning::planning_pair_for_certification_suite;
pub use portal_observation::{
    UiPortalDismissalCertificationOutcome, UiPortalDismissalCertificationStop,
    UiPortalExitTerminalCertificationOutcome, UiPortalNestedCertificationOutcome,
    UiPortalRuntimeCertificationSnapshot, WorthUiPortalRuntimeCertificationExt,
};
pub use presentation_async_installation::{
    WorthUiPresentationAsyncInstallationCertificationDenial,
    WorthUiPresentationAsyncInstallationCertificationExt,
};
pub use presentation_mechanics::initial_presentation_mechanics_for_certification;

pub use rebind_identity_lifecycle::{
    identity_lifecycle_decision_for_certification, UiIdentityLifecyclePresence,
    UiRebindPlanningBasisMutation, UiResolvedIdentityLifecycleCertificationExt,
    WorthUiNodeLifecycleTransition,
};
pub use runtime_launch::launch_empty_runtime_for_certification;
#[cfg(feature = "certification-support")]
pub use runtime_service_scale::{runtime_service_scale_evidence, UiRuntimeServiceScaleEvidence};
pub use scripted_presentation_host::{
    presented_completion, recorded_effects, scripted_presentation_epoch, ScriptedPresentationHost,
    ScriptedSurfaceCompletion,
};
pub use scroll_observation::{
    UiScrollObservationCertificationDenial, UiScrollObservationCertificationOutcome,
    WorthUiScrollObservationCertificationExt,
};
pub use semantic_text_projection::{
    empty_projection_for_certification, semantic_text_projection_for_certification,
    semantic_text_projection_for_certification_with_capability,
    semantic_text_projection_for_certification_with_text,
    UiSemanticTextProjectionCertificationMutation,
};
pub use semantic_text_resolver::{
    semantic_text_layout_resolver_for_certification, UiCertificationQualifiedTextResolver,
};
pub use service_installation_observation::{
    UiRuntimeServiceInstallationCertificationSnapshot,
    WorthUiRuntimeServiceInstallationCertificationExt,
};
pub use service_proposal_observation::{
    UiServiceProposalCertificationSnapshot, WorthUiServiceProposalCertificationExt,
};
pub use service_state_observation::{
    UiScrollOwnerGeometryCertificationRow, UiScrollRuntimeCertificationSnapshot,
    UiSelectionRuntimeCertificationSnapshot, WorthUiServiceStateCertificationExt,
};
pub use shutdown_attempt_observation::native_shutdown_attempt_observations_for_certification;
pub use touch_origin::{
    runtime_origin_fixture, WorthUiTouchOriginCertificationFixture,
    WorthUiTouchOriginFixtureVariant,
};
