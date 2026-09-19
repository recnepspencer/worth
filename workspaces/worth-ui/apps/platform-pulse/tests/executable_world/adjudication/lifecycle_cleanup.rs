use std::fmt;

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulseLifecycleObservationEnvelope,
    PlatformPulseShutdownCompleted,
};

use crate::external_observation::{
    LifecycleStreamMeasurement, NormalNativeCloseRequestObservation,
};
use crate::installation::PulseInstallationCleanupEvidence;
use crate::product_process::PlatformPulseNativeCloseEvidence;
use crate::product_process::SuccessfulPlatformPulseExit;

#[derive(Debug)]
pub(crate) struct ExecutableLifecycleCleanupEvidence {
    close_request: NormalNativeCloseRequestObservation,
    shutdown_envelope: PlatformPulseLifecycleObservationEnvelope,
    shutdown: PlatformPulseShutdownCompleted,
    lifecycle_measurement: LifecycleStreamMeasurement,
    lifecycle_envelopes: Vec<PlatformPulseLifecycleObservationEnvelope>,
    successful_exit: SuccessfulPlatformPulseExit,
    native_close_evidence: PlatformPulseNativeCloseEvidence,
    installation_cleanup: PulseInstallationCleanupEvidence,
}

pub(crate) struct CausalLifecycleCleanupObservationSet {
    process_id: u32,
    close_request: NormalNativeCloseRequestObservation,
    shutdown_envelope: PlatformPulseLifecycleObservationEnvelope,
    lifecycle_measurement: LifecycleStreamMeasurement,
    lifecycle_envelopes: Vec<PlatformPulseLifecycleObservationEnvelope>,
}

pub(crate) struct ExecutableLifecycleCleanupObservationSet {
    causal: CausalLifecycleCleanupObservationSet,
    successful_exit: SuccessfulPlatformPulseExit,
    native_close_evidence: PlatformPulseNativeCloseEvidence,
    installation_cleanup: PulseInstallationCleanupEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExecutableLifecycleCleanupFailure {
    MissingShutdownCompletion,
    ProcessIdentityMismatch,
    HostSessionNotReleased,
    QueryWatcherNotJoined,
    PendingQueryObservations(u64),
    IntentWatcherNotJoined,
    ThemeWatchNotReleased,
    PendingIntentInputs(u64),
    IntentResourcesNotEmpty,
    QueryCloseIncomplete,
    QueryOwnerNotTerminal,
    MountedPresentationNotQuiescent(u64),
    VisualCaptureResidue {
        cancelled: u64,
        disposed_snapshots: u64,
        disposed_pixel_bytes: u64,
        disposed_structural_bytes: u64,
    },
    VisualOverlayResidue {
        pending: u64,
        published: u64,
        clearing: u64,
    },
    NoReleasedSurface,
    InstallationResidue,
}

impl fmt::Display for ExecutableLifecycleCleanupFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ThemeWatchNotReleased => formatter.write_str("theme preference watch was not released"),
            Self::MissingShutdownCompletion => {
                formatter.write_str("terminal lifecycle event was not shutdown completion")
            }
            Self::ProcessIdentityMismatch => {
                formatter.write_str("normal-close request belongs to a different process")
            }
            Self::HostSessionNotReleased => formatter.write_str("host session was not released"),
            Self::QueryWatcherNotJoined => {
                formatter.write_str("Query source watcher worker was not joined")
            }
            Self::PendingQueryObservations(count) => {
                write!(formatter, "{count} Query source observation(s) remained queued")
            }
            Self::IntentWatcherNotJoined => {
                formatter.write_str("intent input watcher worker was not joined")
            }
            Self::PendingIntentInputs(count) => {
                write!(formatter, "{count} intent input observation(s) remained queued")
            }
            Self::IntentResourcesNotEmpty => {
                formatter.write_str("runtime intent resources were not empty at shutdown")
            }
            Self::QueryCloseIncomplete => {
                formatter.write_str("runtime Query close was incomplete at shutdown")
            }
            Self::QueryOwnerNotTerminal => formatter.write_str("Query owner did not close"),
            Self::MountedPresentationNotQuiescent(count) => write!(
                formatter,
                "{count} exceptional mounted presentation shutdown attempt(s) remained"
            ),
            Self::VisualCaptureResidue {
                cancelled,
                disposed_snapshots,
                disposed_pixel_bytes,
                disposed_structural_bytes,
            } => write!(
                formatter,
                "visual capture residue: cancelled={cancelled}, snapshots={disposed_snapshots}, pixel_bytes={disposed_pixel_bytes}, structural_bytes={disposed_structural_bytes}"
            ),
            Self::VisualOverlayResidue {
                pending,
                published,
                clearing,
            } => write!(
                formatter,
                "visual overlay residue: pending={pending}, published={published}, clearing={clearing}"
            ),
            Self::NoReleasedSurface => formatter.write_str("shutdown released no host surface"),
            Self::InstallationResidue => {
                formatter.write_str("isolated source installation remained after cleanup")
            }
        }
    }
}

