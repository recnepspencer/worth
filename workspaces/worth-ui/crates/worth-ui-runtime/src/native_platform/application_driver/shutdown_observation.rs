pub(super) struct UiNativeDriverShutdownEvidence {
    runtime_derived_state_reconstruction:
        Option<worth_ui_host_native::UiNativeClientDerivedStateReconstructionObservation>,
    observation_ingress_counts: [u64; 5],
    visual_snapshot: Option<worth_ui_host_native::UiNativeClientVisualSnapshotObservation>,
}

impl UiNativeDriverShutdownEvidence {
    pub(super) fn captured(
        runtime_derived_state_reconstruction: Option<
            worth_ui_host_native::UiNativeClientDerivedStateReconstructionObservation,
        >,
        observation_ingress_counts: [u64; 5],
        visual_snapshot: Option<worth_ui_host_native::UiNativeClientVisualSnapshotObservation>,
    ) -> Self {
        Self {
            runtime_derived_state_reconstruction,
            observation_ingress_counts,
            visual_snapshot,
        }
    }

    pub(super) fn empty() -> Self {
        Self::captured(None, [0; 5], None)
    }

    pub(super) fn finalize(
        self,
        query_close: &crate::facade::entry::UiNativeApplicationQueryCloseObservation,
    ) -> worth_ui_host_native::UiNativeClientShutdownObservation {
        host_shutdown_observation(query_close)
            .with_derived_state_reconstruction(self.runtime_derived_state_reconstruction)
            .with_observation_ingress(
                worth_ui_host_native::UiNativeClientObservationIngressObservation::reported(
                    self.observation_ingress_counts,
                ),
            )
            .with_visual_snapshot(self.visual_snapshot)
    }
}

pub(super) fn host_shutdown_observation(
    query_close: &crate::facade::entry::UiNativeApplicationQueryCloseObservation,
) -> worth_ui_host_native::UiNativeClientShutdownObservation {
    worth_ui_host_native::UiNativeClientShutdownObservation::from_client_with_presentation_transitions(
        query_close.closed_query_resources(),
        query_close.query_close_complete(),
        map_transition_observations(query_close.transitions()),
        query_close.transition_trace_complete(),
    )
    .with_text_presentation_work(
        map_text_presentation_work(query_close.text_presentation_work()),
        query_close.text_presentation_work_trace_complete(),
    )
    .with_authored_mounted_instances(
        query_close
            .authored_mounted_instances()
            .to_vec()
            .into_boxed_slice(),
    )
    .with_resources(client_resource_observation(
        query_close.client_resource_peaks(),
    ))
    .with_shutdown_attempts(map_shutdown_attempts(
        query_close.mounted_shutdown_attempts(),
    ))
    .with_intent_resources_empty(query_close.intent_resources_empty())
}

pub(crate) fn map_shutdown_attempts(
    attempts: &[crate::mounting::UiMountedPresentationShutdownAttempt],
) -> Box<[worth_ui_host_native::UiNativeClientShutdownAttemptObservation]> {
    attempts
        .iter()
        .map(|attempt| {
            let disposition = match attempt.disposition() {
                crate::mounting::UiMountedPresentationShutdownDisposition::CancelledBeforeEffects => {
                    worth_ui_host_native::UiNativeClientShutdownAttemptDisposition::CancelledBeforeEffects
                }
                crate::mounting::UiMountedPresentationShutdownDisposition::PresentationIndeterminate => {
                    worth_ui_host_native::UiNativeClientShutdownAttemptDisposition::PresentationIndeterminate
                }
            };
            worth_ui_host_native::UiNativeClientShutdownAttemptObservation::reported(
                attempt.attempt().diagnostic_value(),
                disposition,
                attempt
                    .affected_bindings()
                    .iter()
                    .map(|binding| binding.diagnostic_value()),
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

pub(super) fn client_resource_observation(
    peaks: [usize; 2],
) -> worth_ui_host_native::UiNativeClientResourceObservation {
    worth_ui_host_native::UiNativeClientResourceObservation::reported(peaks[0], peaks[1], 0, 0)
}

pub(super) fn map_text_presentation_work(
    observations: &[crate::native_platform::text_presentation::UiNativeTextPresentationWorkObservation],
) -> Box<[worth_ui_host_native::UiNativeClientTextPresentationWorkObservation]> {
    observations
        .iter()
        .map(|observation| {
            worth_ui_host_native::UiNativeClientTextPresentationWorkObservation::reported(
                observation.identity(),
                observation.work_counts(),
                observation.transcript_digests(),
                observation.intrinsic_glyph_runs(),
                observation.mechanic_identity_digests(),
                observation.binding_pin_identities().iter().copied(),
                observation
                    .active_mechanics()
                    .iter()
                    .map(|mechanic| client_mechanic_identity(*mechanic)),
                observation
                    .removed_mechanics()
                    .iter()
                    .map(|mechanic| client_mechanic_identity(*mechanic)),
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn client_mechanic_identity(
    observation: crate::native_platform::text_presentation::UiNativeTextPresentationMechanicObservation,
) -> worth_ui_host_native::UiNativeClientPresentationMechanicIdentityObservation {
    let mechanic = observation.mechanic();
    let (slot, row) = mechanic
        .semantic_text_identity_parts()
        .expect("text presentation work retains only semantic-text mechanics");
    worth_ui_host_native::UiNativeClientPresentationMechanicIdentityObservation::reported(
        mechanic.mounted_instance().diagnostic_value(),
        slot,
        row,
        observation.layout_digest(),
        observation.raster_key_set_digest(),
    )
}

pub(super) fn map_transition_observations(
    observations: &[worth_ui_query_binding::WorthUiPresentationTransitionObservation],
) -> Box<[worth_ui_host_native::UiNativeClientPresentationTransitionObservation]> {
    observations
        .iter()
        .copied()
        .map(|observation| {
            worth_ui_host_native::UiNativeClientPresentationTransitionObservation::reported(
                map_transition_kind(observation.kind()),
                observation.attempt().diagnostic_value(),
                observation.binding().diagnostic_value(),
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn map_transition_kind(
    kind: worth_ui_query_binding::WorthUiPresentationTransitionKind,
) -> worth_ui_host_native::UiNativeClientPresentationTransitionKind {
    use worth_ui_host_native::UiNativeClientPresentationTransitionKind as Host;
    use worth_ui_query_binding::WorthUiPresentationTransitionKind as Query;
    match kind {
        Query::Pending => Host::Pending,
        Query::Superseded => Host::Superseded,
        Query::StaleCompletionRejected => Host::StaleCompletionRejected,
        Query::Completed => Host::Completed,
        Query::DuplicateCompletionRejected => Host::DuplicateCompletionRejected,
        Query::Cancelled => Host::Cancelled,
        Query::Unresolved => Host::Unresolved,
        Query::RecoveryRequired => Host::RecoveryRequired,
        Query::ReconstructionCurrent => Host::ReconstructionCurrent,
        Query::TerminalClosed => Host::TerminalClosed,
    }
}
