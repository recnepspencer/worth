#[cfg(worth_ui_certified_executable)]
mod appearance_pixels;
#[cfg(worth_ui_certified_executable)]
mod content_fingerprint;
#[cfg(worth_ui_certified_executable)]
pub(crate) mod dashboard_visual_oracle;
#[cfg(worth_ui_certified_executable)]
mod identity_trace;
#[cfg(worth_ui_certified_executable)]
mod intent_control_points;
#[cfg(worth_ui_certified_executable)]
mod lifecycle_cleanup;
#[cfg(worth_ui_certified_executable)]
mod native_color;
#[cfg(worth_ui_certified_executable)]
mod native_input_reachability;
#[cfg(worth_ui_certified_executable)]
mod portal_pixels;
#[cfg(worth_ui_certified_executable)]
mod predecessor_preservation;
#[cfg(worth_ui_certified_executable)]
mod publication_identity;
#[cfg(worth_ui_certified_executable)]
mod query_to_pixel;
#[cfg(worth_ui_certified_executable)]
mod replacement_to_pixel;
#[cfg(worth_ui_certified_executable)]
mod runtime_service_story_pixels;
#[cfg(worth_ui_certified_executable)]
mod schema_transition;
#[cfg(target_os = "windows")]
mod scroll_chrome_pixel_failure;
#[cfg(target_os = "windows")]
mod scroll_chrome_pixels;
#[cfg(target_os = "windows")]
#[cfg(worth_ui_certified_executable)]
mod source_to_pixel;
#[cfg(worth_ui_certified_executable)]
mod status_badge_pixels;
mod visual_contract_manifest;
#[cfg(worth_ui_certified_executable)]
mod visual_overlay_pixels;
#[cfg(worth_ui_certified_executable)]
mod wrapping_text_pixels;

#[cfg(worth_ui_certified_executable)]
pub(crate) use appearance_pixels::{
    adjudicate_first_frame_appearance, FirstFrameAppearanceFailure,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use content_fingerprint::content_fingerprint;
#[cfg(worth_ui_certified_executable)]
pub(crate) use identity_trace::{
    adjudicate_successor_visual_snapshot, adjudicate_visual_comparison,
    adjudicate_visual_retirement, adjudicate_visual_snapshot, adjudicate_visual_trace,
    ExecutableVisualIdentityFailure, ExecutableVisualRetirementEvidence,
    ExecutableVisualSnapshotEvidence, ExecutableVisualTraceEvidence,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use intent_control_points::{
    adjudicate_action_control_point, adjudicate_confirmation_control_point,
    adjudicate_portal_control_point, adjudicate_portal_control_point_for_extent,
    adjudicate_visible_control_change, require_distinct_control_points, IntentControlPointFailure,
    NativeControlPixelRegion, PlatformPulseActionControlPoint,
    PlatformPulseConfirmationControlPoint, VisibleControlPixelChange,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use lifecycle_cleanup::{
    adjudicate_lifecycle_cleanup, CausalLifecycleCleanupObservationSet,
    ExecutableLifecycleCleanupEvidence, ExecutableLifecycleCleanupFailure,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use native_color::{
    adjudicate_native_background_point, adjudicate_native_color, ExpectedNativeColor,
    NativeColorFailure, NativeColorVerdict,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use native_input_reachability::{
    adjudicate_native_input_reachability, native_input_background_point,
    ExecutableNativeInputReachabilityEvidence, ExecutableNativeInputReachabilityFailure,
    NativeInputFamilyObservation, NativeInputReachabilityObservationSet,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use portal_pixels::{
    adjudicate_authored_portal_pixels, adjudicate_closed_portal_pixels,
    adjudicate_focus_fallback_portal_pixels, adjudicate_open_portal_pixels, portal_action_points,
    portal_occupancy_point, PlatformPulseAuthoredPortalPixelEvidence,
    PlatformPulsePortalFocusFallbackPixelEvidence, PlatformPulsePortalPixelEvidence,
    PlatformPulsePortalPixelFailure,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use predecessor_preservation::{
    adjudicate_predecessor_preservation, CausalPredecessorPreservationObservationSet,
    ExecutablePredecessorPreservationEvidence, ExecutablePredecessorPreservationFailure,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use publication_identity::ExecutablePublishedIdentity;
#[cfg(worth_ui_certified_executable)]
pub(crate) use query_to_pixel::{
    adjudicate_query_current, ExecutableQueryCurrentEvidence, ExecutableQueryCurrentFailure,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use replacement_to_pixel::{
    adjudicate_replacement, require_replacement_lifecycle, CausalReplacementObservationSet,
    ExecutableReplacementEvidence, ExecutableReplacementFailure, ReplacementExpectation,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use runtime_service_story_pixels::{
    adjudicate_runtime_service_story_pixels, PlatformPulseRuntimeServicePixelEvidence,
    PlatformPulseRuntimeServicePixelFailure,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use schema_transition::{
    adjudicate_schema_transition, schema_posture_changed_pixel_bytes, schema_posture_matches,
    ExecutableSchemaTransitionEvidence, ExecutableSchemaTransitionFailure,
    ExpectedSchemaTransition,
};
#[cfg(target_os = "windows")]
pub(crate) use scroll_chrome_pixel_failure::ScrollChromePixelFailure;
#[cfg(target_os = "windows")]
pub(crate) use scroll_chrome_pixels::{
    adjudicate_content_shift, adjudicate_vertical_thumb, notch_points, physical_px,
    platform_wheel_lines_per_notch, ContentShiftEvidence, RecentActivityScrollGeometry,
    VerticalThumbEvidence,
};
#[cfg(target_os = "windows")]
#[cfg(worth_ui_certified_executable)]
pub(crate) use source_to_pixel::{
    adjudicate_first_frame, adjudicate_source_signal_first_frame, CausalFirstFrameObservationSet,
    ExecutableFirstFrameEvidence, ExecutableFirstFrameFailure,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use status_badge_pixels::{
    adjudicate_dashboard_status_badge, DashboardStatusBadgeFailure,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use visual_overlay_pixels::{
    adjudicate_overlay_pixels, adjudicate_restored_pixels, ExecutableVisualClearEvidence,
    ExecutableVisualOverlayEvidence,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use wrapping_text_pixels::{
    adjudicate_resized_wrapping_text, PlatformPulseWrappingTextFailure,
};
