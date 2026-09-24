use std::fmt;

use crate::lifecycle_observation_publication::PlatformPulseObservationPublicationDenial;

/// Pulse runs until one error stops it; only a stopped Pulse has an error,
/// terminal evidence whose publication may itself have been denied, and a
/// report that is written once.
pub(super) enum PlatformPulseTerminalPosture {
    Running,
    Stopped {
        error: PlatformPulseTerminalError,
        observation: Option<PlatformPulseObservationPublicationDenial>,
        reported: bool,
    },
}

impl PlatformPulseTerminalPosture {
    pub(super) const fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    pub(super) const fn is_stopped(&self) -> bool {
        !self.is_running()
    }
}

pub(super) enum PlatformPulseTerminalError {
    FocusPlacement(worth_ui::facade::app::UiFocusPlacementExecutionDenial),
    ThemePreference(crate::theme_preference::PlatformPulseThemePreferenceDenial),
    ThemeSwitch(super::theme::PlatformPulseThemeSwitchDenial),
    SourceWatcher(worth_ui::facade::source::WorthUiFilesystemWatcherDenial),
    FrameExecution(String),
    ProductCopy(worth_ui_native_platform::UiNativeApplicationProgramDenial),
    UnexpectedInitialFrame,
    NativeRebind(worth_ui::facade::app::WorthUiNativeSourceRebindDenial),
    NativeManagedSourceRebind(worth_ui::facade::app::WorthUiNativeManagedRebindStop),
    NativeManagedProgress(worth_ui::facade::app::WorthUiNativeManagedRebindDenial),
    NativeManagedAttribution(&'static str),
    NativeRecoveredWithoutPublication,
    NativeProjection(super::projection::PlatformPulseProjectionRebindDenial),
    QueryLifecycle(crate::query_source::PlatformPulseQueryLifecycleDenial),
    QueryWatch(crate::query_source::PlatformPulseExternalValueWatchDenial),
    IntentWatch(worth_ui_platform_pulse::intent::PlatformPulseIntentInputWatchDenial),
    IntentGate(worth_ui_platform_pulse::intent::PlatformPulseExecutorGateRevisionDenial),
    IntentFact(worth_ui::facade::intent::UiIntentApplicationFactUpdateDenial),
    IntentClock(super::intent::PlatformPulseIntentClockDenial),
    IntentPosturePublication(super::intent::PlatformPulseIntentPosturePublicationDenial),
    IntentExecution(String),
    VisualIdentity(crate::visual_identity_execution::PlatformPulseVisualExecutionDenial),
    ObservationPublication,
}

impl fmt::Display for PlatformPulseTerminalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FocusPlacement(denial) => write!(formatter, "native Focus placement: {denial:?}"),
            Self::ThemePreference(denial) => write!(formatter, "theme preference: {denial:?}"),
            Self::ThemeSwitch(denial) => write!(formatter, "theme switch: {denial}"),
            Self::SourceWatcher(denial) => write!(formatter, "source watcher: {denial:?}"),
            Self::FrameExecution(detail) => {
                write!(formatter, "mounted frame execution: {detail}")
            }
            Self::ProductCopy(denial) => {
                write!(formatter, "initial product semantic copy: {denial:?}")
            }
            Self::UnexpectedInitialFrame => formatter.write_str("initial frame did not publish"),
            Self::NativeRebind(denial) => write!(formatter, "native source rebind: {denial:?}"),
            Self::NativeManagedSourceRebind(stop) => {
                write!(formatter, "native source rebind did not publish: {stop:?}")
            }
            Self::NativeManagedProgress(denial) => {
                write!(formatter, "native managed publication progress: {denial:?}")
            }
            Self::NativeRecoveredWithoutPublication => {
                formatter.write_str("native predecessor recovered; successor remained unpublished")
            }
            Self::NativeManagedAttribution(detail) => {
                write!(
                    formatter,
                    "native managed publication attribution: {detail}"
                )
            }
            Self::NativeProjection(denial) => {
                write!(formatter, "native projection rebind: {denial}")
            }
            Self::QueryLifecycle(denial) => write!(formatter, "Query lifecycle: {denial}"),
            Self::QueryWatch(denial) => write!(formatter, "Query source watch: {denial}"),
            Self::IntentWatch(denial) => write!(formatter, "intent source watch: {denial}"),
            Self::IntentGate(denial) => write!(
                formatter,
                "intent gate revision {} is not after {}",
                denial.submitted(),
                denial.active()
            ),
            Self::IntentFact(denial) => write!(formatter, "intent fact update: {denial:?}"),
            Self::IntentClock(denial) => write!(formatter, "intent clock: {denial}"),
            Self::IntentPosturePublication(denial) => {
                write!(formatter, "intent posture publication: {denial}")
            }
            Self::IntentExecution(detail) => write!(formatter, "intent execution: {detail}"),
            Self::VisualIdentity(denial) => write!(formatter, "visual identity pulse: {denial}"),
            Self::ObservationPublication => {
                formatter.write_str("lifecycle observation publication")
            }
        }
    }
}
