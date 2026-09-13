#[cfg(target_os = "windows")]
mod first_frame_progression;
#[cfg(target_os = "windows")]
mod installation_progression;
#[cfg(target_os = "windows")]
mod intent_progression;
#[cfg(target_os = "windows")]
mod kill_on_close_job;
mod launch;
#[cfg(target_os = "windows")]
mod native_close_evidence;
#[cfg(target_os = "windows")]
mod native_desktop_lease;
#[cfg(target_os = "windows")]
mod native_input_progression;
#[cfg(target_os = "windows")]
mod normal_close_progression;
mod output_capture;
#[cfg(target_os = "windows")]
mod portal_progression;
#[cfg(target_os = "windows")]
mod preservation_progression;
#[cfg(target_os = "windows")]
mod progression;
#[cfg(target_os = "windows")]
mod query_progression;
#[cfg(target_os = "windows")]
mod quiescent_observation;
#[cfg(target_os = "windows")]
mod replacement_progression;
#[cfg(target_os = "windows")]
mod schema_transition_progression;
mod shutdown;
#[cfg(target_os = "windows")]
mod source_action_progression;
#[cfg(all(target_os = "windows", target_env = "msvc"))]
mod stack_profile;
#[cfg(target_os = "windows")]
mod visual_snapshot_progression;
#[cfg(target_os = "windows")]
mod watched_native_observation;
mod watched_observation;

#[cfg(target_os = "windows")]
pub(crate) use intent_progression::{
    PlatformPulseIntentJourneyEvidence, PlatformPulseIntentJourneyFailure,
};
pub(crate) use launch::{
    CargoBuiltPlatformPulse, EmergencyPlatformPulseExit, EmergencyPlatformPulseExitFailure,
    LivePlatformPulseProcess, NativePhase2ProcessLaunch, PlatformPulseProcessLaunchFailure,
};
#[cfg(target_os = "windows")]
pub(crate) use native_close_evidence::{
    PlatformPulseNativeCloseEvidence, PlatformPulseNativeCloseEvidenceFailure,
    PlatformPulseNativeSampleFrameEvidence, NATIVE_CLOSE_EVIDENCE_FILE_NAME,
    NATIVE_CLOSE_EVIDENCE_PATH_ENVIRONMENT,
};
#[cfg(target_os = "windows")]
pub(crate) use native_input_progression::NativeInputCausalStep;
#[cfg(target_os = "windows")]
pub(crate) use portal_progression::{
    PlatformPulsePortalJourneyEvidence, PlatformPulsePortalJourneyFailure,
};
#[cfg(target_os = "windows")]
pub(crate) use progression::{
    AwaitingFirstFrame, AwaitingPreservation, AwaitingQueryCurrent, AwaitingRecovery,
    AwaitingReplacement, AwaitingSchemaStop, AwaitingStatusRecovery, Closed,
    ComparisonBasisRefreshed, FinalRecovered, FirstCurrent, GreenSuccessor, IdentityTraced,
    InitialBlue, Installed, NativeBoundExecutableWorld, NativeInputReached, OverlayCleared,
    OverlayPublished, PortalReady, PreservedPredecessor, PreservedPredecessorEvidence, Published,
    PulseExecutableWorld, QueryCurrent, RecoveredBlue, SchemaStopped, SecondCurrent,
    SecondQueryCurrent, SnapshotCaptured,
};
#[cfg(target_os = "windows")]
pub(crate) use quiescent_observation::PlatformPulseQuiescenceFailure;
pub(crate) use shutdown::{PlatformPulseProcessExitFailure, SuccessfulPlatformPulseExit};
pub(crate) use watched_observation::{
    await_next_observation, await_watched_observation, WatchedPulseObservationFailure,
    WatchedPulseTransition,
};
