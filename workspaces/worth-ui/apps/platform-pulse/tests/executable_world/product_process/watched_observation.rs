use std::fmt;
use std::process::ExitStatus;
use std::time::{Duration, Instant};

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulseLifecycleObservationEnvelope,
};

use crate::external_observation::{
    PlatformPulseLifecycleStream, PlatformPulseLifecycleStreamFailure,
};

use super::{LivePlatformPulseProcess, PlatformPulseProcessLaunchFailure};

const PROCESS_POLL_SLICE: Duration = Duration::from_millis(25);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WatchedPulseTransition {
    VisualSnapshot,
    VisualIdentityTrace,
    VisualOverlayPublished,
    VisualOverlayCleared,
    VisualSuccessorSnapshot,
    VisualComparison,
    VisualSnapshotRetired,
    GreenReplacement,
    MalformedPreservation,
    CanonicalBlueRecovery,
    RevisionSchemaStopped,
    StatusSchemaRecovered,
    QueryProjectionIssued,
    QueryProjectionPublished,
    IntentInputAdmitted,
    IntentPosturePublished,
    IntentCausalTrace,
    IntentExecutorStarted,
    IntentQueryAction,
    IntentVisualRefreshRetired,
    IntentVisualRefreshCaptured,
    IntentCancellationRebind,
    SemanticFocusPublished,
    PortalDismissed,
}

#[derive(Debug)]
pub(crate) enum WatchedPulseObservationFailure {
    Deadline(WatchedPulseTransition),
    ChildExited {
        expected: WatchedPulseTransition,
        status: ExitStatus,
    },
    ProcessPoll(PlatformPulseProcessLaunchFailure),
    Lifecycle(PlatformPulseLifecycleStreamFailure),
}

pub(crate) fn await_watched_observation(
    process: &mut LivePlatformPulseProcess,
    lifecycle: &mut PlatformPulseLifecycleStream,
    expected: WatchedPulseTransition,
    deadline: Instant,
) -> Result<PlatformPulseLifecycleObservationEnvelope, WatchedPulseObservationFailure> {
    await_observation(process, lifecycle, expected, deadline, true)
}

pub(crate) fn await_next_observation(
    process: &mut LivePlatformPulseProcess,
    lifecycle: &mut PlatformPulseLifecycleStream,
    expected: WatchedPulseTransition,
    deadline: Instant,
) -> Result<PlatformPulseLifecycleObservationEnvelope, WatchedPulseObservationFailure> {
    await_observation(process, lifecycle, expected, deadline, false)
}

fn await_observation(
    process: &mut LivePlatformPulseProcess,
    lifecycle: &mut PlatformPulseLifecycleStream,
    expected: WatchedPulseTransition,
    deadline: Instant,
    filter: bool,
) -> Result<PlatformPulseLifecycleObservationEnvelope, WatchedPulseObservationFailure> {
    loop {
        if let Some(status) = process
            .observed_exit()
            .map_err(WatchedPulseObservationFailure::ProcessPoll)?
        {
            return Err(WatchedPulseObservationFailure::ChildExited { expected, status });
        }
        let now = Instant::now();
        if now >= deadline {
            return Err(WatchedPulseObservationFailure::Deadline(expected));
        }
        let slice_deadline = now
            .checked_add(PROCESS_POLL_SLICE)
            .unwrap_or(deadline)
            .min(deadline);
        match lifecycle.next(slice_deadline) {
            Ok(observation)
                if !filter
                    || matches_transition(expected, observation.outcome())
                    || must_surface_immediately(observation.outcome()) =>
            {
                return Ok(observation)
            }
            Ok(_) => {}
            Err(PlatformPulseLifecycleStreamFailure::Deadline) if slice_deadline < deadline => {}
            Err(PlatformPulseLifecycleStreamFailure::Deadline) => {
                return Err(WatchedPulseObservationFailure::Deadline(expected))
            }
            Err(failure) => return Err(WatchedPulseObservationFailure::Lifecycle(failure)),
        }
    }
}