pub(crate) fn adjudicate_lifecycle_cleanup(
    observations: ExecutableLifecycleCleanupObservationSet,
) -> Result<ExecutableLifecycleCleanupEvidence, ExecutableLifecycleCleanupFailure> {
    let ExecutableLifecycleCleanupObservationSet {
        causal,
        successful_exit,
        native_close_evidence,
        installation_cleanup,
    } = observations;
    let shutdown = match causal.shutdown_envelope.outcome() {
        PlatformPulseLifecycleObservation::ShutdownCompleted(shutdown) => *shutdown,
        _ => {
            return Err(ExecutableLifecycleCleanupFailure::MissingShutdownCompletion);
        }
    };
    if causal.close_request.process_id() != causal.process_id {
        return Err(ExecutableLifecycleCleanupFailure::ProcessIdentityMismatch);
    }
    if !shutdown.host_session_released() {
        return Err(ExecutableLifecycleCleanupFailure::HostSessionNotReleased);
    }
    if !shutdown.theme_watch_released() {
        return Err(ExecutableLifecycleCleanupFailure::ThemeWatchNotReleased);
    }
    require_zero_intent_residue(shutdown)?;
    require_zero_query_residue(shutdown)?;
    if shutdown.mounted_shutdown_attempt_count() != 0 {
        return Err(
            ExecutableLifecycleCleanupFailure::MountedPresentationNotQuiescent(
                shutdown.mounted_shutdown_attempt_count(),
            ),
        );
    }
    require_zero_visual_residue(shutdown)?;
    if shutdown.released_surface_count() == 0 {
        return Err(ExecutableLifecycleCleanupFailure::NoReleasedSurface);
    }
    if !installation_cleanup.removed_owned_root() {
        return Err(ExecutableLifecycleCleanupFailure::InstallationResidue);
    }
    Ok(ExecutableLifecycleCleanupEvidence {
        close_request: causal.close_request,
        shutdown_envelope: causal.shutdown_envelope,
        shutdown,
        lifecycle_measurement: causal.lifecycle_measurement,
        lifecycle_envelopes: causal.lifecycle_envelopes,
        successful_exit,
        native_close_evidence,
        installation_cleanup,
    })
}

fn require_zero_query_residue(
    shutdown: PlatformPulseShutdownCompleted,
) -> Result<(), ExecutableLifecycleCleanupFailure> {
    if !shutdown.query_close_complete() {
        return Err(ExecutableLifecycleCleanupFailure::QueryCloseIncomplete);
    }
    if !shutdown.query_watcher_joined() {
        return Err(ExecutableLifecycleCleanupFailure::QueryWatcherNotJoined);
    }
    if shutdown.pending_query_observation_count() != 0 {
        return Err(ExecutableLifecycleCleanupFailure::PendingQueryObservations(
            shutdown.pending_query_observation_count(),
        ));
    }
    if !shutdown.query_owner_terminal() {
        return Err(ExecutableLifecycleCleanupFailure::QueryOwnerNotTerminal);
    }
    Ok(())
}

