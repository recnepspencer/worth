#[cfg(target_os = "windows")]
mod appearance_pixels;
#[cfg(target_os = "windows")]
mod content_fingerprint;
#[cfg(target_os = "windows")]
mod identity_trace;
#[cfg(target_os = "windows")]
mod intent_control_points;
#[cfg(target_os = "windows")]
mod lifecycle_cleanup;
#[cfg(target_os = "windows")]
mod native_color;
#[cfg(target_os = "windows")]
mod native_input_reachability;
#[cfg(target_os = "windows")]
mod platform_pulse_control_points;
#[cfg(target_os = "windows")]
mod portal_pixels;
#[cfg(target_os = "windows")]
mod predecessor_preservation;
#[cfg(target_os = "windows")]
mod publication_identity;
#[cfg(target_os = "windows")]
mod query_to_pixel;
#[cfg(target_os = "windows")]
mod replacement_to_pixel;
#[cfg(target_os = "windows")]
mod runtime_service_story_pixels;
#[cfg(target_os = "windows")]
mod schema_transition;
#[cfg(target_os = "windows")]
mod source_to_pixel;
mod visual_contract_manifest;
#[cfg(target_os = "windows")]
mod visual_overlay_pixels;
#[cfg(target_os = "windows")]
mod wrapping_text_pixels;

#[cfg(target_os = "windows")]
pub(crate) use appearance_pixels::{
    adjudicate_first_frame_appearance, FirstFrameAppearanceFailure,
};
#[cfg(target_os = "windows")]
pub(crate) use content_fingerprint::content_fingerprint;
#[cfg(target_os = "windows")]
pub(crate) use identity_trace::{
    adjudicate_successor_visual_snapshot, adjudicate_visual_comparison,
    adjudicate_visual_retirement, adjudicate_visual_snapshot, adjudicate_visual_trace,
    ExecutableVisualComparisonEvidence, ExecutableVisualIdentityFailure,
    ExecutableVisualRetirementEvidence, ExecutableVisualSnapshotEvidence,
    ExecutableVisualTraceEvidence,
};
#[cfg(target_os = "windows")]
pub(crate) use intent_control_points::{
    adjudicate_action_control_point, adjudicate_confirmation_control_point,
    adjudicate_portal_control_point, adjudicate_visible_control_change,
    require_distinct_control_points, IntentControlPointFailure, NativeControlPixelRegion,
    PlatformPulseActionControlPoint, PlatformPulseConfirmationControlPoint,
    VisibleControlPixelChange,
};
#[cfg(target_os = "windows")]
pub(crate) use lifecycle_cleanup::{
    adjudicate_lifecycle_cleanup, CausalLifecycleCleanupObservationSet,
    ExecutableLifecycleCleanupEvidence, ExecutableLifecycleCleanupFailure,
};
#[cfg(target_os = "windows")]
pub(crate) use native_color::{
    adjudicate_native_background_point, adjudicate_native_color, ExpectedNativeColor,
    NativeColorFailure, NativeColorVerdict,
};
#[cfg(target_os = "windows")]
pub(crate) use native_input_reachability::{
    adjudicate_native_input_reachability, native_input_background_point,
    ExecutableNativeInputReachabilityEvidence, ExecutableNativeInputReachabilityFailure,
    NativeInputFamilyObservation, NativeInputReachabilityObservationSet,
};
#[cfg(target_os = "windows")]
pub(crate) use portal_pixels::{
    adjudicate_authored_portal_pixels, adjudicate_closed_portal_pixels,
    adjudicate_focus_fallback_portal_pixels, adjudicate_open_portal_pixels, portal_action_points,
    portal_occupancy_point, PlatformPulseAuthoredPortalPixelEvidence,
    PlatformPulsePortalFocusFallbackPixelEvidence, PlatformPulsePortalPixelEvidence,
    PlatformPulsePortalPixelFailure,
};
#[cfg(target_os = "windows")]
pub(crate) use predecessor_preservation::{
    adjudicate_predecessor_preservation, CausalPredecessorPreservationObservationSet,
    ExecutablePredecessorPreservationEvidence, ExecutablePredecessorPreservationFailure,
};
#[cfg(target_os = "windows")]
pub(crate) use publication_identity::ExecutablePublishedIdentity;
#[cfg(target_os = "windows")]
pub(crate) use query_to_pixel::{
    adjudicate_query_current, ExecutableQueryCurrentEvidence, ExecutableQueryCurrentFailure,
};
#[cfg(target_os = "windows")]
pub(crate) use replacement_to_pixel::{
    adjudicate_replacement, require_replacement_lifecycle, CausalReplacementObservationSet,
    ExecutableReplacementEvidence, ExecutableReplacementFailure, ReplacementExpectation,
};
#[cfg(target_os = "windows")]
pub(crate) use runtime_service_story_pixels::{
    adjudicate_runtime_service_story_pixels, PlatformPulseRuntimeServicePixelEvidence,
    PlatformPulseRuntimeServicePixelFailure,
};
#[cfg(target_os = "windows")]
pub(crate) use schema_transition::{
    adjudicate_schema_transition, schema_posture_changed_pixel_bytes, schema_posture_matches,
    ExecutableSchemaTransitionEvidence, ExecutableSchemaTransitionFailure,
    ExpectedSchemaTransition,
};
#[cfg(target_os = "windows")]
pub(crate) use source_to_pixel::{
    adjudicate_first_frame, CausalFirstFrameObservationSet, ExecutableFirstFrameEvidence,
    ExecutableFirstFrameFailure,
};
#[cfg(target_os = "windows")]
pub(crate) use visual_overlay_pixels::{
    adjudicate_overlay_pixels, adjudicate_restored_pixels, ExecutableVisualClearEvidence,
    ExecutableVisualOverlayEvidence,
};
#[cfg(target_os = "windows")]
pub(crate) use wrapping_text_pixels::{
    adjudicate_default_wrapping_text, adjudicate_resized_wrapping_text,
    PlatformPulseWrappingTextFailure,
};