fn matches_transition(
    expected: WatchedPulseTransition,
    observed: &PlatformPulseLifecycleObservation,
) -> bool {
    use PlatformPulseLifecycleObservation as Observation;
    match expected {
        WatchedPulseTransition::VisualSnapshot
        | WatchedPulseTransition::VisualSuccessorSnapshot
        | WatchedPulseTransition::IntentVisualRefreshCaptured => {
            matches!(observed, Observation::VisualSnapshotCaptured(_))
        }
        WatchedPulseTransition::VisualIdentityTrace => {
            matches!(observed, Observation::VisualPointTrace(_))
        }
        WatchedPulseTransition::VisualOverlayPublished => {
            matches!(observed, Observation::VisualOverlayPublished(_))
        }
        WatchedPulseTransition::VisualOverlayCleared => {
            matches!(observed, Observation::VisualOverlayCleared(_))
        }
        WatchedPulseTransition::VisualComparison => {
            matches!(observed, Observation::VisualComparison(_))
        }
        WatchedPulseTransition::VisualSnapshotRetired
        | WatchedPulseTransition::IntentVisualRefreshRetired => {
            matches!(observed, Observation::VisualSnapshotRetired(_))
        }
        WatchedPulseTransition::GreenReplacement
        | WatchedPulseTransition::CanonicalBlueRecovery
        | WatchedPulseTransition::RevisionSchemaStopped
        | WatchedPulseTransition::StatusSchemaRecovered
        | WatchedPulseTransition::IntentCancellationRebind => {
            matches!(observed, Observation::RebindPublished(_))
        }
        WatchedPulseTransition::MalformedPreservation => {
            matches!(observed, Observation::RebindDeniedPreserving(_))
        }
        WatchedPulseTransition::QueryProjectionIssued => {
            matches!(observed, Observation::QueryProjectionIssued(_))
        }
        WatchedPulseTransition::QueryProjectionPublished => {
            matches!(observed, Observation::QueryProjectionPublished(_))
        }
        WatchedPulseTransition::IntentInputAdmitted => {
            matches!(observed, Observation::IntentInputAdmitted(_))
        }
        WatchedPulseTransition::IntentPosturePublished => {
            matches!(observed, Observation::IntentPosturePublished(_))
        }
        WatchedPulseTransition::IntentCausalTrace => {
            matches!(observed, Observation::IntentCausalTrace(_))
        }
        WatchedPulseTransition::IntentExecutorStarted => {
            matches!(observed, Observation::IntentExecutorStarted(_))
        }
        WatchedPulseTransition::IntentQueryAction => {
            matches!(observed, Observation::QueryAction(_))
        }
        WatchedPulseTransition::SemanticFocusPublished => {
            matches!(observed, Observation::SemanticFocusPublished(_))
        }
        WatchedPulseTransition::PortalDismissed => {
            matches!(observed, Observation::PortalDismissed(_))
        }
    }
}

fn must_surface_immediately(observed: &PlatformPulseLifecycleObservation) -> bool {
    matches!(
        observed,
        PlatformPulseLifecycleObservation::TerminalFailure(_)
            | PlatformPulseLifecycleObservation::ShutdownCompleted(_)
    )
}

impl fmt::Display for WatchedPulseObservationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deadline(expected) => {
                write!(
                    formatter,
                    "{expected:?} watcher observation deadline elapsed"
                )
            }
            Self::ChildExited { expected, status } => {
                write!(formatter, "child exited during {expected:?}: {status}")
            }
            Self::ProcessPoll(failure) => {
                write!(formatter, "poll child during watched transition: {failure}")
            }
            Self::Lifecycle(failure) => {
                write!(formatter, "watched lifecycle observation: {failure}")
            }
        }
    }
}