fn require_zero_intent_residue(
    shutdown: PlatformPulseShutdownCompleted,
) -> Result<(), ExecutableLifecycleCleanupFailure> {
    if !shutdown.intent_watcher_joined() {
        return Err(ExecutableLifecycleCleanupFailure::IntentWatcherNotJoined);
    }
    if shutdown.pending_intent_input_count() != 0 {
        return Err(ExecutableLifecycleCleanupFailure::PendingIntentInputs(
            shutdown.pending_intent_input_count(),
        ));
    }
    if !shutdown.intent_resources_empty() {
        return Err(ExecutableLifecycleCleanupFailure::IntentResourcesNotEmpty);
    }
    Ok(())
}

fn require_zero_visual_residue(
    shutdown: PlatformPulseShutdownCompleted,
) -> Result<(), ExecutableLifecycleCleanupFailure> {
    let capture = (
        shutdown.cancelled_visual_capture_count(),
        shutdown.disposed_visual_snapshot_count(),
        shutdown.disposed_visual_pixel_bytes(),
        shutdown.disposed_visual_structural_bytes(),
    );
    if capture != (0, 0, 0, 0) {
        return Err(ExecutableLifecycleCleanupFailure::VisualCaptureResidue {
            cancelled: capture.0,
            disposed_snapshots: capture.1,
            disposed_pixel_bytes: capture.2,
            disposed_structural_bytes: capture.3,
        });
    }
    let overlay = (
        shutdown.cancelled_pending_overlay_count(),
        shutdown.disposed_published_overlay_count(),
        shutdown.disposed_clearing_overlay_count(),
    );
    if overlay != (0, 0, 0) {
        return Err(ExecutableLifecycleCleanupFailure::VisualOverlayResidue {
            pending: overlay.0,
            published: overlay.1,
            clearing: overlay.2,
        });
    }
    Ok(())
}

impl CausalLifecycleCleanupObservationSet {
    pub(crate) fn new(
        process_id: u32,
        close_request: NormalNativeCloseRequestObservation,
        shutdown_envelope: PlatformPulseLifecycleObservationEnvelope,
        lifecycle_measurement: LifecycleStreamMeasurement,
        lifecycle_envelopes: Vec<PlatformPulseLifecycleObservationEnvelope>,
    ) -> Self {
        Self {
            process_id,
            close_request,
            shutdown_envelope,
            lifecycle_measurement,
            lifecycle_envelopes,
        }
    }

    pub(crate) fn join_resource_disposition(
        self,
        successful_exit: SuccessfulPlatformPulseExit,
        native_close_evidence: PlatformPulseNativeCloseEvidence,
        installation_cleanup: PulseInstallationCleanupEvidence,
    ) -> ExecutableLifecycleCleanupObservationSet {
        ExecutableLifecycleCleanupObservationSet {
            causal: self,
            successful_exit,
            native_close_evidence,
            installation_cleanup,
        }
    }
}

impl ExecutableLifecycleCleanupEvidence {
    pub(crate) fn close_request(&self) -> NormalNativeCloseRequestObservation {
        self.close_request
    }

    pub(crate) fn close_request_count(&self) -> u32 {
        self.close_request.request_count()
    }

    pub(crate) fn shutdown_sequence(&self) -> u64 {
        self.shutdown_envelope.sequence().value()
    }

    pub(crate) fn shutdown(&self) -> PlatformPulseShutdownCompleted {
        self.shutdown
    }

    pub(crate) fn lifecycle_measurement(&self) -> LifecycleStreamMeasurement {
        self.lifecycle_measurement
    }

    pub(crate) fn lifecycle_envelopes(&self) -> &[PlatformPulseLifecycleObservationEnvelope] {
        &self.lifecycle_envelopes
    }

    pub(crate) fn successful_exit(&self) -> SuccessfulPlatformPulseExit {
        self.successful_exit
    }

    pub(crate) const fn native_close_evidence(&self) -> &PlatformPulseNativeCloseEvidence {
        &self.native_close_evidence
    }

    pub(crate) fn installation_removed(&self) -> bool {
        self.installation_cleanup.removed_owned_root()
    }

    pub(crate) fn installation_cleanup(&self) -> PulseInstallationCleanupEvidence {
        self.installation_cleanup
    }
}
