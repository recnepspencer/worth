use serde::{Deserialize, Serialize};

use super::launch::PlatformPulseLaunchConfigurationDenialKind;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum PlatformPulseLifecycleObservation {
    ThemeSwitchSettled(super::theme::PlatformPulseThemeSwitchSettled),
    ProcessStarted(PlatformPulseProcessStarted),
    FirstFramePublished(PlatformPulseFirstFramePublished),
    NativeInputReached(super::native_input::PlatformPulseNativeInputReached),
    IntentInputAdmitted(super::intent::PlatformPulseIntentInputObservation),
    IntentExecutorStarted(super::intent::PlatformPulseIntentExecutorStartedObservation),
    IntentPosturePublished(super::intent::PlatformPulseIntentPosturePublished),
    IntentRoutingStopped(super::intent::PlatformPulseIntentRoutingStoppedObservation),
    SemanticFocusPublished(super::focus::PlatformPulseSemanticFocusPublished),
    PortalDismissed(PlatformPulsePortalDismissed),
    IntentCausalTrace(super::intent::PlatformPulseIntentCausalTraceObservation),
    QueryAction(super::intent::PlatformPulseQueryActionObservation),
    QueryProjectionIssued(super::query::PlatformPulseQueryProjectionEvidence),
    QueryProjectionPublished(super::query::PlatformPulseQueryProjectionPublished),
    VisualSnapshotCaptured(super::visual::PlatformPulseVisualSnapshotCaptured),
    VisualPointTrace(super::visual::PlatformPulseVisualPointTrace),
    VisualOverlayPublished(super::visual::PlatformPulseVisualOverlayPublished),
    VisualOverlayCleared(super::visual::PlatformPulseVisualOverlayCleared),
    VisualSnapshotRetired(super::visual::PlatformPulseVisualSnapshotRetired),
    RebindPublished(PlatformPulseReplacementPublished),
    RebindDeniedPreserving(PlatformPulseReplacementPreserved),
    VisualComparison(PlatformPulseVisualComparison),
    ShutdownCompleted(PlatformPulseShutdownCompleted),
    TerminalFailure(PlatformPulseTerminalFailure),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseVisualComparison {
    predecessor_snapshot: u64,
    successor_snapshot: u64,
    identity_rebound: bool,
    retained_pixels_differ: Option<bool>,
    structural_entries_examined: u64,
    retained_pixel_bytes_examined: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseProcessStarted {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseSourceSnapshotObservation {
    final_package_digest: u64,
    event_burst_digest: u64,
    source_sequence: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseApplicationGenerationObservation {
    semantic_package_fingerprint: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseMountedFrameObservation {
    pub(super) diagnostic_value: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulsePortalDismissed {
    frame: PlatformPulseMountedFrameObservation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseFirstFramePublished {
    pub(super) source: PlatformPulseSourceSnapshotObservation,
    pub(super) generation: PlatformPulseApplicationGenerationObservation,
    pub(super) frame: PlatformPulseMountedFrameObservation,
    pub(super) actual_native_effect_count: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseReplacementPublished {
    pub(super) source: PlatformPulseSourceSnapshotObservation,
    pub(super) predecessor_generation: PlatformPulseApplicationGenerationObservation,
    pub(super) active_generation: PlatformPulseApplicationGenerationObservation,
    pub(super) successor_frame: PlatformPulseMountedFrameObservation,
    pub(super) actual_native_effect_count: u64,
    pub(super) schema_transition:
        Option<super::schema_transition::PlatformPulseProjectionSchemaTransitionObservation>,
    pub(super) latest_focus_transition:
        Option<super::focus::PlatformPulseFocusTransitionInspection>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseReplacementPreserved {
    pub(super) source: PlatformPulseSourceSnapshotObservation,
    pub(super) active_generation: PlatformPulseApplicationGenerationObservation,
    pub(super) active_frame: PlatformPulseMountedFrameObservation,
    pub(super) denial_family: PlatformPulseReplacementDenialFamily,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlatformPulseReplacementDenialFamily {
    DslCompilation,
    SourceIngress,
    RuntimePreparation,
    Candidate,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseShutdownCompleted {
    pub(super) watcher_backend: PlatformPulseWatcherBackendObservation,
    pub(super) observed_notification_count: u64,
    pub(super) query_watcher_joined: bool,
    pub(super) pending_query_observation_count: u64,
    pub(super) intent_watcher_joined: bool,
    pub(super) pending_intent_input_count: u64,
    pub(super) theme_watch_released: bool,
    pub(super) intent_resources_empty: bool,
    pub(super) query_close_complete: bool,
    pub(super) query_owner_terminal: bool,
    pub(super) mounted_shutdown_attempt_count: u64,
    pub(super) host_session_released: bool,
    pub(super) released_surface_count: u64,
    pub(super) cancelled_visual_capture_count: u64,
    pub(super) disposed_visual_snapshot_count: u64,
    pub(super) disposed_visual_pixel_bytes: u64,
    pub(super) disposed_visual_structural_bytes: u64,
    pub(super) cancelled_pending_overlay_count: u64,
    pub(super) disposed_published_overlay_count: u64,
    pub(super) disposed_clearing_overlay_count: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlatformPulseWatcherBackendObservation {
    Fsevent,
    Inotify,
    Kqueue,
    ReadDirectoryChanges,
    OtherNative,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseTerminalFailure {
    family: PlatformPulseTerminalFailureFamily,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlatformPulseTerminalFailureFamily {
    LaunchConfiguration(PlatformPulseLaunchConfigurationDenialKind),
    FilesystemWatcher,
    ApplicationPreparation,
    QueryPreparation,
    QueryShutdown,
    IntentPreparation,
    CandidateSubmission,
    NativeSurfaceLaunch,
    MountedFrameExecution,
    NativeApplicationReplacement(PlatformPulseNativeRebindDenialStage),
    VisualIdentity,
    SourceWorkerPanicked,
    NativeEventLoop,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlatformPulseNativeRebindDenialStage {
    Source,
    ObservationTurn,
    ObservationAdmission,
    Classification,
    Scope,
    Identity,
    Planning,
    Preparation(PlatformPulseNativeRebindPreparationDenial),
    ManagedRebindAlreadyInFlight,
    ManagedRebindSessionMismatch,
    NonterminalOutcome,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlatformPulseNativeRebindPreparationDenial {
    ForeignSession,
    StaleSourceBasis,
    StalePredecessorGeneration,
    CandidateGenerationMismatch,
    TimedOutBeforeEffects,
    CancelledBeforeEffects,
    Reservation,
    CandidateBindingMismatch,
    CandidateAllocation,
    CandidateLowering,
    CandidateStaging,
    FrameBoundaryUnavailable,
    ContentMountedPreparation,
    CandidateMountedPreparation,
    CandidateOccurrenceGeometry,
    CandidateCutoverPreparation,
    PlannedChangeBecameSemanticNoOp,
    UnsupportedNonSourcePlan,
    ThemeSwitch,
    ConsequenceFrameMismatch,
    StaleConsequenceOwnerSnapshot,
    InvalidSemanticProof,
}

macro_rules! accessors {
    ($type:ty, $($name:ident : $return:ty),+ $(,)?) => {
        impl $type {
            $(pub fn $name(&self) -> $return { self.$name })+
        }
    };
}

accessors!(
    PlatformPulseSourceSnapshotObservation,
    final_package_digest: u64,
    event_burst_digest: u64,
    source_sequence: u64,
);
accessors!(
    PlatformPulseApplicationGenerationObservation,
    semantic_package_fingerprint: u64,
);
accessors!(PlatformPulseMountedFrameObservation, diagnostic_value: u64);
accessors!(PlatformPulsePortalDismissed, frame: PlatformPulseMountedFrameObservation);

impl PlatformPulsePortalDismissed {
    pub(super) fn from_publication(
        publication: &worth_ui::facade::app::UiMountedFramePublicationReceipt,
    ) -> Self {
        Self {
            frame: PlatformPulseMountedFrameObservation::from_publication(publication),
        }
    }
}

impl PlatformPulseMountedFrameObservation {
    pub(super) fn from_publication(
        publication: &worth_ui::facade::app::UiMountedFramePublicationReceipt,
    ) -> Self {
        Self {
            diagnostic_value: publication.frame().diagnostic_value(),
        }
    }
}
accessors!(
    PlatformPulseFirstFramePublished,
    source: PlatformPulseSourceSnapshotObservation,
    generation: PlatformPulseApplicationGenerationObservation,
    frame: PlatformPulseMountedFrameObservation,
    actual_native_effect_count: u64,
);
accessors!(
    PlatformPulseReplacementPublished,
    source: PlatformPulseSourceSnapshotObservation,
    predecessor_generation: PlatformPulseApplicationGenerationObservation,
    active_generation: PlatformPulseApplicationGenerationObservation,
    successor_frame: PlatformPulseMountedFrameObservation,
    actual_native_effect_count: u64,
);
impl PlatformPulseReplacementPublished {
    pub fn schema_transition(
        &self,
    ) -> Option<&super::schema_transition::PlatformPulseProjectionSchemaTransitionObservation> {
        self.schema_transition.as_ref()
    }

    pub const fn latest_focus_transition(
        &self,
    ) -> Option<super::focus::PlatformPulseFocusTransitionInspection> {
        self.latest_focus_transition
    }
}
accessors!(
    PlatformPulseReplacementPreserved,
    source: PlatformPulseSourceSnapshotObservation,
    active_generation: PlatformPulseApplicationGenerationObservation,
    active_frame: PlatformPulseMountedFrameObservation,
    denial_family: PlatformPulseReplacementDenialFamily,
);
accessors!(
    PlatformPulseVisualComparison,
    predecessor_snapshot: u64,
    successor_snapshot: u64,
    identity_rebound: bool,
    retained_pixels_differ: Option<bool>,
    structural_entries_examined: u64,
    retained_pixel_bytes_examined: u64,
);
accessors!(
    PlatformPulseShutdownCompleted,
    watcher_backend: PlatformPulseWatcherBackendObservation,
    observed_notification_count: u64,
    query_watcher_joined: bool,
    pending_query_observation_count: u64,
    intent_watcher_joined: bool,
    pending_intent_input_count: u64,
    theme_watch_released: bool,
    intent_resources_empty: bool,
    query_close_complete: bool,
    query_owner_terminal: bool,
    mounted_shutdown_attempt_count: u64,
    host_session_released: bool,
    released_surface_count: u64,
    cancelled_visual_capture_count: u64,
    disposed_visual_snapshot_count: u64,
    disposed_visual_pixel_bytes: u64,
    disposed_visual_structural_bytes: u64,
    cancelled_pending_overlay_count: u64,
    disposed_published_overlay_count: u64,
    disposed_clearing_overlay_count: u64,
);
accessors!(
    PlatformPulseTerminalFailure,
    family: PlatformPulseTerminalFailureFamily,
);

impl PlatformPulseProcessStarted {
    pub(super) fn new() -> Self {
        Self {}
    }
}

impl PlatformPulseSourceSnapshotObservation {
    pub(super) fn from_revision(
        revision: &worth_ui::facade::source::WorthUiSourcePackageRevision,
    ) -> Self {
        Self {
            final_package_digest: revision.final_package_digest(),
            event_burst_digest: revision.event_burst_digest(),
            source_sequence: revision.sequence(),
        }
    }
}

impl PlatformPulseApplicationGenerationObservation {
    pub(super) fn from_generation(
        generation: &worth_ui::facade::app::WorthUiPreparedApplicationGenerationIdentity,
    ) -> Self {
        Self {
            semantic_package_fingerprint: generation
                .semantic_package_identity()
                .narrowing_fingerprint(),
        }
    }
}

impl PlatformPulseTerminalFailure {
    pub(super) fn new(family: PlatformPulseTerminalFailureFamily) -> Self {
        Self { family }
    }
}

impl PlatformPulseVisualComparison {
    pub(super) fn from_comparison(
        comparison: worth_ui::facade::inspection::UiVisualSnapshotComparison,
    ) -> Self {
        let snapshots = comparison.snapshot_identities();
        Self {
            predecessor_snapshot: snapshots[0],
            successor_snapshot: snapshots[1],
            identity_rebound: comparison.continuity()
                == worth_ui::facade::inspection::UiVisualIdentityContinuity::Rebound,
            retained_pixels_differ: comparison.retained_pixels_differ(),
            structural_entries_examined: u64::try_from(
                comparison.cost().structural_entries_examined(),
            )
            .unwrap_or(u64::MAX),
            retained_pixel_bytes_examined: u64::try_from(
                comparison.cost().retained_pixel_bytes_examined(),
            )
            .unwrap_or(u64::MAX),
        }
    }
}
