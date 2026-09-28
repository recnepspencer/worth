#[cfg(target_os = "windows")]
mod dashboard_first_frame_progression;
#[cfg(target_os = "windows")]
#[cfg(worth_ui_certified_executable)]
mod first_frame_progression;
#[cfg(worth_ui_certified_executable)]
mod installation_progression;
#[cfg(target_os = "windows")]
mod kill_on_close_job;
mod launch;
mod native_close_evidence;
mod native_desktop_lease;
mod native_process_containment;
#[cfg(worth_ui_certified_executable)]
mod normal_close_progression;
mod output_capture;
#[cfg(worth_ui_certified_executable)]
mod progression;
#[cfg(target_os = "windows")]
mod scroll_progression;
mod shutdown;
#[cfg(all(target_os = "windows", target_env = "msvc"))]
mod stack_profile;

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
pub(crate) use native_process_containment::NativeProcessContainment;
#[cfg(worth_ui_certified_executable)]
pub(crate) use progression::{
    AwaitingFirstFrame, Closed, DashboardAtRest, InitialBlue, Installed,
    NativeBoundExecutableWorld, Published, PulseExecutableWorld,
};
#[cfg(target_os = "windows")]
pub(crate) use scroll_progression::{
    PlatformPulseScrollJourneyEvidence, PlatformPulseScrollJourneyFailure,
};
pub(crate) use shutdown::{PlatformPulseProcessExitFailure, SuccessfulPlatformPulseExit};
