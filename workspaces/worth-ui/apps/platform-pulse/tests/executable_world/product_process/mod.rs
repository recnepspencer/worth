#[cfg(target_os = "windows")]
mod dashboard_first_frame_progression;
#[cfg(target_os = "windows")]
#[cfg(worth_ui_certified_executable)]
mod first_frame_progression;
#[cfg(worth_ui_certified_executable)]
mod installation_progression;
#[cfg(worth_ui_certified_executable)]
mod intent_progression;
#[cfg(target_os = "windows")]
mod kill_on_close_job;
mod launch;
mod native_close_evidence;
mod native_desktop_lease;
#[cfg(worth_ui_certified_executable)]
mod native_input_progression;
mod native_process_containment;
#[cfg(worth_ui_certified_executable)]
mod normal_close_progression;
mod output_capture;
#[cfg(worth_ui_certified_executable)]
mod portal_progression;
#[cfg(worth_ui_certified_executable)]
mod preservation_progression;
#[cfg(worth_ui_certified_executable)]
mod progression;
#[cfg(worth_ui_certified_executable)]
mod query_progression;
#[cfg(worth_ui_certified_executable)]
mod quiescent_observation;
#[cfg(worth_ui_certified_executable)]
mod replacement_progression;
#[cfg(worth_ui_certified_executable)]
mod schema_transition_progression;
#[cfg(target_os = "windows")]
mod scroll_progression;
mod shutdown;
#[cfg(worth_ui_certified_executable)]
mod source_action_progression;
#[cfg(all(target_os = "windows", target_env = "msvc"))]
mod stack_profile;
#[cfg(worth_ui_certified_executable)]
mod visual_snapshot_progression;
#[cfg(worth_ui_certified_executable)]
mod watched_native_observation;
mod watched_observation;

#[cfg(worth_ui_certified_executable)]
pub(crate) use intent_progression::{
    PlatformPulseIntentJourneyEvidence, PlatformPulseIntentJourneyFailure,
};
pub(crate) use launch::{
    CargoBuiltPlatformPulse, EmergencyPlatformPulseExit, EmergencyPlatformPulseExitFailure,
    LivePlatformPulseProcess, NativePhase2ProcessLaunch, PlatformPulseProcessLaunchFailure,
};
// Portable: the product writes the same evidence file on every platform and
// the launcher hands it the path on every platform.
pub(crate) use native_close_evidence::{
    PlatformPulseNativeCloseEvidence, PlatformPulseNativeCloseEvidenceFailure,
    PlatformPulseNativeSampleFrameEvidence, NATIVE_CLOSE_EVIDENCE_FILE_NAME,
    NATIVE_CLOSE_EVIDENCE_PATH_ENVIRONMENT,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use native_input_progression::NativeInputCausalStep;
pub(crate) use native_process_containment::NativeProcessContainment;
#[cfg(worth_ui_certified_executable)]
pub(crate) use portal_progression::{
    PlatformPulsePortalJourneyEvidence, PlatformPulsePortalJourneyFailure,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use progression::{
    AwaitingFirstFrame, AwaitingPreservation, AwaitingQueryCurrent, AwaitingRecovery,
    AwaitingReplacement, AwaitingSchemaStop, AwaitingStatusRecovery, Closed,
    ComparisonBasisRefreshed, DashboardAtRest, FinalRecovered, FirstCurrent, GreenSuccessor,
    IdentityTraced, InitialBlue, Installed, NativeBoundExecutableWorld, NativeInputReached,
    OverlayCleared, OverlayPublished, PortalReady, PreservedPredecessor,
    PreservedPredecessorEvidence, Published, PulseExecutableWorld, QueryCurrent, RecoveredBlue,
    SchemaStopped, SecondCurrent, SecondQueryCurrent, SnapshotCaptured,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use quiescent_observation::PlatformPulseQuiescenceFailure;
#[cfg(target_os = "windows")]
pub(crate) use scroll_progression::{
    PlatformPulseScrollJourneyEvidence, PlatformPulseScrollJourneyFailure,
};
pub(crate) use shutdown::{PlatformPulseProcessExitFailure, SuccessfulPlatformPulseExit};
pub(crate) use watched_observation::{
    await_next_observation, await_watched_observation, WatchedPulseObservationFailure,
    WatchedPulseTransition,
};
